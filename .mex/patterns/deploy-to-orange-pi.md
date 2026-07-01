---
name: deploy-to-orange-pi
description: Flash Armbian onto the Orange Pi Zero 2W, get headless SSH access over WiFi (no keyboard/mouse/GUI), build and run the device server as a systemd service. Ends where the physical panel driver work begins.
triggers:
  - "orange pi"
  - "armbian"
  - "flash"
  - "headless"
  - "systemd"
  - "deploy"
  - "sd card"
edges:
  - target: context/hardware.md
    condition: for the panel model, SPI interface, and OS choice this runbook assumes
  - target: context/decisions.md
    condition: for why Armbian (Debian) was chosen over Orange Pi OS (Arch)
  - target: context/setup.md
    condition: for the dev-machine toolchain (Node, Rust) this runbook builds on
last_updated: 2026-07-02
---

# Deploy the Device Server to the Orange Pi Zero 2W (Armbian, headless)

## Context
Target: Orange Pi Zero 2W running Armbian (Debian), **SSH-only** — no monitor, keyboard,
or GUI ever touches it. This runbook gets from a blank microSD card to the device server
running as a systemd service and serving `/preview.png` over the LAN. It stops there —
driving the physical Waveshare 7.3" E6 panel needs a new `Display` impl in
`device/src/display.rs` (today there's only `WebPreview`); that's separate coding work,
not part of this setup.

## Steps

1. **Flash Armbian.** Download the Armbian **Debian** (Bookworm) image for Orange Pi
   Zero 2W and flash it to a microSD card (balenaEtcher, Raspberry Pi Imager, or `dd`).
2. **Pre-seed WiFi + headless boot** — do this before first boot, while the card is
   still mounted on your dev machine. On the boot partition, copy
   `armbian_first_run.txt.template` to `armbian_first_run.txt` and set:
   ```
   FR_general_delete_this_file_after_completion=1
   FR_net_change_defaults=1
   FR_net_wifi_enabled=1
   FR_net_wifi_ssid='<your SSID>'
   FR_net_wifi_key='<your password>'
   FR_net_wifi_countrycode='<your country code>'
   ```
   This runs once on first boot, joins the network, and deletes itself. SSH is on by
   default on Armbian images — no separate step needed.
3. **Boot it and find its IP.** Check your router's DHCP client list, or
   `nmap -sn <your-subnet>/24`, or try `ssh root@orangepizero2w.local` if mDNS resolves.
4. **First SSH login.** `ssh root@<ip>`. Armbian forces a root password change on first
   login and walks you through creating a non-root user — use that user (with sudo) from
   here on.
5. `sudo apt update && sudo apt full-upgrade -y`
6. **Make `<hostname>.local` resolve (mDNS).** `.local` names aren't DNS and the eero
   router does nothing for them — each device answers for its own name over mDNS. macOS
   (Bonjour) does this out of the box; Armbian doesn't until you install a responder:
   ```
   sudo hostnamectl set-hostname calcifer   # pick the name you want; no reboot needed
   sudo apt install -y avahi-daemon
   sudo systemctl enable --now avahi-daemon
   ```
   Now `calcifer.local` resolves from any Mac/Linux box on the LAN (`ping calcifer.local`).
   Do this once per Pi with a unique hostname, or two boards will both claim the same
   `.local` name.
7. **Enable SPI.** Edit `/boot/armbianEnv.txt` and set `overlays=spidev0_0` — the
   H616's purpose-built overlay that needs no parameters and gives you
   `/dev/spidev0.0` (bus 0, CS 0). Reboot, then confirm `/dev/spidev0.0` exists.

   Do NOT use the generic `spi-spidev` overlay alone: it requires a
   `param_spidev_spi_bus=0` line as well, and silently binds to nothing without it
   (no `/dev/spidev*` node appears). `/dev/mtd/by-name/spi0.0` is the SPI *flash
   chip*, not the userspace SPI interface — it's there regardless and is not what
   the panel driver opens.
8. **Install build deps:** `sudo apt install -y git build-essential pkg-config zlib1g-dev` —
   `freetype-rs`'s `bundled` feature compiles FreeType from C source, so a compiler is
   required. It also builds a bundled libpng, which needs zlib's dev headers
   (`zlib1g-dev`) — without them the build fails with `fatal error: zlib.h: No such
   file or directory`.
9. **Install Rust:** `curl https://sh.rustup.rs -sSf | sh`, then
   `source "$HOME/.cargo/env"`. Native on-device build is simplest — no cross-compile
   target is set up. It's slow on a quad-core A53 (FreeType compiles from source; expect
   several minutes on the first build) but only needs to happen when the Rust code
   changes.
10. **Get the code across.** The editor (`npm run build`) doesn't need to run on the Pi —
   build `dist/` on your dev machine and skip installing Node there. Two ways to land
   files:
   - `git clone` the repo directly on the Pi (needs a remote), then `rsync` just the
     freshly-built `dist/` from your dev machine over the top, or
   - `rsync -av --exclude target --exclude node_modules --exclude .git ./ pi@<ip>:~/corkboard/`
     from your dev machine, having run `npm run build` first so `dist/` is included.

   Either way, also copy `device/data/config.json` **out of band** (scp, not git) — it
   holds the real secret iCal feed URL, which must never enter git history
   (`context/protocol.md`).
11. **Build the device server:** `cd ~/corkboard/device && cargo build --release` →
    binary at `device/target/release/corkboard-device`.
12. **systemd service** — `/etc/systemd/system/corkboard.service`:
    ```ini
    [Unit]
    Description=Corkboard device server
    After=network-online.target
    Wants=network-online.target

    [Service]
    Type=simple
    User=pi
    WorkingDirectory=/home/pi/corkboard/device
    Environment=CORKBOARD_DIST=/home/pi/corkboard/dist
    Environment=CORKBOARD_FONTS=/home/pi/corkboard/public/fonts
    Environment=CORKBOARD_DATA=/home/pi/corkboard/device/data
    Environment=CORKBOARD_PORT=80
    ExecStart=/home/pi/corkboard/device/target/release/corkboard-device
    # Port 80 is privileged; this grants just the bind capability so the service
    # can stay non-root (User=pi) instead of running as root.
    AmbientCapabilities=CAP_NET_BIND_SERVICE
    Restart=on-failure

    [Install]
    WantedBy=multi-user.target
    ```
    Use absolute paths in the unit — unlike running `cargo run` from `device/`, systemd
    doesn't give you the `../dist`/`../public/fonts` relative defaults for free.
    `sudo systemctl daemon-reload && sudo systemctl enable --now corkboard`
13. **Verify:** `http://<pi-ip-or-hostname>/preview.png` loads from another machine
    on the LAN.
14. **Power tuning.** This board mostly sits idle serving an occasional request, so it's
    worth keeping it out of high-power states. Prefer each subsystem's own persistence
    mechanism over a custom unit — but on Armbian 26.8 trixie (the image this was actually
    verified against, on `calcifer`), neither the CPU governor nor the WiFi radio has one
    available, so both need a small oneshot unit:
    - **CPU governor.** `cpufrequtils` isn't packaged on trixie (no installation
      candidate). Its replacement, `sudo apt install -y linux-cpupower`, ships only the
      `cpupower` CLI — no `/etc/default/...` config file or boot-time service. Pin it with
      `/etc/systemd/system/corkboard-cpu-governor.service`:
      ```ini
      [Unit]
      Description=Pin the CPU frequency governor to ondemand (idle diagnostic device)
      After=multi-user.target

      [Service]
      Type=oneshot
      RemainAfterExit=yes
      ExecStart=/usr/bin/cpupower frequency-set -g ondemand

      [Install]
      WantedBy=multi-user.target
      ```
      Check `scaling_available_governors` first and prefer `schedutil` if it's listed
      (newer, generally better) — on `calcifer` the default was already `ondemand`, so this
      unit just guarantees it survives a kernel/image update that might reset it.
    - **Bluetooth.** `rfkill` is already installed (part of `util-linux`, lives in
      `/usr/sbin` — not on a non-interactive SSH session's `$PATH`, so use the full path or
      `sudo rfkill`). Just run `sudo /usr/sbin/rfkill block bluetooth` once, interactively.
      `systemd-rfkill.socket` (static-enabled by default) saves that state on shutdown and
      restores it on boot — no unit file needed.
    - **WiFi power-save.** *If* NetworkManager manages the interface (`systemctl status
      NetworkManager`), add `/etc/NetworkManager/conf.d/wifi-powersave.conf`:
      ```ini
      [connection]
      wifi.powersave=3
      ```
      applied to every connection NM brings up, no unit needed. On `calcifer` the network
      stack is netplan + `systemd-networkd` + `wpa_supplicant` (NetworkManager isn't
      running), so there's no equivalent single config file — used the fallback instead:
      `sudo apt install -y iw`, then `/etc/systemd/system/corkboard-wifi-powersave.service`:
      ```ini
      [Unit]
      Description=Enable WiFi power-save on wlan0 (idle diagnostic device)
      After=network-online.target
      Wants=network-online.target

      [Service]
      Type=oneshot
      RemainAfterExit=yes
      ExecStart=/usr/sbin/iw dev wlan0 set power_save on

      [Install]
      WantedBy=multi-user.target
      ```
      (Interface name confirmed with `ip link` — `wlan0` here.)

      Safe for a server either way: the access point buffers frames while the radio
      sleeps and delivers them at the next wake window (tied to the beacon interval,
      typically ~100ms) — it doesn't drop the connection or ignore requests, just adds up
      to ~100ms of latency to a request that lands mid-sleep.
    - Enable both units: `sudo systemctl daemon-reload && sudo systemctl enable --now
      corkboard-cpu-governor corkboard-wifi-powersave`. Verify with `cat
      .../scaling_governor` and `iw dev wlan0 get power_save`.
    - Dropping the frontend's 5s `/api/status` poll and the preview long-poll helps too:
      each request wakes the CPU out of deeper idle (C-states) and the WiFi radio out of
      power-save sleep to answer it. It costs no real CPU time, but it does block the
      deeper sleep states that actually save power — these settings only get their full
      benefit once the device isn't polled every few seconds.

## The panel driver — code is written, hardware verification is not
`device/src/panel.rs` implements `Display` for the real Waveshare 7.3" E6, ported
byte-for-byte from Waveshare's own `epd7in3e` demo (from the PhotoPainter kit's demo zip:
https://www.waveshare.com/wiki/RPi_Zero_PhotoPainter). It's Linux-only
(`#[cfg(target_os = "linux")]`, and `spidev`/`gpio-cdev` are Linux-only Cargo
dependencies), so it doesn't affect building/testing on a dev Mac. See
`context/decisions.md`'s "Panel driver" entry for the full rationale.

To actually drive the panel once it's wired to the board:
1. Find the real GPIO chip and line numbers for RST, DC, BUSY (and PWR, since this build
   uses the PhotoPainter carrier board, not a bare HAT) with `gpioinfo` — Waveshare's own
   BCM pin numbers (RST=17, DC=25, BUSY=24, PWR=27) are Raspberry Pi-specific and do not
   carry over to the Orange Pi's Allwinner H618, even though the 40-pin header is
   physically compatible. `PanelConfig::from_env` deliberately has no hardcoded pin
   defaults — it errors clearly if these aren't set, rather than risk silently driving the
   wrong line.
2. Add to the systemd unit's `[Service]` section:
   ```ini
   Environment=CORKBOARD_DISPLAY=panel
   Environment=CORKBOARD_PANEL_GPIOCHIP=/dev/gpiochipN
   Environment=CORKBOARD_PANEL_RST_LINE=<n>
   Environment=CORKBOARD_PANEL_DC_LINE=<n>
   Environment=CORKBOARD_PANEL_BUSY_LINE=<n>
   Environment=CORKBOARD_PANEL_PWR_LINE=<n>
   ```
   (`CORKBOARD_PANEL_SPI` defaults to `/dev/spidev0.0` if unset. `CORKBOARD_PANEL_PWR_LINE`
   is required for this carrier board — a missing value is a hard error, not a silent "no
   gate"; only set `CORKBOARD_PANEL_NO_PWR=1` instead if driving a bare HAT with no power
   gate.)
3. Watch `journalctl -u corkboard -f` on the first render — a wrong BUSY line
   number will surface as `panel BUSY line never went idle within 120s`.

What's still unverified against real hardware (can't be resolved without the physical
panel): whether the chunked SPI transfer of the 192,000-byte framebuffer refreshes
cleanly (the driver sends plain chunked `write_all`, matching `python-spidev`'s
`writebytes2`; the fix if a frame ever tears is to raise the kernel's `spidev.bufsiz` and
send it in one write — not `cs_change`), and the exact refresh timing.

## Gotchas
- `armbian_first_run.txt` only runs once and deletes itself — get the WiFi block right
  before first boot, or re-mount the card and redo it.
- The SPI overlay needs a reboot to take effect; the future panel driver silently can't
  find `/dev/spidev0.0` if you skip this.
- systemd doesn't inherit the relative-path env var defaults `cargo run` gives you from
  `device/` — always set `CORKBOARD_DIST`/`CORKBOARD_FONTS`/`CORKBOARD_DATA` as absolute
  paths in the unit file.
- Never let `device/data/config.json` (real feed secret URL) go through git.

## Verify
- [ ] SSH works over WiFi with no monitor ever attached.
- [ ] `<hostname>.local` resolves from another machine after installing `avahi-daemon`.
- [ ] `/dev/spidev0.0` exists after enabling the overlay and rebooting.
- [ ] `cargo build --release` succeeds on-device.
- [ ] `corkboard` systemd service starts on boot and survives a reboot.
- [ ] `/preview.png` is reachable from another machine on the LAN.
- [ ] Power tuning survives a reboot: governor still `ondemand`/`schedutil`, WiFi
  power-save still on, Bluetooth still blocked.
- [ ] Panel driver (`Display` impl beyond `WebPreview`) — separate, not yet done.

## Debug
- Can't SSH in: check `armbian_first_run.txt` for typos before reflashing; check the
  router's DHCP leases for the new device.
- Service won't start: `journalctl -u corkboard -e`; most likely cause is a wrong
  or relative `CORKBOARD_*` path in the unit file.
- Build fails: missing `build-essential`/`pkg-config` (needed for `freetype-rs`'s bundled
  C build).

## Update Scaffold
- [ ] Once the panel is wired up and lit for real, resolve the remaining `[UNVERIFIED]`
  items in `context/hardware.md` (GPIO chip/line numbers, refresh timing, CS-across-chunks
  behaviour) and update the "Panel driver" entry in `context/decisions.md`.
- [ ] Update `.mex/ROUTER.md` "Current Project State" once the hardware deploy is done
  end-to-end (panel actually lit up, not just the web preview).
