use std::fs::{self, File};
use std::io::{IsTerminal, Write};
use std::sync::atomic::{AtomicI64, AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

const RESET: &str = "\x1b[0m";
const THREAD: &str = "Server thread";

static OFFSET_SECS: AtomicI64 = AtomicI64::new(0);
static MAX_RANK: AtomicU8 = AtomicU8::new(2);
static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Error,
    Warn,
    Info,
    Success,
    Debug,
    Trace,
}

impl Level {
    pub fn rank(self) -> u8 {
        match self {
            Level::Error => 0,
            Level::Warn => 1,
            Level::Info | Level::Success => 2,
            Level::Debug => 3,
            Level::Trace => 4,
        }
    }

    pub fn tag(self) -> &'static str {
        match self {
            Level::Error => "ERROR",
            Level::Warn => "WARNING",
            Level::Info | Level::Success => "INFO",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Level::Error => "error",
            Level::Warn => "warn",
            Level::Info | Level::Success => "info",
            Level::Debug => "debug",
            Level::Trace => "trace",
        }
    }
}

// Level filter
pub fn parse_level(text: &str) -> Option<Level> {
    match text.trim().to_ascii_lowercase().as_str() {
        "error" => Some(Level::Error),
        "warn" | "warning" => Some(Level::Warn),
        "info" => Some(Level::Info),
        "debug" => Some(Level::Debug),
        "trace" => Some(Level::Trace),
        _ => None,
    }
}

pub fn set_max_level(level: Level) {
    MAX_RANK.store(level.rank(), Ordering::Relaxed);
}

pub fn max_level() -> Level {
    match MAX_RANK.load(Ordering::Relaxed) {
        0 => Level::Error,
        1 => Level::Warn,
        2 => Level::Info,
        3 => Level::Debug,
        _ => Level::Trace,
    }
}

pub fn enabled(level: Level) -> bool {
    level.rank() <= MAX_RANK.load(Ordering::Relaxed)
}

// Formatting
pub fn clock(secs_of_day: u64, millis: u32) -> String {
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        (secs_of_day / 3600) % 24,
        (secs_of_day / 60) % 60,
        secs_of_day % 60,
        millis
    )
}

pub fn format_line(level: Level, secs_of_day: u64, millis: u32, msg: &str, color: bool) -> String {
    let head = format!("[{}] [{THREAD}/{}]:", clock(secs_of_day, millis), level.tag());
    if !color {
        return format!("{head} {msg}");
    }
    match level {
        Level::Info => format!("\x1b[96m{head}{RESET} {msg}"),
        Level::Success => format!("\x1b[96m{head}{RESET} \x1b[92m{msg}{RESET}"),
        Level::Warn => format!("\x1b[93m{head} {msg}{RESET}"),
        Level::Error => format!("\x1b[91m{head} {msg}{RESET}"),
        Level::Debug | Level::Trace => format!("\x1b[90m{head} {msg}{RESET}"),
    }
}

// Time zone
pub fn parse_utc_offset(text: &str) -> Option<i64> {
    let t = text.trim();
    let (sign, rest) = match t.chars().next()? {
        '+' => (1, &t[1..]),
        '-' => (-1, &t[1..]),
        _ => (1, t),
    };
    let (hours_text, minutes_text) = rest.split_once(':').unwrap_or((rest, "0"));
    let hours: i64 = hours_text.parse().ok()?;
    let minutes: i64 = minutes_text.parse().ok()?;
    if !(0..=14).contains(&hours) || !(0..60).contains(&minutes) {
        return None;
    }
    Some(sign * (hours * 3600 + minutes * 60))
}

#[cfg(unix)]
pub fn auto_offset_secs() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let t: libc::time_t = now as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    let ok = unsafe { !libc::localtime_r(&t, &mut tm).is_null() };
    if ok { tm.tm_gmtoff as i64 } else { 0 }
}

#[cfg(not(unix))]
pub fn auto_offset_secs() -> i64 {
    0
}

pub fn set_utc_offset_secs(secs: i64) {
    OFFSET_SECS.store(secs, Ordering::Relaxed);
}

fn color_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal())
}

// Log file
pub fn init_file(dir: &str) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("cannot create '{dir}': {e}"))?;
    let latest = format!("{dir}/latest.log");
    let previous = format!("{dir}/previous.log");
    if fs::metadata(&latest).is_ok() {
        let _ = fs::rename(&latest, &previous);
    }
    let file = File::create(&latest).map_err(|e| format!("cannot create '{latest}': {e}"))?;
    let _ = LOG_FILE.set(Mutex::new(file));
    Ok(())
}

fn write_to_file(line: &str) {
    if let Some(file) = LOG_FILE.get() {
        let mut file = file.lock().unwrap_or_else(|e| e.into_inner());
        let _ = writeln!(file, "{line}");
    }
}

// Output
pub fn log(level: Level, msg: &str) {
    if !enabled(level) {
        return;
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let local = now.as_secs() as i64 + OFFSET_SECS.load(Ordering::Relaxed);
    let secs_of_day = local.rem_euclid(86_400) as u64;
    let millis = now.subsec_millis();
    let plain = format_line(level, secs_of_day, millis, msg, false);
    write_to_file(&plain);
    if color_enabled() {
        println!("{}", format_line(level, secs_of_day, millis, msg, true));
    } else {
        println!("{plain}");
    }
}

pub fn error(msg: &str) {
    log(Level::Error, msg);
}

pub fn warn(msg: &str) {
    log(Level::Warn, msg);
}

pub fn info(msg: &str) {
    log(Level::Info, msg);
}

pub fn success(msg: &str) {
    log(Level::Success, msg);
}

pub fn debug(msg: &str) {
    log(Level::Debug, msg);
}

#[allow(dead_code)]
pub fn trace(msg: &str) {
    log(Level::Trace, msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_has_milliseconds() {
        assert_eq!(clock(0, 0), "00:00:00.000");
        assert_eq!(clock(3661, 7), "01:01:01.007");
        assert_eq!(clock(86_400 + 59, 999), "00:00:59.999");
    }

    #[test]
    fn plain_lines_follow_the_server_thread_style() {
        assert_eq!(
            format_line(Level::Info, 1, 234, "hello", false),
            "[00:00:01.234] [Server thread/INFO]: hello"
        );
        assert_eq!(
            format_line(Level::Warn, 1, 0, "careful", false),
            "[00:00:01.000] [Server thread/WARNING]: careful"
        );
        assert!(format_line(Level::Success, 1, 0, "ready", false).contains("/INFO]"));
        assert!(format_line(Level::Debug, 1, 0, "x", false).contains("/DEBUG]"));
        assert!(format_line(Level::Trace, 1, 0, "x", false).contains("/TRACE]"));
        assert!(!format_line(Level::Error, 1, 0, "bad", false).contains('\x1b'));
    }

    #[test]
    fn colors_follow_the_level() {
        let info = format_line(Level::Info, 0, 0, "text", true);
        assert!(info.starts_with("\x1b[96m"));
        assert!(info.ends_with("\x1b[0m text"));
        let warn = format_line(Level::Warn, 0, 0, "text", true);
        assert!(warn.starts_with("\x1b[93m"));
        assert!(warn.ends_with("text\x1b[0m"));
        assert!(format_line(Level::Error, 0, 0, "x", true).starts_with("\x1b[91m"));
        assert!(format_line(Level::Success, 0, 0, "x", true).contains("\x1b[92m"));
        assert!(format_line(Level::Debug, 0, 0, "x", true).starts_with("\x1b[90m"));
    }

    #[test]
    fn levels_are_ordered_from_quiet_to_verbose() {
        assert!(Level::Error.rank() < Level::Warn.rank());
        assert!(Level::Warn.rank() < Level::Info.rank());
        assert_eq!(Level::Info.rank(), Level::Success.rank());
        assert!(Level::Info.rank() < Level::Debug.rank());
        assert!(Level::Debug.rank() < Level::Trace.rank());
    }

    #[test]
    fn level_names_are_parsed() {
        assert_eq!(parse_level("ERROR"), Some(Level::Error));
        assert_eq!(parse_level("warning"), Some(Level::Warn));
        assert_eq!(parse_level(" info "), Some(Level::Info));
        assert_eq!(parse_level("debug"), Some(Level::Debug));
        assert_eq!(parse_level("trace"), Some(Level::Trace));
        assert_eq!(parse_level("loud"), None);
        assert_eq!(parse_level("debug").map(Level::name), Some("debug"));
    }

    #[test]
    fn utc_offsets_are_parsed() {
        assert_eq!(parse_utc_offset("+03:30"), Some(12_600));
        assert_eq!(parse_utc_offset("-05:00"), Some(-18_000));
        assert_eq!(parse_utc_offset("3"), Some(10_800));
        assert_eq!(parse_utc_offset("0"), Some(0));
        assert_eq!(parse_utc_offset("+15:00"), None);
        assert_eq!(parse_utc_offset("+03:75"), None);
        assert_eq!(parse_utc_offset("abc"), None);
        assert_eq!(parse_utc_offset(""), None);
    }

    #[test]
    fn auto_offset_is_a_sane_value() {
        assert!(auto_offset_secs().abs() <= 14 * 3600);
    }
}
