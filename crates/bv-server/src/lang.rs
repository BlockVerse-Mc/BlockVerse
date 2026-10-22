use std::sync::atomic::{AtomicU8, Ordering};

static CURRENT: AtomicU8 = AtomicU8::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    En,
    Fa,
}

// Selection
pub fn parse(text: &str) -> Option<Language> {
    match text.trim().to_ascii_lowercase().as_str() {
        "en" | "eng" | "english" => Some(Language::En),
        "fa" | "fas" | "persian" | "farsi" => Some(Language::Fa),
        _ => None,
    }
}

pub fn code(language: Language) -> &'static str {
    match language {
        Language::En => "en",
        Language::Fa => "fa",
    }
}

pub fn set(language: Language) {
    CURRENT.store(
        match language {
            Language::En => 0,
            Language::Fa => 1,
        },
        Ordering::Relaxed,
    );
}

pub fn current() -> Language {
    match CURRENT.load(Ordering::Relaxed) {
        1 => Language::Fa,
        _ => Language::En,
    }
}

fn pick(en: String, fa: String) -> String {
    match current() {
        Language::En => en,
        Language::Fa => fa,
    }
}

// Messages
pub fn starting(release: &str, version: &str, minecraft: &str, protocol: u32) -> String {
    pick(
        format!("Starting BlockVerse {release} ({version}) for Minecraft {minecraft} (protocol {protocol})"),
        format!("در حال راه‌اندازی BlockVerse {release} ({version}) برای ماینکرافت {minecraft} (پروتکل {protocol})"),
    )
}

pub fn offline_warning_1() -> String {
    pick(
        "Online mode is disabled. The server will not verify that players are authenticated to Xbox Live.".to_string(),
        "حالت آنلاین غیرفعاله. سرور هویت بازیکن‌ها رو با Xbox Live تأیید نمی‌کنه.".to_string(),
    )
}

pub fn offline_warning_2() -> String {
    pick(
        "This allows anyone to connect with any username they choose. Do not expose this server to the internet.".to_string(),
        "این یعنی هر کسی می‌تونه با هر اسمی وصل بشه. سرور رو روی اینترنت باز نذار.".to_string(),
    )
}

pub fn network_running(address: &str) -> String {
    pick(
        format!("BlockVerse network interface running on {address}"),
        format!("رابط شبکهٔ BlockVerse روی {address} در حال اجراست"),
    )
}

pub fn level_info(name: &str, seed: u64) -> String {
    pick(
        format!("Level '{name}': flat world, seed {seed}"),
        format!("جهان «{name}»: تخت، سید {seed}"),
    )
}

pub fn done(seconds: f64) -> String {
    pick(
        format!("Done ({seconds:.3}s)! For help, type \"help\" or \"?\""),
        format!("آماده شد ({seconds:.3} ثانیه)! برای راهنما بنویس help"),
    )
}

pub fn signal_stop() -> String {
    pick(
        "Received signal interrupt, stopping the server".to_string(),
        "سیگنال توقف دریافت شد، سرور در حال توقف است".to_string(),
    )
}

pub fn stop_requested() -> String {
    pick(
        "Stop requested from the console".to_string(),
        "درخواست توقف از کنسول رسید".to_string(),
    )
}

pub fn stopping() -> String {
    pick("Stopping other threads".to_string(), "در حال بستن بخش‌های دیگر".to_string())
}

pub fn goodbye() -> String {
    pick("Goodbye.".to_string(), "خداحافظ.".to_string())
}

pub fn player_joined(name: &str, id: u64, skin: &str) -> String {
    pick(
        format!("{name} joined (id {id}, default skin: {skin})"),
        format!("{name} وارد شد (شناسه {id}، اسکین پیش‌فرض: {skin})"),
    )
}

pub fn player_left(name: &str, id: u64) -> String {
    pick(format!("{name} left (id {id})"), format!("{name} خارج شد (شناسه {id})"))
}

const HELP_EN: &str = "\
Commands:
  help                   show this list
  version                show the versions
  status                 show uptime, players, packets and loop lag
  list                   show connected players
  plugins                show loaded plugins
  worlds                 show the world
  skins                  show the built-in default skins
  say <text>             announce a message (console only for now)
  debug                  show the log level
  debug level <level>    set the log level (error, warn, info, debug, trace)
  debug network          show connection and packet counters
  debug chunks           show chunk counters and the world test settings
  debug plugins          show loaded plugins
  save-all               not available yet (nothing is saved)
  fakejoin <name>        simulate a player joining (to test plugins)
  fakechat <id> <text>   simulate a chat message from a player
  fakequit <id>          simulate a player leaving
  selftest               run internal checks
  stop                   stop the server";

const HELP_FA: &str = "\
دستورات:
  help                   نمایش همین فهرست
  version                نمایش نسخه‌ها
  status                 نمایش زمان اجرا، بازیکن‌ها، بسته‌ها و تأخیر
  list                   نمایش بازیکن‌های وصل‌شده
  plugins                نمایش پلاگین‌های بارگذاری‌شده
  worlds                 نمایش جهان
  skins                  نمایش اسکین‌های پیش‌فرض
  say <متن>              اعلام پیام (فعلاً فقط در کنسول)
  debug                  نمایش سطح لاگ
  debug level <سطح>      تغییر سطح لاگ (error, warn, info, debug, trace)
  debug network          نمایش شمارنده‌های اتصال و بسته
  debug chunks           نمایش شمارنده‌های چانک و تنظیمات تست جهان
  debug plugins          نمایش پلاگین‌ها
  save-all               هنوز در دسترس نیست (چیزی ذخیره نمی‌شود)
  fakejoin <نام>         شبیه‌سازی ورود بازیکن (برای تست پلاگین)
  fakechat <id> <متن>    شبیه‌سازی پیام چت
  fakequit <id>          شبیه‌سازی خروج بازیکن
  selftest               اجرای بررسی‌های داخلی
  stop                   توقف سرور";

pub fn help_text() -> String {
    pick(HELP_EN.to_string(), HELP_FA.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn languages_are_parsed() {
        assert_eq!(parse("EN"), Some(Language::En));
        assert_eq!(parse(" persian "), Some(Language::Fa));
        assert_eq!(parse("fa"), Some(Language::Fa));
        assert_eq!(parse("de"), None);
        assert_eq!(code(Language::Fa), "fa");
    }

    #[test]
    fn messages_follow_the_selected_language() {
        set(Language::En);
        assert!(done(1.5).contains("Done (1.500s)"));
        assert!(player_joined("Steve", 1, "Black").contains("joined"));
        assert!(help_text().starts_with("Commands:"));
        set(Language::Fa);
        assert!(done(1.5).contains("آماده شد"));
        assert!(player_left("Steve", 2).contains("خارج شد"));
        assert!(help_text().starts_with("دستورات:"));
        for text in [
            starting("Beta", "0.1.0", "1.21.50", 766),
            offline_warning_1(),
            offline_warning_2(),
            network_running("127.0.0.1:19132"),
            level_info("world", 7),
            signal_stop(),
            stop_requested(),
            stopping(),
            goodbye(),
        ] {
            assert!(!text.is_empty());
        }
        set(Language::En);
        assert_eq!(current(), Language::En);
    }
}
