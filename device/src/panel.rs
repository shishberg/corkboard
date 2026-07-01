//! SPI/GPIO driver for the Waveshare 7.3" E6 (Spectra 6) panel, 800x480,
//! 6-colour. The real hardware access (`spidev`/`gpio-cdev`, which need
//! Linux ioctls) is gated to `target_os = "linux"` — `Panel::open` doesn't
//! exist on other platforms, so `main.rs` can only ever construct a `Panel`
//! there, and macOS dev machines fall back to `WebPreview`. Everything else
//! in this module (the register sequence + BUSY-timeout logic, behind the
//! `GpioLine`/`SpiBus` seams) compiles and is unit-tested on every platform,
//! so the hardware-safety logic doesn't need real hardware or a Linux host
//! to verify.
//!
//! The register sequence, reset/busy timing, and colour codes below are a
//! byte-for-byte port of Waveshare's own Python `epd7in3e` driver (verified
//! identical between the PhotoPainter kit's copy and the canonical
//! `waveshareteam/e-Paper` repo). The C demo agrees on everything except one
//! register write in `turn_on_display` — see the note there. See
//! `.mex/patterns/deploy-to-orange-pi.md` for the source and for what's
//! still unverified: the GPIO chip/line numbers are Raspberry Pi BCM numbers
//! in Waveshare's demo and do NOT carry over to the Orange Pi's Allwinner
//! H618 numbering, even though the 40-pin header is physically pin-compatible.
//! Determine the real numbers with `gpioinfo` once the panel is wired to the
//! board — there are no hardcoded pin defaults here on purpose, so a wrong
//! guess can't silently drive the wrong line.
//!
//! Usage model: each `show()` drives the panel as a self-contained one-shot
//! — hardware reset, full init (register config + POWER_ON), image + refresh
//! + POWER_OFF, then DEEP_SLEEP — exactly how Waveshare's own demo and other
//! persistent apps (e.g. PaperPiAI) drive this hardware: `init(); display();
//! sleep()` per image. Two consequences worth stating:
//!  * The leading hardware reset re-establishes the controller's state from
//!    scratch every render, so a previous render that timed out or failed
//!    partway leaves nothing to clean up across calls — hence no fault latch
//!    and no cross-render state. Each render also re-asserts the PWR gate at
//!    the top (`power_up`), so it's fine for a failure to have left it low.
//!    If a render fails *after* POWER_ON, `show()` runs `emergency_power_off`:
//!    a best-effort SPI POWER_OFF *and* — the part that still works when SPI
//!    itself is what failed — dropping the PWR gate to hard-cut the
//!    high-voltage rail (the acute damage hazard). The next render's
//!    `power_up` + reset recovers from there.
//!  * On the success path the panel sits in DEEP_SLEEP between renders, its
//!    intended idle state, with PWR still asserted. We don't de-assert PWR on
//!    process shutdown (that would need graceful-shutdown wiring `main.rs`
//!    doesn't have, and deep sleep already draws almost nothing).

use std::sync::Mutex;
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
#[cfg(target_os = "linux")]
use spidev::{SpiModeFlags, Spidev, SpidevOptions};

use crate::display::Display;
use crate::document::Colour;

const WIDTH: usize = 800;
const HEIGHT: usize = 480;

/// Max bytes per write to the SPI device, matching the stock Linux `spidev`
/// kernel module's default `bufsiz` (4096). See `send_data_bulk` for why
/// this is a plain chunk size, not a chip-select-preserving trick.
const SPI_CHUNK: usize = 4096;

/// How long to wait for BUSY to go idle before giving up on a render. The
/// panel's full refresh takes tens of seconds (`context/hardware.md`), so
/// this is deliberately generous — it can only fire on a genuinely stuck
/// panel, and a render that hits it just errors (the next render resets and
/// retries from scratch).
const BUSY_TIMEOUT: Duration = Duration::from_secs(120);

/// How long to let the panel's power rail stabilise after asserting the PWR
/// gate, before driving the reset pulse. Paid once per render (negligible next
/// to a render's tens of seconds) so PWR handling can stay stateless: every
/// render re-asserts PWR unconditionally rather than tracking rail state.
const PWR_SETTLE: Duration = Duration::from_millis(10);

/// Seam over `gpio_cdev::LineHandle` so the reset/BUSY/command logic can be
/// unit-tested with a fake line instead of a real GPIO character device.
trait GpioLine: Send {
    fn set_value(&self, value: u8) -> anyhow::Result<()>;
    fn get_value(&self) -> anyhow::Result<u8>;
}

#[cfg(target_os = "linux")]
impl GpioLine for LineHandle {
    fn set_value(&self, value: u8) -> anyhow::Result<()> {
        LineHandle::set_value(self, value)?;
        Ok(())
    }

    fn get_value(&self) -> anyhow::Result<u8> {
        Ok(LineHandle::get_value(self)?)
    }
}

/// Seam over `spidev::Spidev`, for the same reason as `GpioLine`.
trait SpiBus: Send {
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()>;
}

#[cfg(target_os = "linux")]
impl SpiBus for Spidev {
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        std::io::Write::write_all(self, buf)
    }
}

pub struct PanelConfig {
    pub spi_path: String,
    pub gpiochip_path: String,
    pub rst_line: u32,
    pub dc_line: u32,
    pub busy_line: u32,
    /// PhotoPainter carrier board's panel power-enable line. `None` only when
    /// explicitly opted out (a bare HAT with no gate); a missing PWR var is an
    /// error, not a silent `None` — see `from_getter`.
    pub pwr_line: Option<u32>,
}

impl PanelConfig {
    /// Reads `CORKBOARD_PANEL_*` env vars. No pin-number defaults — see the
    /// module doc comment for why guessing here would be worse than failing
    /// loudly.
    pub fn from_env() -> anyhow::Result<Self> {
        Self::from_getter(&|name: &str| std::env::var(name))
    }

    /// Does the actual parsing, taking the var-lookup as a parameter instead
    /// of calling `std::env::var` directly, so tests can supply a fake lookup
    /// instead of mutating real process env vars — mutating shared process
    /// state from tests running in parallel threads is a flakiness trap.
    fn from_getter(
        get: &dyn Fn(&str) -> Result<String, std::env::VarError>,
    ) -> anyhow::Result<Self> {
        fn required(
            get: &dyn Fn(&str) -> Result<String, std::env::VarError>,
            name: &str,
        ) -> anyhow::Result<String> {
            get(name).map_err(|_| anyhow::anyhow!("{name} is not set"))
        }
        fn parse_line(name: &str, value: String) -> anyhow::Result<u32> {
            value.parse().map_err(|e| anyhow::anyhow!("{name}: {e}"))
        }
        fn required_line(
            get: &dyn Fn(&str) -> Result<String, std::env::VarError>,
            name: &str,
        ) -> anyhow::Result<u32> {
            parse_line(name, required(get, name)?)
        }
        // PWR is the carrier board's panel power-enable gate. It is NOT
        // silently optional: defaulting a missing PWR var to "no gate" would
        // let a typo or forgotten var run a carrier-board panel unpowered,
        // back-feeding its lines through the ESD/protection diodes — the exact
        // hazard the power-first ordering exists to prevent. So a missing
        // PWR_LINE is only allowed when the operator *explicitly* opts out with
        // CORKBOARD_PANEL_NO_PWR=1 (a bare HAT with no gate to switch);
        // otherwise it's a hard error.
        fn pwr_line(
            get: &dyn Fn(&str) -> Result<String, std::env::VarError>,
        ) -> anyhow::Result<Option<u32>> {
            match get("CORKBOARD_PANEL_PWR_LINE") {
                Ok(v) => Ok(Some(parse_line("CORKBOARD_PANEL_PWR_LINE", v)?)),
                Err(e @ std::env::VarError::NotUnicode(_)) => {
                    Err(anyhow::anyhow!("CORKBOARD_PANEL_PWR_LINE: {e}"))
                }
                Err(std::env::VarError::NotPresent) => match get("CORKBOARD_PANEL_NO_PWR") {
                    Ok(v) if v == "1" => Ok(None),
                    Ok(v) => Err(anyhow::anyhow!(
                        "CORKBOARD_PANEL_NO_PWR must be \"1\" if set, got {v:?}"
                    )),
                    Err(_) => Err(anyhow::anyhow!(
                        "CORKBOARD_PANEL_PWR_LINE is not set; set it to the panel's \
                         power-enable line, or set CORKBOARD_PANEL_NO_PWR=1 for a bare \
                         HAT with no power gate"
                    )),
                },
            }
        }
        Ok(PanelConfig {
            spi_path: get("CORKBOARD_PANEL_SPI").unwrap_or_else(|_| "/dev/spidev0.0".to_string()),
            gpiochip_path: required(get, "CORKBOARD_PANEL_GPIOCHIP")?,
            rst_line: required_line(get, "CORKBOARD_PANEL_RST_LINE")?,
            dc_line: required_line(get, "CORKBOARD_PANEL_DC_LINE")?,
            busy_line: required_line(get, "CORKBOARD_PANEL_BUSY_LINE")?,
            pwr_line: pwr_line(get)?,
        })
    }
}

struct Inner {
    spi: Box<dyn SpiBus>,
    rst: Box<dyn GpioLine>,
    dc: Box<dyn GpioLine>,
    busy: Box<dyn GpioLine>,
    /// The carrier board's panel power-enable gate, if wired. Re-asserted high
    /// at the top of every render (`power_up`), and dropped low as a hard rail
    /// cut on failure (`emergency_power_off`). Holding the handle also keeps
    /// the line claimed for the `Panel`'s lifetime — dropping it would release
    /// the line back to the kernel and let it float.
    pwr: Option<Box<dyn GpioLine>>,
}

pub struct Panel {
    inner: Mutex<Inner>,
}

#[cfg(target_os = "linux")]
impl Panel {
    pub fn open(cfg: &PanelConfig) -> anyhow::Result<Self> {
        let mut chip = Chip::new(&cfg.gpiochip_path)?;

        fn request_line(
            chip: &mut Chip,
            line: u32,
            flags: LineRequestFlags,
            default: u8,
            consumer: &str,
        ) -> anyhow::Result<Box<dyn GpioLine>> {
            Ok(Box::new(
                chip.get_line(line)?.request(flags, default, consumer)?,
            ))
        }

        // Power the panel before driving *anything* into it — GPIO or SPI.
        // Waveshare's `module_init()` asserts PWR before it opens SPI or
        // touches RST/DC. Driving a line into a panel whose power gate is off
        // back-feeds current through its ESD/input protection diodes; that's
        // true of a logic-high RST *and* of the CS/SCLK/MOSI idle levels that
        // opening spidev can drive. So PWR is claimed-and-asserted first
        // (`request()` sets the line atomically as it claims it, leaving no
        // claimed-but-low window), then SPI is brought up, then the remaining
        // GPIOs. RST defaults *low* until the deliberate reset pulse in
        // `init`.
        //
        // We do NOT init the panel here: init happens per render (see
        // `render`), so `open()` only brings the hardware up and leaves the
        // panel idle until the first `show()`.
        let pwr = cfg
            .pwr_line
            .map(|line| {
                request_line(
                    &mut chip,
                    line,
                    LineRequestFlags::OUTPUT,
                    1,
                    "corkboard-panel-pwr",
                )
            })
            .transpose()?;

        let mut spi = Spidev::open(&cfg.spi_path)?;
        spi.configure(
            &SpidevOptions::new()
                .bits_per_word(8)
                .max_speed_hz(4_000_000)
                .mode(SpiModeFlags::SPI_MODE_0)
                .build(),
        )?;

        let rst = request_line(
            &mut chip,
            cfg.rst_line,
            LineRequestFlags::OUTPUT,
            0,
            "corkboard-panel-rst",
        )?;
        let dc = request_line(
            &mut chip,
            cfg.dc_line,
            LineRequestFlags::OUTPUT,
            0,
            "corkboard-panel-dc",
        )?;
        let busy = request_line(
            &mut chip,
            cfg.busy_line,
            LineRequestFlags::INPUT,
            0,
            "corkboard-panel-busy",
        )?;

        Ok(Panel {
            inner: Mutex::new(Inner {
                spi: Box::new(spi),
                rst,
                dc,
                busy,
                pwr,
            }),
        })
    }
}

impl Display for Panel {
    fn show(&self, png: &[u8]) -> anyhow::Result<()> {
        let img = image::load_from_memory(png)?.to_rgb8();
        let packed = pack(&img)?;

        let mut inner = self.inner.lock().unwrap();
        let result = render(&mut inner, &packed);
        if result.is_err() {
            // A render can only fail with the rail live between `init`'s
            // POWER_ON and the refresh's POWER_OFF. Clear the high-voltage rail
            // — the acute damage hazard — before returning: a best-effort SPI
            // POWER_OFF plus a hard PWR-gate drop that works even if SPI is the
            // thing that failed. (If the failure was before POWER_ON this is a
            // harmless extra shutdown; not worth tracking rail state to avoid.)
            // The next render's `power_up` + reset recovers everything.
            emergency_power_off(&mut inner);
        }
        result
    }
}

/// One full refresh as a self-contained one-shot, mirroring Waveshare's own
/// `init(); display(); sleep()` usage of this panel. See the module doc
/// comment for why each render re-inits and ends in deep sleep.
fn render(inner: &mut Inner, packed: &[u8]) -> anyhow::Result<()> {
    power_up(inner)?;
    init(inner)?;
    send_command(inner, 0x10)?; // DATA_START_TRANSMISSION
    send_data_bulk(inner, packed)?;
    turn_on_display(inner)?;
    deep_sleep(inner)?;
    Ok(())
}

/// Asserts the carrier PWR gate (if wired) and lets the rail settle, at the
/// top of every render. Doing it unconditionally every render is what lets a
/// failed render's `emergency_power_off` drop PWR as a hard kill without any
/// cross-render state to restore — the next render just re-asserts here. A
/// near no-op when PWR is already high, and skipped entirely on a bare HAT.
fn power_up(inner: &mut Inner) -> anyhow::Result<()> {
    if let Some(pwr) = inner.pwr.as_ref() {
        pwr.set_value(1)?;
        std::thread::sleep(PWR_SETTLE);
    }
    Ok(())
}

/// Hardware reset + full register configuration, ending with POWER_ON (as the
/// reference does — the refresh in `turn_on_display` follows immediately).
fn init(inner: &mut Inner) -> anyhow::Result<()> {
    reset(inner.rst.as_ref())?;
    wait_busy(inner.busy.as_ref(), BUSY_TIMEOUT)?;
    std::thread::sleep(Duration::from_millis(30));

    send_command(inner, 0xAA)?;
    for b in [0x49, 0x55, 0x20, 0x08, 0x09, 0x18] {
        send_data(inner, b)?;
    }

    send_command(inner, 0x01)?;
    send_data(inner, 0x3F)?;

    send_command(inner, 0x00)?;
    send_data(inner, 0x5F)?;
    send_data(inner, 0x69)?;

    send_command(inner, 0x03)?;
    for b in [0x00, 0x54, 0x00, 0x44] {
        send_data(inner, b)?;
    }

    send_command(inner, 0x05)?;
    for b in [0x40, 0x1F, 0x1F, 0x2C] {
        send_data(inner, b)?;
    }

    send_command(inner, 0x06)?;
    for b in [0x6F, 0x1F, 0x17, 0x49] {
        send_data(inner, b)?;
    }

    send_command(inner, 0x08)?;
    for b in [0x6F, 0x1F, 0x1F, 0x22] {
        send_data(inner, b)?;
    }

    send_command(inner, 0x30)?;
    send_data(inner, 0x03)?;

    send_command(inner, 0x50)?;
    send_data(inner, 0x3F)?;

    send_command(inner, 0x60)?;
    send_data(inner, 0x02)?;
    send_data(inner, 0x00)?;

    send_command(inner, 0x61)?;
    for b in [0x03, 0x20, 0x01, 0xE0] {
        send_data(inner, b)?;
    }

    send_command(inner, 0x84)?;
    send_data(inner, 0x01)?;

    send_command(inner, 0xE3)?;
    send_data(inner, 0x2F)?;

    send_command(inner, 0x04)?; // POWER_ON
    wait_busy(inner.busy.as_ref(), BUSY_TIMEOUT)?;
    Ok(())
}

/// Powers the rail on, triggers the refresh, and powers it back off — the
/// panel's actual "paint what was just sent" cycle.
///
/// NOTE on a discrepancy between Waveshare's own two reference drivers: the C
/// demo (`EPD_7IN3E_TurnOnDisplay`) repeats command `0x06` with data
/// `0x6F,0x1F,0x17,0x49` here (comment: "Second setting") between POWER_ON and
/// DISPLAY_REFRESH; the Python demo this port follows does not. This holds in
/// Waveshare's canonical repo too, not just the PhotoPainter kit. Left out to
/// match Python — flagged as the first suspect if a real panel refuses to
/// refresh correctly.
fn turn_on_display(inner: &mut Inner) -> anyhow::Result<()> {
    send_command(inner, 0x04)?; // POWER_ON
    wait_busy(inner.busy.as_ref(), BUSY_TIMEOUT)?;

    send_command(inner, 0x12)?; // DISPLAY_REFRESH
    send_data(inner, 0x00)?;
    wait_busy(inner.busy.as_ref(), BUSY_TIMEOUT)?;

    send_command(inner, 0x02)?; // POWER_OFF
    send_data(inner, 0x00)?;
    wait_busy(inner.busy.as_ref(), BUSY_TIMEOUT)?;
    Ok(())
}

/// Puts the controller into DEEP_SLEEP — its intended state between renders.
/// Exited by the hardware reset at the start of the next `init`.
fn deep_sleep(inner: &mut Inner) -> anyhow::Result<()> {
    send_command(inner, 0x07)?; // DEEP_SLEEP
    send_data(inner, 0xA5)?;
    Ok(())
}

/// Best-effort panel shutdown, ignoring every error — the on-failure safety
/// net in `show()`. Kills the high-voltage rail (the acute damage hazard) two
/// ways: the SPI POWER_OFF command (a clean stop, if SPI still works) and
/// dropping the carrier PWR gate (a hard cut that still works when SPI itself
/// is what failed). RST/DC are driven low first so they aren't left sitting
/// high against a panel we're about to unpower (back-feeding its protection
/// diodes), matching the reference's `module_exit` ordering. No BUSY-wait: a
/// stuck panel would just hang again, and the commands are sent regardless.
///
/// One residual we accept: we don't close spidev, so its CS line keeps idling
/// high against the now-unpowered panel until the next render's `power_up`.
/// That's a mild, current-limited back-feed on a single line (the reference
/// tolerates the same steady state while the panel deep-sleeps between images);
/// fully closing it would mean GPIO-driven CS, which we deliberately don't do.
fn emergency_power_off(inner: &mut Inner) {
    let _ = send_command(inner, 0x02); // POWER_OFF
    let _ = send_data(inner, 0x00);
    let _ = inner.rst.set_value(0);
    let _ = inner.dc.set_value(0);
    if let Some(pwr) = inner.pwr.as_ref() {
        let _ = pwr.set_value(0);
    }
}

fn reset(rst: &dyn GpioLine) -> anyhow::Result<()> {
    rst.set_value(1)?;
    std::thread::sleep(Duration::from_millis(20));
    rst.set_value(0)?;
    std::thread::sleep(Duration::from_millis(2));
    rst.set_value(1)?;
    std::thread::sleep(Duration::from_millis(20));
    Ok(())
}

/// Polls BUSY until it reads idle, or errors on timeout. BUSY reads low while
/// the panel is busy, high when idle (Waveshare's polarity for this panel —
/// not the more common inverse). Waveshare waits unconditionally; we cap it
/// (see `BUSY_TIMEOUT`) so a stuck panel errors this render instead of hanging
/// the render thread forever.
fn wait_busy(busy: &dyn GpioLine, timeout: Duration) -> anyhow::Result<()> {
    let deadline = Instant::now() + timeout;
    while busy.get_value()? == 0 {
        if Instant::now() > deadline {
            anyhow::bail!("panel BUSY line never went idle (timed out after {timeout:?})");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Ok(())
}

fn send_command(inner: &mut Inner, cmd: u8) -> anyhow::Result<()> {
    inner.dc.set_value(0)?;
    inner.spi.write_all(&[cmd])?;
    Ok(())
}

fn send_data(inner: &mut Inner, data: u8) -> anyhow::Result<()> {
    inner.dc.set_value(1)?;
    inner.spi.write_all(&[data])?;
    Ok(())
}

/// Sends a large buffer as a sequence of plain writes, chunked to stay under
/// the kernel `spidev` driver's per-write transfer cap (`SPI_CHUNK`). This
/// matches how Waveshare's Python driver behaves once `spidev.writebytes2`
/// splits a buffer bigger than `bufsiz` — the proven-safe path for this panel.
///
/// Do NOT try to hold chip-select asserted across chunks via `cs_change` on a
/// manual `SpidevTransfer`: the `spidev` crate documents `cs_change` as
/// "deselect device before starting the next transfer", so it *adds* CS edges
/// rather than removing them. If a real panel ever shows a torn frame, raise
/// the kernel's `spidev.bufsiz` and send the whole 192,000-byte frame in one
/// write instead.
fn send_data_bulk(inner: &mut Inner, data: &[u8]) -> anyhow::Result<()> {
    inner.dc.set_value(1)?;
    for chunk in data.chunks(SPI_CHUNK) {
        inner.spi.write_all(chunk)?;
    }
    Ok(())
}

fn nearest_colour(rgb: [u8; 3]) -> Colour {
    Colour::ALL
        .iter()
        .min_by_key(|c| {
            let p = c.rgb();
            let dr = p[0] as i32 - rgb[0] as i32;
            let dg = p[1] as i32 - rgb[1] as i32;
            let db = p[2] as i32 - rgb[2] as i32;
            dr * dr + dg * dg + db * db
        })
        .cloned()
        .expect("Colour::ALL is non-empty")
}

/// Quantises an RGB image (expected to already be exact-palette, per
/// `render.rs`'s dithering guarantee — nearest-match here is a safety net, not
/// the primary quantisation step) into the panel's packed 4-bit-per-pixel
/// format: two pixels per byte, high nibble first.
fn pack(rgb: &image::RgbImage) -> anyhow::Result<Vec<u8>> {
    if rgb.width() as usize != WIDTH || rgb.height() as usize != HEIGHT {
        anyhow::bail!(
            "panel expects {}x{}, got {}x{}",
            WIDTH,
            HEIGHT,
            rgb.width(),
            rgb.height()
        );
    }
    let codes: Vec<u8> = rgb
        .pixels()
        .map(|p| nearest_colour([p[0], p[1], p[2]]).panel_code())
        .collect();
    let mut packed = vec![0u8; codes.len() / 2];
    for (i, pair) in codes.chunks(2).enumerate() {
        packed[i] = (pair[0] << 4) | pair[1];
    }
    Ok(packed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// Builds a getter closure for `PanelConfig::from_getter` backed by a
    /// fixed list of pairs, instead of touching real process env vars (which
    /// would race with every other test running in parallel).
    fn env_from(
        pairs: &'static [(&'static str, &'static str)],
    ) -> impl Fn(&str) -> Result<String, std::env::VarError> {
        move |name: &str| {
            pairs
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| value.to_string())
                .ok_or(std::env::VarError::NotPresent)
        }
    }

    /// A minimal valid env: all required pins plus the explicit PWR opt-out
    /// (`NO_PWR=1`), since a missing PWR var is now an error rather than a
    /// silent `None`. Tests that want a real PWR line override with PWR_LINE.
    const REQUIRED_PINS: &[(&str, &str)] = &[
        ("CORKBOARD_PANEL_GPIOCHIP", "/dev/gpiochip0"),
        ("CORKBOARD_PANEL_RST_LINE", "5"),
        ("CORKBOARD_PANEL_DC_LINE", "6"),
        ("CORKBOARD_PANEL_BUSY_LINE", "7"),
        ("CORKBOARD_PANEL_NO_PWR", "1"),
    ];

    #[test]
    fn from_getter_uses_spi_default_and_leaves_pwr_none_when_opted_out() {
        let get = env_from(REQUIRED_PINS);
        let cfg = PanelConfig::from_getter(&get).unwrap();

        assert_eq!(cfg.spi_path, "/dev/spidev0.0");
        assert_eq!(cfg.gpiochip_path, "/dev/gpiochip0");
        assert_eq!(cfg.rst_line, 5);
        assert_eq!(cfg.dc_line, 6);
        assert_eq!(cfg.busy_line, 7);
        assert_eq!(cfg.pwr_line, None);
    }

    /// The safety fix: a missing PWR var with no explicit opt-out must be a
    /// hard error, not a silent `None` that would run a carrier-board panel
    /// unpowered.
    #[test]
    fn from_getter_errors_when_pwr_is_neither_set_nor_explicitly_opted_out() {
        let get = env_from(&[
            ("CORKBOARD_PANEL_GPIOCHIP", "/dev/gpiochip0"),
            ("CORKBOARD_PANEL_RST_LINE", "5"),
            ("CORKBOARD_PANEL_DC_LINE", "6"),
            ("CORKBOARD_PANEL_BUSY_LINE", "7"),
            // neither CORKBOARD_PANEL_PWR_LINE nor CORKBOARD_PANEL_NO_PWR set
        ]);

        assert!(PanelConfig::from_getter(&get).is_err());
    }

    /// The opt-out must be exactly `1`, so a stray `NO_PWR=0` (operator meaning
    /// "no, I do have a gate") can't be misread as "opt out of PWR".
    #[test]
    fn from_getter_errors_when_no_pwr_is_not_exactly_one() {
        let get = env_from(&[
            ("CORKBOARD_PANEL_GPIOCHIP", "/dev/gpiochip0"),
            ("CORKBOARD_PANEL_RST_LINE", "5"),
            ("CORKBOARD_PANEL_DC_LINE", "6"),
            ("CORKBOARD_PANEL_BUSY_LINE", "7"),
            ("CORKBOARD_PANEL_NO_PWR", "0"),
        ]);

        assert!(PanelConfig::from_getter(&get).is_err());
    }

    /// A real PWR line takes precedence over the opt-out path entirely.
    #[test]
    fn from_getter_uses_pwr_line_when_set_even_without_opt_out() {
        let get = env_from(&[
            ("CORKBOARD_PANEL_GPIOCHIP", "/dev/gpiochip0"),
            ("CORKBOARD_PANEL_RST_LINE", "5"),
            ("CORKBOARD_PANEL_DC_LINE", "6"),
            ("CORKBOARD_PANEL_BUSY_LINE", "7"),
            ("CORKBOARD_PANEL_PWR_LINE", "3"),
        ]);

        assert_eq!(PanelConfig::from_getter(&get).unwrap().pwr_line, Some(3));
    }

    #[test]
    fn from_getter_errors_when_a_required_pin_is_missing() {
        let get = env_from(&[
            ("CORKBOARD_PANEL_GPIOCHIP", "/dev/gpiochip0"),
            ("CORKBOARD_PANEL_RST_LINE", "5"),
            ("CORKBOARD_PANEL_DC_LINE", "6"),
            // CORKBOARD_PANEL_BUSY_LINE deliberately missing.
        ]);

        assert!(PanelConfig::from_getter(&get).is_err());
    }

    #[test]
    fn from_getter_errors_when_a_pin_does_not_parse_as_u32() {
        let get = env_from(&[
            ("CORKBOARD_PANEL_GPIOCHIP", "/dev/gpiochip0"),
            ("CORKBOARD_PANEL_RST_LINE", "not-a-number"),
            ("CORKBOARD_PANEL_DC_LINE", "6"),
            ("CORKBOARD_PANEL_BUSY_LINE", "7"),
        ]);

        assert!(PanelConfig::from_getter(&get).is_err());
    }

    /// Regression test: a set-but-non-Unicode env var must be a hard error,
    /// not silently treated the same as "unset" (a real bug an earlier review
    /// caught — `optional_line` used to match on any `Err(_)`).
    #[test]
    fn from_getter_errors_on_non_unicode_pwr_line_instead_of_treating_it_as_unset() {
        fn get(name: &str) -> Result<String, std::env::VarError> {
            if name == "CORKBOARD_PANEL_PWR_LINE" {
                return Err(std::env::VarError::NotUnicode(std::ffi::OsString::from(
                    "bad",
                )));
            }
            env_from(REQUIRED_PINS)(name)
        }

        assert!(PanelConfig::from_getter(&get).is_err());
    }

    /// Fake GPIO line: shared state behind `Arc<Mutex<_>>` so a clone kept by
    /// the test can observe what the code under test did after handing the
    /// other clone off into a `Box<dyn GpioLine>`.
    #[derive(Clone, Default)]
    struct FakeGpio(Arc<Mutex<FakeGpioState>>);

    #[derive(Default)]
    struct FakeGpioState {
        value: u8,
        fail: bool,
    }

    impl FakeGpio {
        fn failing() -> Self {
            let f = FakeGpio::default();
            f.0.lock().unwrap().fail = true;
            f
        }

        /// A busy line reading idle (1) — the normal "not stuck" case.
        fn idle() -> Self {
            let f = FakeGpio::default();
            f.set_value(1).unwrap();
            f
        }
    }

    impl GpioLine for FakeGpio {
        fn set_value(&self, value: u8) -> anyhow::Result<()> {
            self.0.lock().unwrap().value = value;
            Ok(())
        }

        fn get_value(&self) -> anyhow::Result<u8> {
            let state = self.0.lock().unwrap();
            if state.fail {
                anyhow::bail!("fake gpio read failure");
            }
            Ok(state.value)
        }
    }

    #[derive(Clone, Default)]
    struct FakeSpi(Arc<Mutex<FakeSpiState>>);

    #[derive(Default)]
    struct FakeSpiState {
        writes: Vec<Vec<u8>>,
        fail: bool,
        /// If set, only the write whose first byte equals this fails (models
        /// an SPI error on one specific command, e.g. DISPLAY_REFRESH); every
        /// other write still succeeds and is recorded.
        fail_on_command: Option<u8>,
        /// If set, any multi-byte write fails (models the bulk frame transfer
        /// failing — the frame is the only write longer than one byte).
        fail_on_bulk: bool,
    }

    impl FakeSpi {
        fn failing() -> Self {
            let s = FakeSpi::default();
            s.0.lock().unwrap().fail = true;
            s
        }

        fn failing_on_command(cmd: u8) -> Self {
            let s = FakeSpi::default();
            s.0.lock().unwrap().fail_on_command = Some(cmd);
            s
        }

        fn failing_on_bulk() -> Self {
            let s = FakeSpi::default();
            s.0.lock().unwrap().fail_on_bulk = true;
            s
        }

        fn writes(&self) -> Vec<Vec<u8>> {
            self.0.lock().unwrap().writes.clone()
        }
    }

    impl SpiBus for FakeSpi {
        fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
            let mut state = self.0.lock().unwrap();
            if state.fail
                || state.fail_on_command == buf.first().copied()
                || (state.fail_on_bulk && buf.len() > 1)
            {
                return Err(std::io::Error::other("fake spi write failure"));
            }
            state.writes.push(buf.to_vec());
            Ok(())
        }
    }

    fn fake_inner(spi: FakeSpi, busy: impl GpioLine + 'static) -> Inner {
        fake_inner_with_dc(spi, busy, FakeGpio::default())
    }

    /// Like `fake_inner`, but with a PWR gate the test keeps a handle on, to
    /// assert `power_up`/`emergency_power_off` drive it.
    fn fake_inner_with_pwr(
        spi: FakeSpi,
        busy: impl GpioLine + 'static,
        pwr: impl GpioLine + 'static,
    ) -> Inner {
        Inner {
            spi: Box::new(spi),
            rst: Box::new(FakeGpio::default()),
            dc: Box::new(FakeGpio::default()),
            busy: Box::new(busy),
            pwr: Some(Box::new(pwr)),
        }
    }

    /// Like `fake_inner`, but lets a test keep its own handle on the DC line
    /// (e.g. to assert it was left in data mode).
    fn fake_inner_with_dc(
        spi: FakeSpi,
        busy: impl GpioLine + 'static,
        dc: impl GpioLine + 'static,
    ) -> Inner {
        Inner {
            spi: Box::new(spi),
            rst: Box::new(FakeGpio::default()),
            dc: Box::new(dc),
            busy: Box::new(busy),
            pwr: None,
        }
    }

    fn fake_panel(inner: Inner) -> Panel {
        Panel {
            inner: Mutex::new(inner),
        }
    }

    /// A well-formed WIDTHxHEIGHT PNG — content doesn't matter to the tests
    /// that use it, only that it decodes and packs. (It comes out black:
    /// `RgbImage::new` zero-fills.)
    fn valid_frame_png() -> Vec<u8> {
        let img = image::RgbImage::new(WIDTH as u32, HEIGHT as u32);
        let mut png = Vec::new();
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        png
    }

    /// Command byte of each recorded SPI write (its first byte).
    fn command_bytes(spi: &FakeSpi) -> Vec<u8> {
        spi.writes().into_iter().map(|w| w[0]).collect()
    }

    #[test]
    fn wait_busy_succeeds_when_line_is_already_idle() {
        assert!(wait_busy(&FakeGpio::idle(), Duration::from_secs(1)).is_ok());
    }

    #[test]
    fn wait_busy_times_out_when_line_never_goes_idle() {
        let busy = FakeGpio::default(); // 0 = busy, forever
        assert!(wait_busy(&busy, Duration::from_millis(20)).is_err());
    }

    #[test]
    fn wait_busy_propagates_gpio_read_errors() {
        assert!(wait_busy(&FakeGpio::failing(), Duration::from_secs(1)).is_err());
    }

    #[test]
    fn emergency_power_off_sends_power_off_command_and_data() {
        let spi = FakeSpi::default();
        let spi_probe = spi.clone();
        let mut inner = fake_inner(spi, FakeGpio::idle());

        emergency_power_off(&mut inner);

        assert_eq!(spi_probe.writes(), vec![vec![0x02], vec![0x00]]);
    }

    #[test]
    fn emergency_power_off_ignores_spi_errors() {
        let mut inner = fake_inner(FakeSpi::failing(), FakeGpio::idle());
        // Must not panic even though every write fails — it's the best-effort
        // path run after a render already failed.
        emergency_power_off(&mut inner);
    }

    #[test]
    fn turn_on_display_sends_power_on_refresh_power_off() {
        let spi = FakeSpi::default();
        let spi_probe = spi.clone();
        let mut inner = fake_inner(spi, FakeGpio::idle());

        turn_on_display(&mut inner).unwrap();

        assert_eq!(
            spi_probe.writes(),
            vec![vec![0x04], vec![0x12], vec![0x00], vec![0x02], vec![0x00]]
        );
    }

    /// Locks the full init register sequence byte-for-byte against the
    /// Waveshare reference, ending with POWER_ON (0x04) and no trailing
    /// power-off — the render's display step powers off, matching the
    /// reference's per-image init -> display flow.
    #[test]
    fn init_sends_the_full_register_sequence_ending_in_power_on() {
        let spi = FakeSpi::default();
        let spi_probe = spi.clone();
        let mut inner = fake_inner(spi, FakeGpio::idle());

        init(&mut inner).unwrap();

        #[rustfmt::skip]
        let expected = vec![
            0xAA, 0x49, 0x55, 0x20, 0x08, 0x09, 0x18,
            0x01, 0x3F,
            0x00, 0x5F, 0x69,
            0x03, 0x00, 0x54, 0x00, 0x44,
            0x05, 0x40, 0x1F, 0x1F, 0x2C,
            0x06, 0x6F, 0x1F, 0x17, 0x49,
            0x08, 0x6F, 0x1F, 0x1F, 0x22,
            0x30, 0x03,
            0x50, 0x3F,
            0x60, 0x02, 0x00,
            0x61, 0x03, 0x20, 0x01, 0xE0,
            0x84, 0x01,
            0xE3, 0x2F,
            0x04,
        ];
        assert_eq!(command_bytes(&spi_probe), expected);
    }

    /// The happy path, asserted as the full control-byte skeleton so a missing
    /// step (a dropped `0x10`, a skipped frame, a missing POWER_OFF) can't slip
    /// through. Every command and single data byte is a 1-byte write; the frame
    /// is the only multi-byte (bulk) write, so splitting on length recovers the
    /// exact control sequence AND proves the whole frame was sent.
    #[test]
    fn show_runs_the_full_one_shot_command_skeleton() {
        let spi = FakeSpi::default();
        let spi_probe = spi.clone();
        let panel = fake_panel(fake_inner(spi, FakeGpio::idle()));

        panel.show(&valid_frame_png()).unwrap();

        let writes = spi_probe.writes();
        let control: Vec<u8> = writes
            .iter()
            .filter(|w| w.len() == 1)
            .map(|w| w[0])
            .collect();
        let frame: Vec<u8> = writes
            .iter()
            .filter(|w| w.len() > 1)
            .flatten()
            .copied()
            .collect();

        #[rustfmt::skip]
        let expected_control = vec![
            // init: full register config ending in POWER_ON (0x04)
            0xAA, 0x49, 0x55, 0x20, 0x08, 0x09, 0x18,
            0x01, 0x3F,
            0x00, 0x5F, 0x69,
            0x03, 0x00, 0x54, 0x00, 0x44,
            0x05, 0x40, 0x1F, 0x1F, 0x2C,
            0x06, 0x6F, 0x1F, 0x17, 0x49,
            0x08, 0x6F, 0x1F, 0x1F, 0x22,
            0x30, 0x03,
            0x50, 0x3F,
            0x60, 0x02, 0x00,
            0x61, 0x03, 0x20, 0x01, 0xE0,
            0x84, 0x01,
            0xE3, 0x2F,
            0x04,
            0x10,                          // DATA_START_TRANSMISSION (frame follows)
            0x04, 0x12, 0x00, 0x02, 0x00,  // POWER_ON, REFRESH, POWER_OFF
            0x07, 0xA5,                    // DEEP_SLEEP
        ];
        assert_eq!(control, expected_control);
        assert_eq!(frame.len(), WIDTH * HEIGHT / 2, "full frame sent");
        assert!(
            frame.iter().all(|&b| b == 0),
            "black test frame packs to zeroes"
        );
    }

    /// On any render failure with the rail potentially live, `show` must make
    /// a best-effort POWER_OFF and must NOT proceed to deep sleep. Modelled by
    /// an SPI error on the DISPLAY_REFRESH command, after POWER_ON succeeded.
    #[test]
    fn show_powers_off_and_does_not_deep_sleep_when_a_render_step_fails() {
        let spi = FakeSpi::failing_on_command(0x12); // DISPLAY_REFRESH fails
        let spi_probe = spi.clone();
        let panel = fake_panel(fake_inner(spi, FakeGpio::idle()));

        let result = panel.show(&valid_frame_png());

        assert!(result.is_err());
        let writes = spi_probe.writes();
        assert_eq!(
            &writes[writes.len() - 2..],
            [vec![0x02u8], vec![0x00u8]],
            "ends in a best-effort POWER_OFF"
        );
        assert!(
            !command_bytes(&spi_probe).contains(&0x07),
            "must not deep-sleep after a failure"
        );
    }

    /// Same guarantee, but for a failure on the POWER_ON write itself — the
    /// very command that makes the rail live. The top-level catch in `show`
    /// still runs a best-effort POWER_OFF and does not deep-sleep.
    #[test]
    fn show_powers_off_when_the_power_on_write_itself_fails() {
        let spi = FakeSpi::failing_on_command(0x04); // POWER_ON fails
        let spi_probe = spi.clone();
        let panel = fake_panel(fake_inner(spi, FakeGpio::idle()));

        let result = panel.show(&valid_frame_png());

        assert!(result.is_err());
        let writes = spi_probe.writes();
        assert_eq!(
            &writes[writes.len() - 2..],
            [vec![0x02u8], vec![0x00u8]],
            "a POWER_ON send failure still ends in a best-effort POWER_OFF"
        );
        assert!(!command_bytes(&spi_probe).contains(&0x07));
    }

    /// And for a failure during the bulk frame transfer (after POWER_ON made
    /// the rail live) — same guarantee: best-effort POWER_OFF, no deep sleep.
    #[test]
    fn show_powers_off_when_the_frame_write_fails() {
        let spi = FakeSpi::failing_on_bulk(); // the bulk frame transfer fails
        let spi_probe = spi.clone();
        let panel = fake_panel(fake_inner(spi, FakeGpio::idle()));

        let result = panel.show(&valid_frame_png());

        assert!(result.is_err());
        let writes = spi_probe.writes();
        assert_eq!(
            &writes[writes.len() - 2..],
            [vec![0x02u8], vec![0x00u8]],
            "ends in a best-effort POWER_OFF"
        );
        assert!(!command_bytes(&spi_probe).contains(&0x07));
    }

    /// A render asserts the PWR gate at the top and, on success, leaves it
    /// high (the panel sits in DEEP_SLEEP with power still on between renders).
    #[test]
    fn show_asserts_pwr_and_leaves_it_high_on_success() {
        let pwr = FakeGpio::default(); // starts low, as if never powered
        let panel = fake_panel(fake_inner_with_pwr(
            FakeSpi::default(),
            FakeGpio::idle(),
            pwr.clone(),
        ));

        panel.show(&valid_frame_png()).unwrap();

        assert_eq!(pwr.get_value().unwrap(), 1, "PWR left high on success");
    }

    /// On failure, the PWR gate is dropped low as a hard rail cut — the part
    /// of the safety net that still works when SPI itself is what failed.
    #[test]
    fn show_drops_the_pwr_gate_on_failure() {
        let pwr = FakeGpio::default();
        pwr.set_value(1).unwrap(); // starts high, so a drop is observable
        let panel = fake_panel(fake_inner_with_pwr(
            FakeSpi::failing_on_command(0x12), // DISPLAY_REFRESH fails
            FakeGpio::idle(),
            pwr.clone(),
        ));

        assert!(panel.show(&valid_frame_png()).is_err());

        assert_eq!(pwr.get_value().unwrap(), 0, "PWR hard-cut low on failure");
    }

    #[test]
    fn power_up_reasserts_pwr_after_a_prior_failure_dropped_it() {
        let pwr = FakeGpio::default(); // low, as a prior emergency left it
        let mut inner = fake_inner_with_pwr(FakeSpi::default(), FakeGpio::idle(), pwr.clone());

        power_up(&mut inner).unwrap();

        assert_eq!(pwr.get_value().unwrap(), 1);
    }

    /// The full hard-cut: RST, DC, and PWR are all driven low (so they don't
    /// sit high against the about-to-be-unpowered panel). Guards against a
    /// regression that drops any of them. (Strict low-before-PWR *ordering*
    /// isn't asserted — the fakes are independent, so only final values are
    /// observable — but the values are the regression that matters.)
    #[test]
    fn emergency_power_off_drives_rst_dc_and_pwr_low() {
        let rst = FakeGpio::default();
        let dc = FakeGpio::default();
        let pwr = FakeGpio::default();
        for line in [&rst, &dc, &pwr] {
            line.set_value(1).unwrap(); // start high, so a drop is observable
        }
        let mut inner = Inner {
            spi: Box::new(FakeSpi::default()),
            rst: Box::new(rst.clone()),
            dc: Box::new(dc.clone()),
            busy: Box::new(FakeGpio::idle()),
            pwr: Some(Box::new(pwr.clone())),
        };

        emergency_power_off(&mut inner);

        assert_eq!(rst.get_value().unwrap(), 0, "RST driven low");
        assert_eq!(dc.get_value().unwrap(), 0, "DC driven low");
        assert_eq!(pwr.get_value().unwrap(), 0, "PWR dropped low");
    }

    #[test]
    fn send_data_bulk_chunks_large_buffers_without_dropping_or_reordering_bytes() {
        let spi = FakeSpi::default();
        let spi_probe = spi.clone();
        let dc = FakeGpio::default();
        let mut inner = fake_inner_with_dc(spi, FakeGpio::default(), dc.clone());
        let data: Vec<u8> = (0..(SPI_CHUNK * 2 + 100))
            .map(|i| (i % 256) as u8)
            .collect();

        send_data_bulk(&mut inner, &data).unwrap();

        let writes = spi_probe.writes();
        assert!(
            writes.len() > 1,
            "expected the buffer to be split into chunks"
        );
        assert!(writes.iter().all(|c| c.len() <= SPI_CHUNK));
        assert_eq!(writes.concat(), data);
        assert_eq!(dc.get_value().unwrap(), 1, "DC must be high (data mode)");
    }

    #[test]
    fn packs_two_pixels_per_byte_high_nibble_first() {
        let mut img = image::RgbImage::new(WIDTH as u32, HEIGHT as u32);
        // Pixel 0 = black (0x0), pixel 1 = white (0x1) -> byte 0x01.
        img.put_pixel(0, 0, image::Rgb(Colour::Black.rgb()));
        img.put_pixel(1, 0, image::Rgb(Colour::White.rgb()));
        // Pixel 2 = red (0x3), pixel 3 = green (0x6) -> byte 0x36.
        img.put_pixel(2, 0, image::Rgb(Colour::Red.rgb()));
        img.put_pixel(3, 0, image::Rgb(Colour::Green.rgb()));

        let packed = pack(&img).unwrap();
        assert_eq!(packed[0], 0x01);
        assert_eq!(packed[1], 0x36);
        assert_eq!(packed.len(), WIDTH * HEIGHT / 2);
    }

    #[test]
    fn rejects_wrong_dimensions() {
        let img = image::RgbImage::new(10, 10);
        assert!(pack(&img).is_err());
    }

    #[test]
    fn nearest_colour_matches_exact_palette_values() {
        for c in Colour::ALL {
            assert_eq!(nearest_colour(c.rgb()).panel_code(), c.panel_code());
        }
    }
}
