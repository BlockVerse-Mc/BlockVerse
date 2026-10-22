use std::sync::atomic::{AtomicUsize, Ordering};

struct Entry {
    code: &'static str,
    name: &'static str,
    aliases: &'static [&'static str],
    text: &'static str,
}

// Catalogs
static LANGUAGES: [Entry; 14] = [
    Entry {
        code: "en",
        name: "English",
        aliases: &["eng", "english"],
        text: include_str!("../lang/en.properties"),
    },
    Entry {
        code: "fa",
        name: "فارسی",
        aliases: &["fas", "per", "persian", "farsi"],
        text: include_str!("../lang/fa.properties"),
    },
    Entry {
        code: "ar",
        name: "العربية",
        aliases: &["ara", "arabic"],
        text: include_str!("../lang/ar.properties"),
    },
    Entry {
        code: "tr",
        name: "Türkçe",
        aliases: &["tur", "turkish", "türkçe", "turkce"],
        text: include_str!("../lang/tr.properties"),
    },
    Entry {
        code: "ru",
        name: "Русский",
        aliases: &["rus", "russian", "русский"],
        text: include_str!("../lang/ru.properties"),
    },
    Entry {
        code: "es",
        name: "Español",
        aliases: &["spa", "spanish", "español", "espanol"],
        text: include_str!("../lang/es.properties"),
    },
    Entry {
        code: "pt",
        name: "Português",
        aliases: &["por", "portuguese", "pt-br", "pt-pt", "português", "portugues"],
        text: include_str!("../lang/pt.properties"),
    },
    Entry {
        code: "fr",
        name: "Français",
        aliases: &["fra", "fre", "french", "français", "francais"],
        text: include_str!("../lang/fr.properties"),
    },
    Entry {
        code: "de",
        name: "Deutsch",
        aliases: &["deu", "ger", "german", "deutsch"],
        text: include_str!("../lang/de.properties"),
    },
    Entry {
        code: "it",
        name: "Italiano",
        aliases: &["ita", "italian", "italiano"],
        text: include_str!("../lang/it.properties"),
    },
    Entry {
        code: "id",
        name: "Bahasa Indonesia",
        aliases: &["ind", "indonesian", "bahasa"],
        text: include_str!("../lang/id.properties"),
    },
    Entry {
        code: "zh",
        name: "简体中文",
        aliases: &["zho", "chi", "chinese", "zh-cn", "zh-hans", "简体中文"],
        text: include_str!("../lang/zh.properties"),
    },
    Entry {
        code: "ja",
        name: "日本語",
        aliases: &["jpn", "japanese", "日本語"],
        text: include_str!("../lang/ja.properties"),
    },
    Entry {
        code: "ko",
        name: "한국어",
        aliases: &["kor", "korean", "한국어"],
        text: include_str!("../lang/ko.properties"),
    },
];

static CURRENT: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Language(usize);

#[allow(non_upper_case_globals, dead_code)]
impl Language {
    pub const En: Language = Language(0);
    pub const Fa: Language = Language(1);
}

// Selection
pub fn parse(text: &str) -> Option<Language> {
    let wanted = text.trim().to_lowercase().replace('_', "-");
    if wanted.is_empty() {
        return None;
    }
    LANGUAGES
        .iter()
        .position(|e| e.code == wanted || e.aliases.contains(&wanted.as_str()))
        .map(Language)
}

pub fn code(language: Language) -> &'static str {
    LANGUAGES[language.0].code
}

pub fn all() -> Vec<(&'static str, &'static str)> {
    LANGUAGES.iter().map(|e| (e.code, e.name)).collect()
}

pub fn set(language: Language) {
    CURRENT.store(language.0, Ordering::Relaxed);
}

pub fn current() -> Language {
    Language(CURRENT.load(Ordering::Relaxed))
}

// Lookup
fn lookup(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        if let Some((name, value)) = line.split_once('=') {
            if name.trim() == key {
                return Some(value.trim().to_string());
            }
        }
    }
    None
}

pub fn text_in(language: Language, key: &str, args: &[(&str, String)]) -> String {
    let mut text = lookup(LANGUAGES[language.0].text, key)
        .or_else(|| lookup(LANGUAGES[0].text, key))
        .unwrap_or_else(|| key.to_string());
    for (name, value) in args {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

fn tr(key: &str, args: &[(&str, String)]) -> String {
    text_in(current(), key, args)
}

// Messages
pub fn starting(release: &str, version: &str, minecraft: &str, protocol: u32) -> String {
    tr(
        "starting",
        &[
            ("release", release.to_string()),
            ("version", version.to_string()),
            ("minecraft", minecraft.to_string()),
            ("protocol", protocol.to_string()),
        ],
    )
}

pub fn offline_warning_1() -> String {
    tr("offline_warning_1", &[])
}

pub fn offline_warning_2() -> String {
    tr("offline_warning_2", &[])
}

pub fn network_running(address: &str) -> String {
    tr("network_running", &[("address", address.to_string())])
}

pub fn level_info(name: &str, seed: u64) -> String {
    tr("level_info", &[("name", name.to_string()), ("seed", seed.to_string())])
}

pub fn done(seconds: f64) -> String {
    tr("done", &[("seconds", format!("{seconds:.3}"))])
}

pub fn signal_stop() -> String {
    tr("signal_stop", &[])
}

pub fn stop_requested() -> String {
    tr("stop_requested", &[])
}

pub fn stopping() -> String {
    tr("stopping", &[])
}

pub fn goodbye() -> String {
    tr("goodbye", &[])
}

pub fn player_joined(name: &str, id: u64, skin: &str) -> String {
    tr(
        "player_joined",
        &[("name", name.to_string()), ("id", id.to_string()), ("skin", skin.to_string())],
    )
}

pub fn player_left(name: &str, id: u64) -> String {
    tr("player_left", &[("name", name.to_string()), ("id", id.to_string())])
}

// Help
const HELP_COMMANDS: [(&str, &str); 19] = [
    ("help", "help.help"),
    ("version", "help.version"),
    ("status", "help.status"),
    ("list", "help.list"),
    ("plugins", "help.plugins"),
    ("worlds", "help.worlds"),
    ("skins", "help.skins"),
    ("say <text>", "help.say"),
    ("debug", "help.debug"),
    ("debug level <level>", "help.debug_level"),
    ("debug network", "help.debug_network"),
    ("debug chunks", "help.debug_chunks"),
    ("debug plugins", "help.debug_plugins"),
    ("save-all", "help.save_all"),
    ("fakejoin <name>", "help.fakejoin"),
    ("fakechat <id> <text>", "help.fakechat"),
    ("fakequit <id>", "help.fakequit"),
    ("selftest", "help.selftest"),
    ("stop", "help.stop"),
];

pub fn help_text() -> String {
    let mut out = tr("help.title", &[]);
    for (syntax, key) in HELP_COMMANDS {
        out.push_str(&format!("\n  {syntax:<22} {}", tr(key, &[])));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(text: &str) -> Vec<&str> {
        text.lines().filter_map(|l| l.split_once('=')).map(|(k, _)| k.trim()).collect()
    }

    fn value_of<'a>(text: &'a str, key: &str) -> Option<&'a str> {
        text.lines()
            .filter_map(|l| l.split_once('='))
            .find(|(k, _)| k.trim() == key)
            .map(|(_, v)| v.trim())
    }

    fn placeholders(value: &str) -> Vec<String> {
        let mut found = Vec::new();
        let mut rest = value;
        while let Some(start) = rest.find('{') {
            match rest[start..].find('}') {
                Some(end) => {
                    found.push(rest[start + 1..start + end].to_string());
                    rest = &rest[start + end + 1..];
                }
                None => break,
            }
        }
        found.sort();
        found
    }

    #[test]
    fn languages_are_parsed() {
        assert_eq!(parse("EN"), Some(Language::En));
        assert_eq!(parse(" persian "), Some(Language::Fa));
        assert_eq!(parse("fa"), Some(Language::Fa));
        assert_eq!(code(Language::Fa), "fa");
        assert_eq!(parse("pt_BR").map(code), Some("pt"));
        assert_eq!(parse("Deutsch").map(code), Some("de"));
        assert_eq!(parse("xx"), None);
        assert_eq!(parse(""), None);
    }

    #[test]
    fn language_codes_are_unique_and_parse_to_themselves() {
        assert_eq!(LANGUAGES[0].code, "en");
        assert_eq!(LANGUAGES[1].code, "fa");
        for (index, entry) in LANGUAGES.iter().enumerate() {
            assert_eq!(parse(entry.code), Some(Language(index)), "{}", entry.code);
            assert_eq!(LANGUAGES.iter().filter(|e| e.code == entry.code).count(), 1);
            assert!(!entry.name.is_empty());
        }
        assert_eq!(all().len(), LANGUAGES.len());
    }

    #[test]
    fn every_language_has_the_keys_and_placeholders_of_english() {
        let english = LANGUAGES[0].text;
        let mut expected = keys(english);
        expected.sort();
        assert!(expected.len() >= 36);
        for entry in &LANGUAGES {
            let mut own = keys(entry.text);
            own.sort();
            assert_eq!(own, expected, "keys of {}", entry.code);
            for key in &expected {
                let reference = value_of(english, key).unwrap();
                let value = value_of(entry.text, key).unwrap();
                assert!(!value.is_empty(), "{} has an empty {key}", entry.code);
                assert_eq!(placeholders(value), placeholders(reference), "{} {key}", entry.code);
            }
        }
    }

    #[test]
    fn the_selected_language_changes_the_messages() {
        set(Language::En);
        assert!(done(1.5).contains("Done (1.500s)"));
        assert!(player_joined("Steve", 1, "Black").contains("joined"));
        assert!(help_text().starts_with("Commands:"));
        assert!(help_text().contains("stop"));
        set(Language::Fa);
        assert!(done(1.5).contains("آماده شد"));
        assert!(player_left("Steve", 2).contains("خارج شد"));
        assert!(help_text().starts_with("دستورات:"));
        set(parse("de").unwrap());
        assert!(done(2.0).contains("Fertig (2.000 s)"));
        for entry_index in 0..LANGUAGES.len() {
            set(Language(entry_index));
            for text in [
                starting("Release", "1.0.0", "1.21.50", 766),
                offline_warning_1(),
                offline_warning_2(),
                network_running("127.0.0.1:19132"),
                level_info("world", 7),
                done(0.5),
                signal_stop(),
                stop_requested(),
                stopping(),
                goodbye(),
                player_joined("Steve", 1, "Black"),
                player_left("Steve", 1),
                help_text(),
            ] {
                assert!(!text.is_empty());
                assert!(!text.contains('{') && !text.contains('}'), "{text}");
            }
        }
        set(Language::En);
        assert_eq!(current(), Language::En);
    }

    #[test]
    fn a_missing_key_falls_back_to_english_then_to_the_key() {
        assert_eq!(text_in(Language::Fa, "no.such.key", &[]), "no.such.key");
        assert_eq!(text_in(Language::En, "goodbye", &[]), "Goodbye.");
    }
}
