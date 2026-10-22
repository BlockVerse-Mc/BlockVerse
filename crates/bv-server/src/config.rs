use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;

use crate::lang::{self, Language};
use crate::logger::{self, Level};

const FILE: &str = "server.properties";
const LEGACY_FILE: &str = "blockverse.properties";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionLevel {
    Visitor,
    Member,
    Operator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelType {
    Flat,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub server_ip: IpAddr,
    pub server_port: u16,
    pub server_name: String,
    pub motd: String,
    pub language: Language,
    pub gamemode: GameMode,
    pub difficulty: Difficulty,
    pub max_players: i32,
    pub level_name: String,
    pub level_seed: Option<u64>,
    pub level_type: LevelType,
    pub view_distance: i32,
    pub online_mode: bool,
    pub allow_cheats: bool,
    pub default_permission: PermissionLevel,
    pub max_threads: usize,
    pub log_level: Level,
    pub plugin_dir: String,
    pub login_timeout_secs: u64,
    pub max_pending_logins: usize,
    pub max_packets_per_second: u64,
    pub utc_offset_secs: Option<i64>,
    pub world_stage: u8,
    pub chunk_variant: Option<u8>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
            server_port: 19132,
            server_name: "BlockVerse".to_string(),
            motd: "BlockVerse".to_string(),
            language: Language::En,
            gamemode: GameMode::Creative,
            difficulty: Difficulty::Peaceful,
            max_players: 20,
            level_name: "world".to_string(),
            level_seed: None,
            level_type: LevelType::Flat,
            view_distance: 4,
            online_mode: false,
            allow_cheats: true,
            default_permission: PermissionLevel::Member,
            max_threads: 0,
            log_level: Level::Info,
            plugin_dir: "plugins".to_string(),
            login_timeout_secs: 15,
            max_pending_logins: 32,
            max_packets_per_second: 2000,
            utc_offset_secs: None,
            world_stage: 3,
            chunk_variant: None,
        }
    }
}

impl Config {
    pub fn bind(&self) -> SocketAddr {
        SocketAddr::new(self.server_ip, self.server_port)
    }
}

pub const DEFAULT_FILE: &str = "\
# BlockVerse server settings (Bedrock 1.21.50, protocol 766)
# Lines starting with # are comments. Invalid values are reported and the default is used.

# Address and port to listen on. 127.0.0.1 accepts only this device; use 0.0.0.0 for your network.
server-ip=127.0.0.1
server-port=19132

# Server list name and second line
server-name=BlockVerse
motd=BlockVerse

# Language of the console messages: en or fa
language=en

# survival, creative or adventure
gamemode=creative
# peaceful, easy, normal or hard
difficulty=peaceful
max-players=20

# World name (letters, digits, space, dash, underscore, dot)
level-name=world
# Number or text; empty picks a random seed at every start
level-seed=
# Only flat is available at the moment
level-type=flat
# Chunks around the player (1 to 16)
view-distance=4

# Must stay false: identity verification is not implemented yet
online-mode=false
# Allow commands in game
allow-cheats=true
# visitor, member or operator
default-player-permission-level=member

# Worker threads of the server runtime (0 = automatic)
max-threads=0
# error, warn, info, debug or trace
log-level=info
# Time zone of log timestamps: auto or an offset like +03:30
utc-offset=auto

# Folder with native plugins (.so / .dll / .dylib)
plugin-dir=plugins

# Seconds a client has to finish logging in
login-timeout=15
# Logins that may run at the same time (1 to 1000)
max-pending-logins=32
# Packets one player may send per second before being disconnected (10 to 100000)
max-packets-per-second=2000

# Testing only. World sending stage: 0 radius only, 1 plus publisher update, 2 plus one chunk, 3 everything
world-stage=3
# Testing only. Chunk format variant: auto (a new variant for every connection) or 0 to 3
chunk-variant=auto
";

// Value parsers
fn parse_bool(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

pub fn parse_gamemode(value: &str) -> Option<GameMode> {
    match value.to_ascii_lowercase().as_str() {
        "survival" | "0" => Some(GameMode::Survival),
        "creative" | "1" => Some(GameMode::Creative),
        "adventure" | "2" => Some(GameMode::Adventure),
        _ => None,
    }
}

pub fn parse_difficulty(value: &str) -> Option<Difficulty> {
    match value.to_ascii_lowercase().as_str() {
        "peaceful" | "0" => Some(Difficulty::Peaceful),
        "easy" | "1" => Some(Difficulty::Easy),
        "normal" | "2" => Some(Difficulty::Normal),
        "hard" | "3" => Some(Difficulty::Hard),
        _ => None,
    }
}

pub fn parse_permission(value: &str) -> Option<PermissionLevel> {
    match value.to_ascii_lowercase().as_str() {
        "visitor" => Some(PermissionLevel::Visitor),
        "member" => Some(PermissionLevel::Member),
        "operator" => Some(PermissionLevel::Operator),
        _ => None,
    }
}

pub fn valid_level_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().count() <= 64
        && name != "."
        && name != ".."
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.'))
}

pub fn seed_from_text(text: &str) -> Option<u64> {
    let t = text.trim();
    if t.is_empty() {
        return None;
    }
    if let Ok(n) = t.parse::<i64>() {
        return Some(n as u64);
    }
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in t.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    Some(hash)
}

// Initial file
pub fn default_file_with(server_name: Option<&str>, language: Option<&str>) -> String {
    let mut text = DEFAULT_FILE.to_string();
    if let Some(name) = server_name {
        let cleaned: String = name.chars().filter(|c| !c.is_control()).collect();
        let cleaned = cleaned.trim();
        if !cleaned.is_empty() {
            text = text.replace("server-name=BlockVerse\n", &format!("server-name={cleaned}\n"));
        }
    }
    if let Some(l) = language.and_then(lang::parse) {
        text = text.replace("language=en\n", &format!("language={}\n", lang::code(l)));
    }
    text
}

// Parsing
pub fn parse(text: &str) -> (Config, Vec<String>) {
    let mut cfg = Config::default();
    let mut warnings = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            warnings.push(format!("line {}: expected key=value", i + 1));
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        let mut bad = |what: &str| warnings.push(format!("invalid {key} '{value}' ({what})"));
        match key {
            "server-ip" => match value.parse::<IpAddr>() {
                Ok(ip) => cfg.server_ip = ip,
                Err(_) => bad("use an IP address such as 127.0.0.1"),
            },
            "server-port" => match value.parse::<u16>() {
                Ok(p) if p > 0 => cfg.server_port = p,
                _ => bad("use 1 to 65535"),
            },
            "bind" => match value.parse::<SocketAddr>() {
                Ok(addr) => {
                    cfg.server_ip = addr.ip();
                    cfg.server_port = addr.port();
                    warnings.push("'bind' is deprecated, use server-ip and server-port".to_string());
                }
                Err(_) => bad("use ip:port"),
            },
            "server-name" => cfg.server_name = value.to_string(),
            "motd" => cfg.motd = value.to_string(),
            "language" => match lang::parse(value) {
                Some(l) => cfg.language = l,
                None => bad("use en or fa"),
            },
            "gamemode" => match parse_gamemode(value) {
                Some(m) => cfg.gamemode = m,
                None => bad("use survival, creative or adventure"),
            },
            "difficulty" => match parse_difficulty(value) {
                Some(d) => cfg.difficulty = d,
                None => bad("use peaceful, easy, normal or hard"),
            },
            "max-players" => match value.parse::<i32>() {
                Ok(n) if n > 0 => cfg.max_players = n,
                _ => bad("use a number above 0"),
            },
            "level-name" => {
                if valid_level_name(value) {
                    cfg.level_name = value.to_string();
                } else {
                    bad("use letters, digits, space, dash, underscore or dot, at most 64 characters");
                }
            }
            "level-seed" => cfg.level_seed = seed_from_text(value),
            "level-type" => {
                if value.eq_ignore_ascii_case("flat") {
                    cfg.level_type = LevelType::Flat;
                } else {
                    bad("only flat is available");
                }
            }
            "view-distance" => match value.parse::<i32>() {
                Ok(n) if (1..=16).contains(&n) => cfg.view_distance = n,
                _ => bad("use 1 to 16"),
            },
            "online-mode" => match parse_bool(value) {
                Some(b) => cfg.online_mode = b,
                None => bad("use true or false"),
            },
            "allow-cheats" => match parse_bool(value) {
                Some(b) => cfg.allow_cheats = b,
                None => bad("use true or false"),
            },
            "default-player-permission-level" => match parse_permission(value) {
                Some(p) => cfg.default_permission = p,
                None => bad("use visitor, member or operator"),
            },
            "max-threads" => match value.parse::<usize>() {
                Ok(n) if n <= 256 => cfg.max_threads = n,
                _ => bad("use 0 to 256"),
            },
            "log-level" => match logger::parse_level(value) {
                Some(l) => cfg.log_level = l,
                None => bad("use error, warn, info, debug or trace"),
            },
            "debug" => match parse_bool(value) {
                Some(true) => {
                    cfg.log_level = Level::Debug;
                    warnings.push("'debug' is deprecated, use log-level=debug".to_string());
                }
                Some(false) => {}
                None => bad("use true or false"),
            },
            "utc-offset" => {
                if value.eq_ignore_ascii_case("auto") {
                    cfg.utc_offset_secs = None;
                } else if let Some(secs) = logger::parse_utc_offset(value) {
                    cfg.utc_offset_secs = Some(secs);
                } else {
                    bad("use auto or an offset like +03:30");
                }
            }
            "plugin-dir" => cfg.plugin_dir = value.to_string(),
            "login-timeout" => match value.parse::<u64>() {
                Ok(n) if n > 0 => cfg.login_timeout_secs = n,
                _ => bad("use a number above 0"),
            },
            "max-pending-logins" => match value.parse::<usize>() {
                Ok(n) if (1..=1000).contains(&n) => cfg.max_pending_logins = n,
                _ => bad("use 1 to 1000"),
            },
            "max-packets-per-second" => match value.parse::<u64>() {
                Ok(n) if (10..=100_000).contains(&n) => cfg.max_packets_per_second = n,
                _ => bad("use 10 to 100000"),
            },
            "world-stage" => match value.parse::<u8>() {
                Ok(n) if n <= 3 => cfg.world_stage = n,
                _ => bad("use 0 to 3"),
            },
            "chunk-variant" => {
                if value.eq_ignore_ascii_case("auto") {
                    cfg.chunk_variant = None;
                } else {
                    match value.parse::<u8>() {
                        Ok(n) if n <= 3 => cfg.chunk_variant = Some(n),
                        _ => bad("use auto or 0 to 3"),
                    }
                }
            }
            _ => warnings.push(format!("unknown setting '{key}' (ignored)")),
        }
    }
    (cfg, warnings)
}

// Loading
pub fn load(path: &str) -> (Config, Vec<String>) {
    match std::fs::read_to_string(path) {
        Ok(text) => parse(&text),
        Err(_) => {
            let mut warnings = Vec::new();
            if let Err(e) = std::fs::write(path, DEFAULT_FILE) {
                warnings.push(format!("could not write default {path}: {e}"));
            }
            (Config::default(), warnings)
        }
    }
}

pub fn load_settings() -> (Config, Vec<String>) {
    if !Path::new(FILE).exists() && Path::new(LEGACY_FILE).exists() {
        let (cfg, mut warnings) = load(LEGACY_FILE);
        warnings.push(format!("{LEGACY_FILE} is deprecated: copy its values to {FILE}"));
        return (cfg, warnings);
    }
    load(FILE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_file_parses_to_defaults_without_warnings() {
        let (cfg, warnings) = parse(DEFAULT_FILE);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn custom_values_are_applied() {
        let text = "server-ip=0.0.0.0\nserver-port=19133\nserver-name = My Server\ngamemode=survival\n\
                    difficulty=hard\nmax-players=5\nlevel-name=My World\nlevel-seed=12345\nview-distance=6\n\
                    allow-cheats=false\ndefault-player-permission-level=operator\nmax-threads=2\n\
                    log-level=trace\nutc-offset=+03:30\nmax-pending-logins=8\nmax-packets-per-second=500\n\
                    world-stage=1\nchunk-variant=2\n";
        let (cfg, warnings) = parse(text);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(cfg.bind(), "0.0.0.0:19133".parse::<SocketAddr>().unwrap());
        assert_eq!(cfg.server_name, "My Server");
        assert_eq!(cfg.gamemode, GameMode::Survival);
        assert_eq!(cfg.difficulty, Difficulty::Hard);
        assert_eq!(cfg.max_players, 5);
        assert_eq!(cfg.level_name, "My World");
        assert_eq!(cfg.level_seed, Some(12345));
        assert_eq!(cfg.view_distance, 6);
        assert!(!cfg.allow_cheats);
        assert_eq!(cfg.default_permission, PermissionLevel::Operator);
        assert_eq!(cfg.max_threads, 2);
        assert_eq!(cfg.log_level, Level::Trace);
        assert_eq!(cfg.utc_offset_secs, Some(12_600));
        assert_eq!(cfg.max_pending_logins, 8);
        assert_eq!(cfg.max_packets_per_second, 500);
        assert_eq!(cfg.world_stage, 1);
        assert_eq!(cfg.chunk_variant, Some(2));
    }

    #[test]
    fn invalid_values_warn_and_keep_defaults() {
        let text = "server-ip=nope\nserver-port=0\ngamemode=hardcore\ndifficulty=insane\nmax-players=0\n\
                    language=de\nlevel-name=../escape\nlevel-type=normal\nview-distance=99\nonline-mode=maybe\n\
                    allow-cheats=yes\ndefault-player-permission-level=god\nmax-threads=999\nlog-level=loud\n\
                    utc-offset=xyz\nlogin-timeout=0\nmax-pending-logins=0\nmax-packets-per-second=5\n\
                    world-stage=9\nchunk-variant=7\nfoo=1\njunk\n";
        let (cfg, warnings) = parse(text);
        assert_eq!(warnings.len(), 22, "{warnings:?}");
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn the_installer_file_carries_the_chosen_name_and_language() {
        let text = default_file_with(Some("  My\tServer \n"), Some("FA"));
        let (cfg, warnings) = parse(&text);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(cfg.server_name, "MyServer");
        assert_eq!(cfg.language, Language::Fa);
        assert_eq!(default_file_with(None, None), DEFAULT_FILE);
        assert_eq!(default_file_with(Some("   "), Some("xx")), DEFAULT_FILE);
    }

    #[test]
    fn legacy_keys_still_work_with_a_warning() {
        let (cfg, warnings) = parse("bind=0.0.0.0:19140\ndebug=true\n");
        assert_eq!(cfg.bind(), "0.0.0.0:19140".parse::<SocketAddr>().unwrap());
        assert_eq!(cfg.log_level, Level::Debug);
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn seeds_are_numbers_or_hashed_text() {
        assert_eq!(seed_from_text(""), None);
        assert_eq!(seed_from_text("42"), Some(42));
        assert_eq!(seed_from_text("-1"), Some(u64::MAX));
        assert_eq!(seed_from_text("hello"), seed_from_text(" hello "));
        assert_ne!(seed_from_text("hello"), seed_from_text("world"));
    }

    #[test]
    fn level_names_cannot_escape_the_folder() {
        assert!(valid_level_name("world"));
        assert!(valid_level_name("My World-2.0"));
        assert!(!valid_level_name(""));
        assert!(!valid_level_name(".."));
        assert!(!valid_level_name("a/b"));
        assert!(!valid_level_name("a\\b"));
        assert!(!valid_level_name(&"x".repeat(65)));
    }

    #[test]
    fn game_modes_accept_names_and_numbers() {
        assert_eq!(parse_gamemode("Creative"), Some(GameMode::Creative));
        assert_eq!(parse_gamemode("2"), Some(GameMode::Adventure));
        assert_eq!(parse_difficulty("HARD"), Some(Difficulty::Hard));
        assert_eq!(parse_difficulty("5"), None);
        assert_eq!(parse_permission("visitor"), Some(PermissionLevel::Visitor));
    }
}
