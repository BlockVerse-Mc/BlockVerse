#![allow(dead_code)]

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Php,
    Rust,
    Cpp,
    Native,
}

pub fn parse_language(name: &str) -> Option<Language> {
    match name.trim().to_ascii_lowercase().as_str() {
        "php" => Some(Language::Php),
        "rust" => Some(Language::Rust),
        "cpp" | "c++" => Some(Language::Cpp),
        "native" => Some(Language::Native),
        _ => None,
    }
}

fn extension(main: &str) -> String {
    Path::new(main)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn is_native_library(ext: &str) -> bool {
    matches!(ext, "so" | "dll" | "dylib")
}

pub fn detect_language(declared: Option<&str>, main: &str) -> Result<Language, String> {
    let ext = extension(main);
    if ext == "rs" {
        return Err(format!(
            "'{main}' is Rust source code and cannot be loaded; compile it to a native library first"
        ));
    }
    match declared {
        Some(name) => {
            let language = parse_language(name).ok_or_else(|| format!("unknown language '{name}'"))?;
            let fits = match language {
                Language::Php => ext == "php",
                Language::Rust | Language::Cpp | Language::Native => is_native_library(&ext),
            };
            if fits {
                Ok(language)
            } else {
                Err(format!("language '{name}' does not match main file '{main}'"))
            }
        }
        None => match ext.as_str() {
            "php" => Ok(Language::Php),
            e if is_native_library(e) => Ok(Language::Native),
            _ => Err(format!("cannot detect the language of '{main}'")),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_language_matching_the_file_is_accepted() {
        assert_eq!(detect_language(Some("php"), "src/main.php"), Ok(Language::Php));
        assert_eq!(detect_language(Some("rust"), "build/plugin.so"), Ok(Language::Rust));
        assert_eq!(detect_language(Some("C++"), "build/plugin.dll"), Ok(Language::Cpp));
    }

    #[test]
    fn mismatch_is_rejected_not_guessed() {
        assert!(detect_language(Some("php"), "build/plugin.so").is_err());
        assert!(detect_language(Some("rust"), "src/main.php").is_err());
    }

    #[test]
    fn rust_source_files_are_never_loadable() {
        assert!(detect_language(Some("rust"), "src/main.rs").is_err());
        assert!(detect_language(None, "src/main.rs").is_err());
    }

    #[test]
    fn language_is_detected_from_the_extension_when_missing() {
        assert_eq!(detect_language(None, "main.php"), Ok(Language::Php));
        assert_eq!(detect_language(None, "libplugin.so"), Ok(Language::Native));
        assert_eq!(detect_language(None, "plugin.DYLIB"), Ok(Language::Native));
        assert!(detect_language(None, "notes.txt").is_err());
    }

    #[test]
    fn unknown_language_names_are_rejected() {
        assert!(detect_language(Some("cobol"), "main.php").is_err());
    }
}
