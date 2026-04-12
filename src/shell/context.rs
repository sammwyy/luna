use indexmap::IndexMap;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// A unified context holding all shell state for a single "cycle" (e.g. prompt rendering or command execution).
/// This data is shared between:
/// 1. Rust variable interpolation (expand_vars)
/// 2. Lua Theme rendering
/// 3. Lua Plugin hooks
#[derive(Clone, Debug)]
pub struct ShellContext {
    pub last_exit_code: i32,
    pub last_duration_ms: u128,
    pub cwd: String,
    pub cwd_home: String,
    pub cwd_short: String,
    pub user: String,
    pub hostname: String,
    pub shell: String,
    pub pid: u32,

    pub time_h: String,
    pub time_m: String,
    pub time_s: String,
    pub date_y: u32,
    pub date_mo: u32,
    pub date_d: u32,
    pub date_iso: String,
    pub datetime: String,

    pub env: IndexMap<String, String>,
    pub vars: HashMap<String, String>,
}

impl ShellContext {
    pub fn new(
        last_exit_code: i32,
        last_duration_ms: u128,
        cwd: &str,
        env: &IndexMap<String, String>,
        vars: &HashMap<String, String>,
    ) -> Self {
        let (year, month, day, hour, minute, second) = local_time();

        let cwd_home = if let Some(home) = env.get("HOME") {
            if cwd.starts_with(home.as_str()) {
                format!("~{}", &cwd[home.len()..])
            } else {
                cwd.to_string()
            }
        } else {
            cwd.to_string()
        };

        let cwd_short = cwd
            .split('/')
            .filter(|s| !s.is_empty())
            .last()
            .unwrap_or(cwd)
            .to_string();

        let user = env
            .get("USER")
            .or_else(|| env.get("LOGNAME"))
            .cloned()
            .unwrap_or_else(|| "user".to_string());

        let hostname = std::fs::read_to_string("/proc/sys/kernel/hostname")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "localhost".to_string());

        Self {
            last_exit_code,
            last_duration_ms,
            cwd: cwd.to_string(),
            cwd_home,
            cwd_short,
            user,
            hostname,
            shell: "luna".to_string(),
            pid: std::process::id(),

            time_h: format!("{:02}", hour),
            time_m: format!("{:02}", minute),
            time_s: format!("{:02}", second),
            date_y: year,
            date_mo: month,
            date_d: day,
            date_iso: format!("{:04}-{:02}-{:02}", year, month, day),
            datetime: format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                year, month, day, hour, minute, second
            ),

            env: env.clone(),
            vars: vars.clone(),
        }
    }
}

// ─── Local time algorithm ───────────────────────────────────────────────────

fn local_time() -> (u32, u32, u32, u32, u32, u32) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let offset_secs: i64 = unsafe {
        extern "C" {
            static timezone: i64;
            fn tzset();
        }
        tzset();
        -timezone
    };
    let local_secs = (secs as i64 + offset_secs).max(0) as u64;
    let days = local_secs / 86400;
    let tod = local_secs % 86400;
    let h = (tod / 3600) as u32;
    let m = ((tod % 3600) / 60) as u32;
    let s = (tod % 60) as u32;
    let (year, month, day) = days_to_ymd(days);
    (year, month, day, h, m, s)
}

fn days_to_ymd(days: u64) -> (u32, u32, u32) {
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as u32, m as u32, d as u32)
}
