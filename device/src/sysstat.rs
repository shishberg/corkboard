/// System-level stats for the dashboard's power/health panel: CPU temp, load
/// average, CPU frequency (a proxy for power draw — there's no onboard power
/// monitoring on the Orange Pi Zero 2W), and memory. All read from `/proc` and
/// `/sys`, so this is Linux-only in practice — on any other OS (e.g. a dev
/// Mac) the paths simply don't exist and every field comes back `None`. No
/// `cfg(target_os = "linux")` needed for that reason.
use serde::Serialize;

#[derive(Serialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub cpu_temp_c: Option<f64>,
    /// 1/5/15-minute exponentially-decayed run-queue averages, straight from
    /// `/proc/loadavg` — not instantaneous CPU%.
    pub load_avg_1: Option<f64>,
    pub load_avg_5: Option<f64>,
    pub load_avg_15: Option<f64>,
    /// Current CPU 0 clock speed. Tracks the cpufreq governor's decisions and
    /// reacts immediately (unlike temp, which lags), so it's a better proxy
    /// for instantaneous power draw.
    pub cpu_freq_mhz: Option<u64>,
    pub mem_total_kb: Option<u64>,
    pub mem_available_kb: Option<u64>,
}

pub fn snapshot() -> SystemInfo {
    let load = read_load_avg();
    let (mem_total_kb, mem_available_kb) = read_mem_kb();
    SystemInfo {
        cpu_temp_c: read_cpu_temp_c(),
        load_avg_1: load.map(|l| l.0),
        load_avg_5: load.map(|l| l.1),
        load_avg_15: load.map(|l| l.2),
        cpu_freq_mhz: read_cpu_freq_mhz(),
        mem_total_kb,
        mem_available_kb,
    }
}

fn read_load_avg() -> Option<(f64, f64, f64)> {
    parse_load_avg(&std::fs::read_to_string("/proc/loadavg").ok()?)
}

fn read_cpu_temp_c() -> Option<f64> {
    parse_temp_millideg(&std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp").ok()?)
}

fn read_cpu_freq_mhz() -> Option<u64> {
    parse_freq_khz(
        &std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq").ok()?,
    )
}

fn read_mem_kb() -> (Option<u64>, Option<u64>) {
    match std::fs::read_to_string("/proc/meminfo") {
        Ok(text) => parse_meminfo(&text),
        Err(_) => (None, None),
    }
}

fn parse_load_avg(text: &str) -> Option<(f64, f64, f64)> {
    let mut parts = text.split_whitespace();
    let one: f64 = parts.next()?.parse().ok()?;
    let five: f64 = parts.next()?.parse().ok()?;
    let fifteen: f64 = parts.next()?.parse().ok()?;
    Some((one, five, fifteen))
}

fn parse_temp_millideg(text: &str) -> Option<f64> {
    let millideg: f64 = text.trim().parse().ok()?;
    Some(millideg / 1000.0)
}

fn parse_freq_khz(text: &str) -> Option<u64> {
    let khz: u64 = text.trim().parse().ok()?;
    Some(khz / 1000)
}

fn parse_meminfo(text: &str) -> (Option<u64>, Option<u64>) {
    let mut total = None;
    let mut available = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            total = rest.trim().split_whitespace().next().and_then(|s| s.parse().ok());
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            available = rest.trim().split_whitespace().next().and_then(|s| s.parse().ok());
        }
    }
    (total, available)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_loadavg_line() {
        let (one, five, fifteen) = parse_load_avg("0.52 0.58 0.59 2/187 12345").unwrap();
        assert_eq!(one, 0.52);
        assert_eq!(five, 0.58);
        assert_eq!(fifteen, 0.59);
    }

    #[test]
    fn parses_thermal_zone_millidegrees() {
        assert_eq!(parse_temp_millideg("42800\n"), Some(42.8));
    }

    #[test]
    fn parses_scaling_cur_freq_khz() {
        assert_eq!(parse_freq_khz("1008000\n"), Some(1008));
    }

    #[test]
    fn parses_meminfo_relevant_fields() {
        let text = "MemTotal:       1998960 kB\n\
                     MemFree:          89216 kB\n\
                     MemAvailable:    512340 kB\n";
        assert_eq!(parse_meminfo(text), (Some(1998960), Some(512340)));
    }

    #[test]
    fn snapshot_never_panics_even_when_proc_paths_are_absent() {
        // On a dev Mac none of these paths exist; every field should just be `None`.
        let _ = snapshot();
    }
}
