use std::path::Path;

use crate::config;
use crate::console;

pub fn option_value(args: &[String], name: &str) -> Option<String> {
    let index = args.iter().position(|a| a == name)?;
    args.get(index + 1).cloned()
}

// Init
pub fn init_in(path: &Path, args: &[String]) -> i32 {
    let target = path.join("server.properties");
    if target.exists() {
        println!("server.properties already exists, it was not changed");
        return 0;
    }
    let name = option_value(args, "--server-name");
    let language = option_value(args, "--language");
    let text = config::default_file_with(name.as_deref(), language.as_deref());
    match std::fs::write(&target, text) {
        Ok(()) => {
            println!("Created {}", target.display());
            0
        }
        Err(e) => {
            eprintln!("Cannot write {}: {e}", target.display());
            1
        }
    }
}

const USAGE: &str = "\
Usage: blockverse [option]
  (no option)                 run the server in the current folder
  --version                   print the versions
  --init [--server-name N] [--language en|fa]
                              create server.properties in the current folder
  --help                      show this text";

// Options
pub fn handle(args: &[String]) -> Option<i32> {
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("{}", console::version_line());
        return Some(0);
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return Some(0);
    }
    if args.iter().any(|a| a == "--init") {
        return Some(init_in(Path::new("."), args));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("bv-cli-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn option_values_are_read() {
        let a = args(&["--init", "--server-name", "My Server", "--language", "fa"]);
        assert_eq!(option_value(&a, "--server-name"), Some("My Server".to_string()));
        assert_eq!(option_value(&a, "--language"), Some("fa".to_string()));
        assert_eq!(option_value(&a, "--missing"), None);
        assert_eq!(option_value(&args(&["--language"]), "--language"), None);
    }

    #[test]
    fn no_option_means_run_the_server() {
        assert_eq!(handle(&args(&[])), None);
        assert_eq!(handle(&args(&["something"])), None);
    }

    #[test]
    fn init_creates_a_parsable_file_with_the_answers() {
        let dir = temp_dir("init");
        let a = args(&["--init", "--server-name", "Stars Server", "--language", "fa"]);
        assert_eq!(init_in(&dir, &a), 0);
        let text = std::fs::read_to_string(dir.join("server.properties")).unwrap();
        let (cfg, warnings) = config::parse(&text);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(cfg.server_name, "Stars Server");
        assert_eq!(cfg.language, crate::lang::Language::Fa);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn init_never_overwrites_an_existing_file() {
        let dir = temp_dir("keep");
        std::fs::write(dir.join("server.properties"), "server-name=Mine\n").unwrap();
        assert_eq!(init_in(&dir, &args(&["--init", "--server-name", "Other"])), 0);
        let text = std::fs::read_to_string(dir.join("server.properties")).unwrap();
        assert_eq!(text, "server-name=Mine\n");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
