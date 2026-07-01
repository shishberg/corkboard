#![allow(dead_code)]

//! SPI/GPIO driver for the Waveshare 7.3" E6 (Spectra 6) panel, 800x480,
//! 6-colour. A byte-for-byte port of Waveshare's own Python `epd7in3e` driver
//! (verified register-identical between the PhotoPainter kit's copy and the
//! canonical `waveshareteam/e-Paper` repo). The C demo agrees on everything
//! except one register write in `turn_on_display` — see the note there.
//!
//! **One-shot per refresh, mirroring the reference's whole host lifecycle.**
//! `Panel` stores only config. Every `show()` opens a fresh host session
//! (assert PWR, open SPI, claim RST/DC/BUSY), runs one refresh (reset, init,
//! frame, refresh, deep sleep), then releases *everything* — matching the
//! reference's `module_init()` / `module_exit()` bracketing of each image.
//! Nothing is carried between refreshes: no held handles, no cross-render
//! state. Between refreshes the panel is unpowered.
//!
//! **Teardown is guaranteed on every exit path.** The opened resources live in
//! a `Session` (and, mid-open, an `OpeningGuard`) whose `Drop` runs teardown,
//! so any early return — a `?` on an SPI/GPIO error, a BUSY timeout — still
//! releases the hardware. Teardown drives RST/DC low, closes SPI, then drops
//! the PWR gate (the reference's module_exit order: bus released *before* power
//! is cut, so no line is left driving an unpowered panel). On a failed refresh
//! it also best-effort sends POWER_OFF as an emergency high-voltage-rail kill;
//! on a clean refresh the refresh already powered off, so that's skipped.
//!
//! Real hardware access (`spidev`/`gpio-cdev`, which need Linux ioctls) is
//! gated to `target_os = "linux"`; everything else — the register sequence and
//! timing behind the `HostOpener`/`SpiBus`/`GpioLine` seams — compiles and is
//! unit-tested with fakes on any host, so the hardware-safety logic needs no
//! real hardware or Linux box to verify.
//!
//! See `.mex/patterns/deploy-to-orange-pi.md` for what's still unverified: the
//! GPIO chip/line numbers are Raspberry Pi BCM numbers in Waveshare's demo and
//! do NOT carry over to the Orange Pi's Allwinner H618 numbering, even though
//! the 40-pin header is physically pin-compatible. Determine the real numbers
//! with `gpioinfo` once the panel is wired — there are no hardcoded pin
//! defaults here on purpose, so a wrong guess can't silently drive the wrong
//! line.

use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
#[cfg(target_os = "linux")]
use spidev::{SpiModeFlags, Spidev, SpidevOptions};

use crate::display::Display;
use crate::document::Colour;

const WIDTH: usize = 800;
const HEIGHT: usize = 480;
const SPI_CHUNK: usize = 4096;
const BUSY_TIMEOUT: Duration = Duration::from_secs(120);

/// Time to let the panel's power rail stabilise after asserting the PWR gate,
/// before anything drives the panel. The reference doesn't wait here; this is
/// a small safety margin so the reset pulse never lands on a still-settling
/// rail. Skipped entirely on a bare HAT with no gate.
const PWR_SETTLE: Duration = Duration::from_millis(10);

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

trait SpiBus: Send {
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()>;
    fn close(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl SpiBus for Spidev {
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        std::io::Write::write_all(self, buf)
    }
}

trait HostBackend {
    fn request_output(
        &mut self,
        line: u32,
        initial: u8,
        consumer: &'static str,
    ) -> anyhow::Result<Box<dyn GpioLine>>;
    fn request_input(
        &mut self,
        line: u32,
        consumer: &'static str,
    ) -> anyhow::Result<Box<dyn GpioLine>>;
    fn open_spi(&mut self, path: &str) -> anyhow::Result<Box<dyn SpiBus>>;
}

trait HostOpener: Send + Sync {
    fn open(&self, cfg: &PanelConfig) -> anyhow::Result<HostResources>;
}

struct RealOpener;

impl HostOpener for RealOpener {
    fn open(&self, cfg: &PanelConfig) -> anyhow::Result<HostResources> {
        open_real_host(cfg)
    }
}

#[cfg(target_os = "linux")]
struct LinuxBackend {
    chip: Chip,
}

#[cfg(target_os = "linux")]
impl HostBackend for LinuxBackend {
    fn request_output(
        &mut self,
        line: u32,
        initial: u8,
        consumer: &'static str,
    ) -> anyhow::Result<Box<dyn GpioLine>> {
        Ok(Box::new(self.chip.get_line(line)?.request(
            LineRequestFlags::OUTPUT,
            initial,
            consumer,
        )?))
    }

    fn request_input(
        &mut self,
        line: u32,
        consumer: &'static str,
    ) -> anyhow::Result<Box<dyn GpioLine>> {
        Ok(Box::new(self.chip.get_line(line)?.request(
            LineRequestFlags::INPUT,
            0,
            consumer,
        )?))
    }

    fn open_spi(&mut self, path: &str) -> anyhow::Result<Box<dyn SpiBus>> {
        let mut spi = Spidev::open(path)?;
        spi.configure(
            &SpidevOptions::new()
                .bits_per_word(8)
                .max_speed_hz(4_000_000)
                .mode(SpiModeFlags::SPI_MODE_0)
                .build(),
        )?;
        Ok(Box::new(spi))
    }
}

#[cfg(target_os = "linux")]
fn open_real_host(cfg: &PanelConfig) -> anyhow::Result<HostResources> {
    let mut backend = LinuxBackend {
        chip: Chip::new(&cfg.gpiochip_path)?,
    };
    open_resources(cfg, &mut backend)
}

#[cfg(not(target_os = "linux"))]
fn open_real_host(_cfg: &PanelConfig) -> anyhow::Result<HostResources> {
    anyhow::bail!("real e-paper panel access is only available on Linux")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelConfig {
    pub spi_path: String,
    pub gpiochip_path: String,
    pub rst_line: u32,
    pub dc_line: u32,
    pub busy_line: u32,
    pub pwr_line: Option<u32>,
}

impl PanelConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Self::from_getter(&|name| std::env::var(name))
    }

    fn from_getter(
        get: &dyn Fn(&str) -> Result<String, std::env::VarError>,
    ) -> anyhow::Result<Self> {
        fn required(
            get: &dyn Fn(&str) -> Result<String, std::env::VarError>,
            name: &str,
        ) -> anyhow::Result<String> {
            match get(name) {
                Ok(value) => Ok(value),
                Err(std::env::VarError::NotPresent) => Err(anyhow::anyhow!("{name} is not set")),
                Err(e @ std::env::VarError::NotUnicode(_)) => Err(anyhow::anyhow!("{name}: {e}")),
            }
        }

        fn optional_with_default(
            get: &dyn Fn(&str) -> Result<String, std::env::VarError>,
            name: &str,
            default: &str,
        ) -> anyhow::Result<String> {
            match get(name) {
                Ok(value) => Ok(value),
                Err(std::env::VarError::NotPresent) => Ok(default.to_string()),
                Err(e @ std::env::VarError::NotUnicode(_)) => Err(anyhow::anyhow!("{name}: {e}")),
            }
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

        fn pwr_line(
            get: &dyn Fn(&str) -> Result<String, std::env::VarError>,
        ) -> anyhow::Result<Option<u32>> {
            match get("CORKBOARD_PANEL_PWR_LINE") {
                Ok(value) => Ok(Some(parse_line("CORKBOARD_PANEL_PWR_LINE", value)?)),
                Err(e @ std::env::VarError::NotUnicode(_)) => {
                    Err(anyhow::anyhow!("CORKBOARD_PANEL_PWR_LINE: {e}"))
                }
                Err(std::env::VarError::NotPresent) => match get("CORKBOARD_PANEL_NO_PWR") {
                    Ok(value) if value == "1" => Ok(None),
                    Ok(value) => Err(anyhow::anyhow!(
                        "CORKBOARD_PANEL_NO_PWR must be \"1\" if set, got {value:?}"
                    )),
                    Err(e @ std::env::VarError::NotUnicode(_)) => {
                        Err(anyhow::anyhow!("CORKBOARD_PANEL_NO_PWR: {e}"))
                    }
                    Err(std::env::VarError::NotPresent) => Err(anyhow::anyhow!(
                        "CORKBOARD_PANEL_PWR_LINE is not set; set it to the panel power line, \
                         or set CORKBOARD_PANEL_NO_PWR=1 for a bare HAT with no power gate"
                    )),
                },
            }
        }

        Ok(PanelConfig {
            spi_path: optional_with_default(get, "CORKBOARD_PANEL_SPI", "/dev/spidev0.0")?,
            gpiochip_path: required(get, "CORKBOARD_PANEL_GPIOCHIP")?,
            rst_line: required_line(get, "CORKBOARD_PANEL_RST_LINE")?,
            dc_line: required_line(get, "CORKBOARD_PANEL_DC_LINE")?,
            busy_line: required_line(get, "CORKBOARD_PANEL_BUSY_LINE")?,
            pwr_line: pwr_line(get)?,
        })
    }
}

pub struct Panel {
    config: PanelConfig,
}

impl Panel {
    pub fn new(config: PanelConfig) -> Self {
        Self { config }
    }

    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self::new(PanelConfig::from_env()?))
    }

    fn show_with_opener(&self, png: &[u8], opener: &dyn HostOpener) -> anyhow::Result<()> {
        let img = image::load_from_memory(png)?.to_rgb8();
        let frame = pack(&img)?;
        let resources = opener.open(&self.config)?;
        let mut session = Session::new(resources);
        run_refresh(&mut session, &frame)
    }
}

impl Display for Panel {
    fn show(&self, png: &[u8]) -> anyhow::Result<()> {
        self.show_with_opener(png, &RealOpener)
    }
}

struct HostResources {
    spi: Option<Box<dyn SpiBus>>,
    rst: Option<Box<dyn GpioLine>>,
    dc: Option<Box<dyn GpioLine>>,
    busy: Option<Box<dyn GpioLine>>,
    pwr: Option<Box<dyn GpioLine>>,
}

impl HostResources {
    fn empty() -> Self {
        Self {
            spi: None,
            rst: None,
            dc: None,
            busy: None,
            pwr: None,
        }
    }
}

struct OpeningGuard {
    resources: HostResources,
}

impl OpeningGuard {
    fn new() -> Self {
        Self {
            resources: HostResources::empty(),
        }
    }

    fn finish(mut self) -> HostResources {
        HostResources {
            spi: self.resources.spi.take(),
            rst: self.resources.rst.take(),
            dc: self.resources.dc.take(),
            busy: self.resources.busy.take(),
            pwr: self.resources.pwr.take(),
        }
    }
}

impl Drop for OpeningGuard {
    fn drop(&mut self) {
        // A failure during open() is before POWER_ON is ever sent, so there's
        // no live rail to kill — just release whatever was claimed. (After a
        // successful open, `finish()` has emptied this, so this is a no-op.)
        teardown_resources(&mut self.resources, false);
    }
}

fn open_resources(
    cfg: &PanelConfig,
    backend: &mut dyn HostBackend,
) -> anyhow::Result<HostResources> {
    let mut guard = OpeningGuard::new();

    // PWR first, before anything is driven into the panel — matching the
    // reference's module_init (PWR high, then SPI open). Let the rail settle
    // before the reset pulse follows in `init`.
    if let Some(line) = cfg.pwr_line {
        guard.resources.pwr = Some(backend.request_output(line, 1, "corkboard-panel-pwr")?);
        delay(PWR_SETTLE);
    }

    guard.resources.spi = Some(backend.open_spi(&cfg.spi_path)?);
    guard.resources.rst = Some(backend.request_output(cfg.rst_line, 0, "corkboard-panel-rst")?);
    guard.resources.dc = Some(backend.request_output(cfg.dc_line, 0, "corkboard-panel-dc")?);
    guard.resources.busy = Some(backend.request_input(cfg.busy_line, "corkboard-panel-busy")?);

    Ok(guard.finish())
}

struct Session {
    resources: HostResources,
    /// Set true only once a refresh has run to completion (through DEEP_SLEEP).
    /// On the clean path `turn_on_display` already sent POWER_OFF, so teardown
    /// must NOT re-send it; when this is false (any failure), teardown does a
    /// best-effort POWER_OFF as the emergency high-voltage-rail kill.
    clean: bool,
}

impl Session {
    fn new(resources: HostResources) -> Self {
        Self {
            resources,
            clean: false,
        }
    }

    fn send_command(&mut self, command: u8) -> anyhow::Result<()> {
        send_command(&mut self.resources, command)
    }

    fn send_data(&mut self, data: u8) -> anyhow::Result<()> {
        send_data(&mut self.resources, data)
    }

    fn send_data_bulk(&mut self, data: &[u8]) -> anyhow::Result<()> {
        send_data_bulk(&mut self.resources, data)
    }

    fn busy(&self) -> anyhow::Result<&dyn GpioLine> {
        self.resources
            .busy
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("BUSY line is not open"))
    }

    fn rst(&self) -> anyhow::Result<&dyn GpioLine> {
        self.resources
            .rst
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("RST line is not open"))
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // Emergency POWER_OFF only when the refresh didn't finish cleanly.
        teardown_resources(&mut self.resources, !self.clean);
    }
}

/// Best-effort release of the host resources, run by both guards' `Drop` so it
/// happens on every exit path. `send_power_off` requests the emergency HV kill
/// (POWER_OFF 0x02,0x00) — used only when a refresh did not complete cleanly;
/// on the clean path `turn_on_display` already powered off, so re-sending it
/// after DEEP_SLEEP would just poke a sleeping controller (the reference's
/// module_exit sends no command here). RST/DC are driven low, then SPI is
/// closed, then PWR is dropped — the reference's module_exit order (SPI
/// released before power is cut), so no line is left driving an unpowered
/// panel. Every step ignores errors; nothing panics.
fn teardown_resources(resources: &mut HostResources, send_power_off: bool) {
    if send_power_off {
        let _ = send_command(resources, 0x02);
        let _ = send_data(resources, 0x00);
    }

    if let Some(rst) = resources.rst.as_ref() {
        let _ = rst.set_value(0);
    }
    if let Some(dc) = resources.dc.as_ref() {
        let _ = dc.set_value(0);
    }

    if let Some(mut spi) = resources.spi.take() {
        let _ = spi.close();
        drop(spi);
    }

    if let Some(pwr) = resources.pwr.as_ref() {
        let _ = pwr.set_value(0);
    }
}

fn run_refresh(session: &mut Session, frame: &[u8]) -> anyhow::Result<()> {
    init(session)?;
    session.send_command(0x10)?;
    session.send_data_bulk(frame)?;
    turn_on_display(session)?;
    deep_sleep(session)?;
    // Reached only if every step above succeeded: the panel is powered off
    // (by turn_on_display) and asleep, so teardown skips the emergency POWER_OFF.
    session.clean = true;
    Ok(())
}

fn init(session: &mut Session) -> anyhow::Result<()> {
    reset(session.rst()?)?;
    wait_busy(session.busy()?, BUSY_TIMEOUT)?;
    delay(Duration::from_millis(30));

    session.send_command(0xAA)?;
    for byte in [0x49, 0x55, 0x20, 0x08, 0x09, 0x18] {
        session.send_data(byte)?;
    }

    session.send_command(0x01)?;
    session.send_data(0x3F)?;

    session.send_command(0x00)?;
    session.send_data(0x5F)?;
    session.send_data(0x69)?;

    session.send_command(0x03)?;
    for byte in [0x00, 0x54, 0x00, 0x44] {
        session.send_data(byte)?;
    }

    session.send_command(0x05)?;
    for byte in [0x40, 0x1F, 0x1F, 0x2C] {
        session.send_data(byte)?;
    }

    session.send_command(0x06)?;
    for byte in [0x6F, 0x1F, 0x17, 0x49] {
        session.send_data(byte)?;
    }

    session.send_command(0x08)?;
    for byte in [0x6F, 0x1F, 0x1F, 0x22] {
        session.send_data(byte)?;
    }

    session.send_command(0x30)?;
    session.send_data(0x03)?;

    session.send_command(0x50)?;
    session.send_data(0x3F)?;

    session.send_command(0x60)?;
    session.send_data(0x02)?;
    session.send_data(0x00)?;

    session.send_command(0x61)?;
    for byte in [0x03, 0x20, 0x01, 0xE0] {
        session.send_data(byte)?;
    }

    session.send_command(0x84)?;
    session.send_data(0x01)?;

    session.send_command(0xE3)?;
    session.send_data(0x2F)?;

    session.send_command(0x04)?;
    wait_busy(session.busy()?, BUSY_TIMEOUT)?;
    Ok(())
}

fn turn_on_display(session: &mut Session) -> anyhow::Result<()> {
    session.send_command(0x04)?;
    wait_busy(session.busy()?, BUSY_TIMEOUT)?;

    // Python does not repeat command 0x06 here. The C demo does. If a real
    // panel will not refresh, this is the first place to compare.
    session.send_command(0x12)?;
    session.send_data(0x00)?;
    wait_busy(session.busy()?, BUSY_TIMEOUT)?;

    session.send_command(0x02)?;
    session.send_data(0x00)?;
    wait_busy(session.busy()?, BUSY_TIMEOUT)?;
    Ok(())
}

fn deep_sleep(session: &mut Session) -> anyhow::Result<()> {
    session.send_command(0x07)?;
    session.send_data(0xA5)?;
    delay(Duration::from_secs(2));
    Ok(())
}

fn reset(rst: &dyn GpioLine) -> anyhow::Result<()> {
    rst.set_value(1)?;
    delay(Duration::from_millis(20));
    rst.set_value(0)?;
    delay(Duration::from_millis(2));
    rst.set_value(1)?;
    delay(Duration::from_millis(20));
    Ok(())
}

fn wait_busy(busy: &dyn GpioLine, timeout: Duration) -> anyhow::Result<()> {
    let deadline = Instant::now() + timeout;
    while busy.get_value()? == 0 {
        if Instant::now() >= deadline {
            anyhow::bail!("panel BUSY line never went idle within {timeout:?}");
        }
        delay(Duration::from_millis(5));
    }
    Ok(())
}

fn delay(duration: Duration) {
    #[cfg(not(test))]
    std::thread::sleep(duration);
    #[cfg(test)]
    let _ = duration;
}

fn send_command(resources: &mut HostResources, command: u8) -> anyhow::Result<()> {
    let dc = resources
        .dc
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("DC line is not open"))?;
    dc.set_value(0)?;
    let spi = resources
        .spi
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("SPI is not open"))?;
    spi.write_all(&[command])?;
    Ok(())
}

fn send_data(resources: &mut HostResources, data: u8) -> anyhow::Result<()> {
    let dc = resources
        .dc
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("DC line is not open"))?;
    dc.set_value(1)?;
    let spi = resources
        .spi
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("SPI is not open"))?;
    spi.write_all(&[data])?;
    Ok(())
}

fn send_data_bulk(resources: &mut HostResources, data: &[u8]) -> anyhow::Result<()> {
    let dc = resources
        .dc
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("DC line is not open"))?;
    dc.set_value(1)?;
    let spi = resources
        .spi
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("SPI is not open"))?;
    for chunk in data.chunks(SPI_CHUNK) {
        spi.write_all(chunk)?;
    }
    Ok(())
}

fn nearest_colour(rgb: [u8; 3]) -> Colour {
    Colour::ALL
        .iter()
        .min_by_key(|colour| {
            let palette = colour.rgb();
            let dr = palette[0] as i32 - rgb[0] as i32;
            let dg = palette[1] as i32 - rgb[1] as i32;
            let db = palette[2] as i32 - rgb[2] as i32;
            dr * dr + dg * dg + db * db
        })
        .cloned()
        .expect("Colour::ALL is non-empty")
}

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
        .map(|pixel| nearest_colour([pixel[0], pixel[1], pixel[2]]).panel_code())
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
    use std::collections::VecDeque;
    use std::ffi::OsString;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Event {
        LineSet(&'static str, u8),
        LineGet(&'static str),
        SpiOpen,
        SpiWrite(Vec<u8>),
        SpiClose,
    }

    type Log = Arc<Mutex<Vec<Event>>>;

    fn event_log() -> Log {
        Arc::new(Mutex::new(Vec::new()))
    }

    #[derive(Clone)]
    struct FakeGpio {
        name: &'static str,
        state: Arc<Mutex<FakeGpioState>>,
        log: Log,
    }

    #[derive(Default)]
    struct FakeGpioState {
        value: u8,
        fail_set: bool,
        fail_get: bool,
        reads: VecDeque<Result<u8, &'static str>>,
    }

    impl FakeGpio {
        fn new(name: &'static str, log: Log) -> Self {
            Self {
                name,
                state: Arc::new(Mutex::new(FakeGpioState::default())),
                log,
            }
        }

        fn idle(name: &'static str, log: Log) -> Self {
            let line = Self::new(name, log);
            line.set_value(1).unwrap();
            line
        }

        fn failing_get(name: &'static str, log: Log) -> Self {
            let line = Self::new(name, log);
            line.state.lock().unwrap().fail_get = true;
            line
        }

        fn value(&self) -> u8 {
            self.state.lock().unwrap().value
        }
    }

    impl GpioLine for FakeGpio {
        fn set_value(&self, value: u8) -> anyhow::Result<()> {
            self.log
                .lock()
                .unwrap()
                .push(Event::LineSet(self.name, value));
            let mut state = self.state.lock().unwrap();
            state.value = value;
            if state.fail_set {
                anyhow::bail!("fake gpio set failure");
            }
            Ok(())
        }

        fn get_value(&self) -> anyhow::Result<u8> {
            self.log.lock().unwrap().push(Event::LineGet(self.name));
            let mut state = self.state.lock().unwrap();
            if state.fail_get {
                anyhow::bail!("fake gpio read failure");
            }
            match state.reads.pop_front() {
                Some(Ok(value)) => Ok(value),
                Some(Err(message)) => anyhow::bail!(message),
                None => Ok(state.value),
            }
        }
    }

    #[derive(Clone)]
    struct FakeSpi {
        state: Arc<Mutex<FakeSpiState>>,
        log: Log,
    }

    #[derive(Default)]
    struct FakeSpiState {
        writes: Vec<Vec<u8>>,
        closed: bool,
        fail_all: bool,
        fail_on_command: Option<u8>,
        fail_on_bulk: bool,
        fail_close: bool,
    }

    impl FakeSpi {
        fn new(log: Log) -> Self {
            Self {
                state: Arc::new(Mutex::new(FakeSpiState::default())),
                log,
            }
        }

        fn failing_all(log: Log) -> Self {
            let spi = Self::new(log);
            spi.state.lock().unwrap().fail_all = true;
            spi
        }

        fn failing_on_command(log: Log, command: u8) -> Self {
            let spi = Self::new(log);
            spi.state.lock().unwrap().fail_on_command = Some(command);
            spi
        }

        fn failing_on_bulk(log: Log) -> Self {
            let spi = Self::new(log);
            spi.state.lock().unwrap().fail_on_bulk = true;
            spi
        }

        fn writes(&self) -> Vec<Vec<u8>> {
            self.state.lock().unwrap().writes.clone()
        }

        fn closed(&self) -> bool {
            self.state.lock().unwrap().closed
        }
    }

    impl SpiBus for FakeSpi {
        fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
            self.log.lock().unwrap().push(Event::SpiWrite(buf.to_vec()));
            let mut state = self.state.lock().unwrap();
            state.writes.push(buf.to_vec());
            if state.fail_all
                || state.fail_on_command == buf.first().copied()
                || (state.fail_on_bulk && buf.len() > 1)
            {
                return Err(std::io::Error::other("fake spi write failure"));
            }
            Ok(())
        }

        fn close(&mut self) -> std::io::Result<()> {
            self.log.lock().unwrap().push(Event::SpiClose);
            let mut state = self.state.lock().unwrap();
            state.closed = true;
            if state.fail_close {
                return Err(std::io::Error::other("fake spi close failure"));
            }
            Ok(())
        }
    }

    struct FakeBackend {
        spi: FakeSpi,
        rst: FakeGpio,
        dc: FakeGpio,
        busy: FakeGpio,
        pwr: FakeGpio,
        log: Log,
    }

    impl FakeBackend {
        fn new(log: Log, spi: FakeSpi, busy: FakeGpio) -> Self {
            Self {
                spi,
                rst: FakeGpio::new("rst", log.clone()),
                dc: FakeGpio::new("dc", log.clone()),
                busy,
                pwr: FakeGpio::new("pwr", log.clone()),
                log,
            }
        }
    }

    impl HostBackend for FakeBackend {
        fn request_output(
            &mut self,
            _line: u32,
            initial: u8,
            consumer: &'static str,
        ) -> anyhow::Result<Box<dyn GpioLine>> {
            let line = match consumer {
                "corkboard-panel-pwr" => self.pwr.clone(),
                "corkboard-panel-rst" => self.rst.clone(),
                "corkboard-panel-dc" => self.dc.clone(),
                _ => anyhow::bail!("unexpected output consumer {consumer}"),
            };
            line.set_value(initial)?;
            Ok(Box::new(line))
        }

        fn request_input(
            &mut self,
            _line: u32,
            consumer: &'static str,
        ) -> anyhow::Result<Box<dyn GpioLine>> {
            if consumer != "corkboard-panel-busy" {
                anyhow::bail!("unexpected input consumer {consumer}");
            }
            Ok(Box::new(self.busy.clone()))
        }

        fn open_spi(&mut self, _path: &str) -> anyhow::Result<Box<dyn SpiBus>> {
            self.log.lock().unwrap().push(Event::SpiOpen);
            Ok(Box::new(self.spi.clone()))
        }
    }

    #[derive(Clone)]
    struct RecordingOpener {
        backend: Arc<Mutex<FakeBackend>>,
    }

    impl RecordingOpener {
        fn new(log: Log, spi: FakeSpi, busy: FakeGpio) -> Self {
            Self {
                backend: Arc::new(Mutex::new(FakeBackend::new(log, spi, busy))),
            }
        }

        fn pwr(&self) -> FakeGpio {
            self.backend.lock().unwrap().pwr.clone()
        }
    }

    impl HostOpener for RecordingOpener {
        fn open(&self, cfg: &PanelConfig) -> anyhow::Result<HostResources> {
            open_resources(cfg, &mut *self.backend.lock().unwrap())
        }
    }

    fn env_from(
        pairs: &'static [(&'static str, &'static str)],
    ) -> impl Fn(&str) -> Result<String, std::env::VarError> {
        move |name| {
            pairs
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| value.to_string())
                .ok_or(std::env::VarError::NotPresent)
        }
    }

    fn config_with_pwr() -> PanelConfig {
        PanelConfig {
            spi_path: "/dev/spidev-test".to_string(),
            gpiochip_path: "/dev/gpiochip-test".to_string(),
            rst_line: 1,
            dc_line: 2,
            busy_line: 3,
            pwr_line: Some(4),
        }
    }

    fn valid_frame_png() -> Vec<u8> {
        let img = image::RgbImage::new(WIDTH as u32, HEIGHT as u32);
        let mut png = Vec::new();
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        png
    }

    fn show_with(spi: FakeSpi, log: Log) -> (anyhow::Result<()>, RecordingOpener) {
        let busy = FakeGpio::idle("busy", log.clone());
        let opener = RecordingOpener::new(log, spi, busy);
        let panel = Panel::new(config_with_pwr());
        let result = panel.show_with_opener(&valid_frame_png(), &opener);
        (result, opener)
    }

    fn control_bytes(writes: &[Vec<u8>]) -> Vec<u8> {
        writes
            .iter()
            .filter(|write| write.len() == 1)
            .map(|write| write[0])
            .collect()
    }

    fn frame_bytes(writes: &[Vec<u8>]) -> Vec<u8> {
        writes
            .iter()
            .filter(|write| write.len() > 1)
            .flatten()
            .copied()
            .collect()
    }

    #[rustfmt::skip]
    fn expected_init_control() -> Vec<u8> {
        vec![
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
        ]
    }

    fn indices(events: &[Event], needle: &Event) -> Vec<usize> {
        events
            .iter()
            .enumerate()
            .filter_map(|(idx, event)| (event == needle).then_some(idx))
            .collect()
    }

    fn assert_teardown_ran(spi: &FakeSpi, opener: &RecordingOpener, events: &[Event]) {
        assert!(spi.closed(), "SPI was closed");
        assert_eq!(opener.pwr().value(), 0, "PWR was dropped low");

        let rst_low = indices(events, &Event::LineSet("rst", 0))
            .last()
            .copied()
            .expect("RST low event");
        let dc_low = indices(events, &Event::LineSet("dc", 0))
            .last()
            .copied()
            .expect("DC low event");
        let spi_close = indices(events, &Event::SpiClose)
            .last()
            .copied()
            .expect("SPI close event");
        let pwr_low = indices(events, &Event::LineSet("pwr", 0))
            .last()
            .copied()
            .expect("PWR low event");

        assert!(rst_low < spi_close, "RST low before SPI close");
        assert!(dc_low < spi_close, "DC low before SPI close");
        assert!(spi_close < pwr_low, "SPI close before PWR low");
    }

    #[test]
    fn happy_path_sends_full_reference_skeleton_and_tears_down() {
        let log = event_log();
        let spi = FakeSpi::new(log.clone());
        let spi_probe = spi.clone();
        let (result, opener) = show_with(spi, log.clone());

        result.unwrap();

        // Ends at DEEP_SLEEP (0x07,0xA5): the clean-path teardown no longer
        // re-sends POWER_OFF, so there is no trailing 0x02,0x00.
        let writes = spi_probe.writes();
        let mut expected = expected_init_control();
        expected.extend([0x10, 0x04, 0x12, 0x00, 0x02, 0x00, 0x07, 0xA5]);
        assert_eq!(control_bytes(&writes), expected);

        let frame = frame_bytes(&writes);
        assert_eq!(frame.len(), WIDTH * HEIGHT / 2);
        assert!(frame.iter().all(|&byte| byte == 0x00));

        assert_teardown_ran(&spi_probe, &opener, &log.lock().unwrap());
    }

    #[test]
    fn teardown_runs_when_init_power_on_write_fails() {
        let log = event_log();
        let spi = FakeSpi::failing_on_command(log.clone(), 0x04);
        let spi_probe = spi.clone();
        let (result, opener) = show_with(spi, log.clone());

        assert!(result.is_err());
        let writes = spi_probe.writes();
        assert!(writes.contains(&vec![0x02]));
        assert!(writes.contains(&vec![0x00]));
        assert!(!control_bytes(&writes).contains(&0x07));
        assert_teardown_ran(&spi_probe, &opener, &log.lock().unwrap());
    }

    #[test]
    fn teardown_runs_when_bulk_frame_write_fails() {
        let log = event_log();
        let spi = FakeSpi::failing_on_bulk(log.clone());
        let spi_probe = spi.clone();
        let (result, opener) = show_with(spi, log.clone());

        assert!(result.is_err());
        let writes = spi_probe.writes();
        assert!(writes.contains(&vec![0x02]));
        assert!(!control_bytes(&writes).contains(&0x07));
        assert_teardown_ran(&spi_probe, &opener, &log.lock().unwrap());
    }

    #[test]
    fn teardown_runs_when_display_refresh_write_fails() {
        let log = event_log();
        let spi = FakeSpi::failing_on_command(log.clone(), 0x12);
        let spi_probe = spi.clone();
        let (result, opener) = show_with(spi, log.clone());

        assert!(result.is_err());
        let writes = spi_probe.writes();
        assert!(writes.contains(&vec![0x02]));
        assert!(!control_bytes(&writes).contains(&0x07));
        assert_teardown_ran(&spi_probe, &opener, &log.lock().unwrap());
    }

    #[test]
    fn teardown_does_not_panic_when_all_spi_writes_fail() {
        let log = event_log();
        let spi = FakeSpi::failing_all(log.clone());
        let spi_probe = spi.clone();
        let (result, opener) = show_with(spi, log.clone());

        assert!(result.is_err());
        assert_teardown_ran(&spi_probe, &opener, &log.lock().unwrap());
    }

    #[test]
    fn pwr_is_asserted_before_spi_is_opened() {
        let log = event_log();
        let spi = FakeSpi::new(log.clone());
        let (result, _opener) = show_with(spi, log.clone());

        result.unwrap();
        let events = log.lock().unwrap();
        let pwr_high = indices(&events, &Event::LineSet("pwr", 1))[0];
        let spi_open = indices(&events, &Event::SpiOpen)[0];
        assert!(pwr_high < spi_open);
    }

    #[test]
    fn send_data_bulk_chunks_without_losing_or_reordering_bytes() {
        let log = event_log();
        let spi = FakeSpi::new(log);
        let spi_probe = spi.clone();
        let mut resources = HostResources {
            spi: Some(Box::new(spi)),
            rst: None,
            dc: Some(Box::new(FakeGpio::new("dc", event_log()))),
            busy: None,
            pwr: None,
        };
        let data: Vec<u8> = (0..(SPI_CHUNK * 2 + 100))
            .map(|idx| (idx % 251) as u8)
            .collect();

        send_data_bulk(&mut resources, &data).unwrap();

        let writes = spi_probe.writes();
        assert!(writes.len() > 1);
        assert!(writes.iter().all(|chunk| chunk.len() <= SPI_CHUNK));
        assert_eq!(writes.concat(), data);
    }

    #[test]
    fn wait_busy_succeeds_when_idle() {
        let log = event_log();
        assert!(wait_busy(&FakeGpio::idle("busy", log), Duration::from_secs(1)).is_ok());
    }

    #[test]
    fn wait_busy_times_out_when_line_stays_busy() {
        let log = event_log();
        let busy = FakeGpio::new("busy", log);
        assert!(wait_busy(&busy, Duration::from_millis(1)).is_err());
    }

    #[test]
    fn wait_busy_propagates_read_errors() {
        let log = event_log();
        assert!(wait_busy(&FakeGpio::failing_get("busy", log), Duration::from_secs(1)).is_err());
    }

    #[test]
    fn packs_two_pixels_per_byte_high_nibble_first() {
        let mut img = image::RgbImage::new(WIDTH as u32, HEIGHT as u32);
        img.put_pixel(0, 0, image::Rgb(Colour::Black.rgb()));
        img.put_pixel(1, 0, image::Rgb(Colour::White.rgb()));
        img.put_pixel(2, 0, image::Rgb(Colour::Red.rgb()));
        img.put_pixel(3, 0, image::Rgb(Colour::Green.rgb()));

        let packed = pack(&img).unwrap();
        assert_eq!(packed[0], 0x01);
        assert_eq!(packed[1], 0x36);
        assert_eq!(packed.len(), WIDTH * HEIGHT / 2);
    }

    #[test]
    fn nearest_colour_matches_exact_palette_values() {
        for colour in Colour::ALL {
            assert_eq!(
                nearest_colour(colour.rgb()).panel_code(),
                colour.panel_code()
            );
        }
    }

    #[test]
    fn pack_rejects_wrong_dimensions() {
        let img = image::RgbImage::new(10, 10);
        assert!(pack(&img).is_err());
    }

    #[test]
    fn from_getter_errors_when_required_var_is_missing() {
        let get = env_from(&[
            ("CORKBOARD_PANEL_RST_LINE", "1"),
            ("CORKBOARD_PANEL_DC_LINE", "2"),
            ("CORKBOARD_PANEL_BUSY_LINE", "3"),
            ("CORKBOARD_PANEL_PWR_LINE", "4"),
        ]);

        assert!(PanelConfig::from_getter(&get).is_err());
    }

    #[test]
    fn from_getter_no_pwr_opt_out_yields_no_pwr_line() {
        let get = env_from(&[
            ("CORKBOARD_PANEL_GPIOCHIP", "/dev/gpiochip0"),
            ("CORKBOARD_PANEL_RST_LINE", "1"),
            ("CORKBOARD_PANEL_DC_LINE", "2"),
            ("CORKBOARD_PANEL_BUSY_LINE", "3"),
            ("CORKBOARD_PANEL_NO_PWR", "1"),
        ]);

        let cfg = PanelConfig::from_getter(&get).unwrap();
        assert_eq!(cfg.spi_path, "/dev/spidev0.0");
        assert_eq!(cfg.pwr_line, None);
    }

    #[test]
    fn from_getter_missing_pwr_without_opt_out_errors() {
        let get = env_from(&[
            ("CORKBOARD_PANEL_GPIOCHIP", "/dev/gpiochip0"),
            ("CORKBOARD_PANEL_RST_LINE", "1"),
            ("CORKBOARD_PANEL_DC_LINE", "2"),
            ("CORKBOARD_PANEL_BUSY_LINE", "3"),
        ]);

        assert!(PanelConfig::from_getter(&get).is_err());
    }

    #[test]
    fn from_getter_non_unicode_pwr_var_errors() {
        fn get(name: &str) -> Result<String, std::env::VarError> {
            if name == "CORKBOARD_PANEL_PWR_LINE" {
                return Err(std::env::VarError::NotUnicode(OsString::from("bad")));
            }
            env_from(&[
                ("CORKBOARD_PANEL_GPIOCHIP", "/dev/gpiochip0"),
                ("CORKBOARD_PANEL_RST_LINE", "1"),
                ("CORKBOARD_PANEL_DC_LINE", "2"),
                ("CORKBOARD_PANEL_BUSY_LINE", "3"),
            ])(name)
        }

        assert!(PanelConfig::from_getter(&get).is_err());
    }

    #[test]
    fn from_getter_real_pwr_line_parses() {
        let get = env_from(&[
            ("CORKBOARD_PANEL_GPIOCHIP", "/dev/gpiochip0"),
            ("CORKBOARD_PANEL_RST_LINE", "1"),
            ("CORKBOARD_PANEL_DC_LINE", "2"),
            ("CORKBOARD_PANEL_BUSY_LINE", "3"),
            ("CORKBOARD_PANEL_PWR_LINE", "4"),
        ]);

        let cfg = PanelConfig::from_getter(&get).unwrap();
        assert_eq!(cfg.pwr_line, Some(4));
    }

    #[test]
    fn from_getter_non_unicode_required_var_errors() {
        fn get(name: &str) -> Result<String, std::env::VarError> {
            if name == "CORKBOARD_PANEL_GPIOCHIP" {
                return Err(std::env::VarError::NotUnicode(OsString::from("bad")));
            }
            env_from(&[
                ("CORKBOARD_PANEL_RST_LINE", "1"),
                ("CORKBOARD_PANEL_DC_LINE", "2"),
                ("CORKBOARD_PANEL_BUSY_LINE", "3"),
                ("CORKBOARD_PANEL_PWR_LINE", "4"),
            ])(name)
        }

        assert!(PanelConfig::from_getter(&get).is_err());
    }
}
