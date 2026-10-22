use std::io::{self, BufRead, IsTerminal, Write};

use crate::config;
use crate::lang::{self, Language};

const FILE: &str = "server.properties";

pub struct Answers {
    pub server_name: String,
    pub language: Language,
}

// Answers
fn read_answer(input: &mut dyn BufRead) -> Option<String> {
    let mut line = String::new();
    match input.read_line(&mut line) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(line.trim().to_string()),
    }
}

fn choose_language(text: &str) -> Option<Language> {
    let text = text.trim();
    if text.is_empty() {
        return Some(Language::En);
    }
    if let Ok(number) = text.parse::<usize>() {
        return lang::all()
            .get(number.wrapping_sub(1))
            .and_then(|(code, _)| lang::parse(code));
    }
    lang::parse(text)
}

pub fn ask(input: &mut dyn BufRead, output: &mut dyn Write) -> Answers {
    let _ = writeln!(output, "{}", lang::text_in(Language::En, "setup.title", &[]));
    for (index, (code, name)) in lang::all().iter().enumerate() {
        let _ = writeln!(output, "  {:>2}) {:<3} {}", index + 1, code, name);
    }
    let mut language = Language::En;
    for _ in 0..3 {
        let prompt = lang::text_in(Language::En, "setup.language", &[]);
        let _ = write!(output, "{prompt} [en]: ");
        let _ = output.flush();
        let Some(answer) = read_answer(input) else {
            break;
        };
        if let Some(chosen) = choose_language(&answer) {
            language = chosen;
            break;
        }
    }
    let prompt = lang::text_in(language, "setup.name", &[]);
    let _ = write!(output, "{prompt} [BlockVerse]: ");
    let _ = output.flush();
    let server_name = read_answer(input).unwrap_or_default();
    Answers { server_name, language }
}

// First start
pub fn first_run() {
    if std::env::var_os("BV_NO_SETUP").is_some() || config::settings_exist() {
        return;
    }
    let stdin = io::stdin();
    if !stdin.is_terminal() || !io::stdout().is_terminal() {
        return;
    }
    let mut input = stdin.lock();
    let mut output = io::stdout();
    let answers = ask(&mut input, &mut output);
    let text = config::default_file_with(Some(answers.server_name.as_str()), Some(lang::code(answers.language)));
    match std::fs::write(FILE, text) {
        Ok(()) => {
            let saved = lang::text_in(answers.language, "setup.saved", &[("file", FILE.to_string())]);
            let _ = writeln!(output, "{saved}");
        }
        Err(e) => eprintln!("Cannot write {FILE}: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn run(input: &str) -> (Answers, String) {
        let mut reader = Cursor::new(input.as_bytes().to_vec());
        let mut out: Vec<u8> = Vec::new();
        let answers = ask(&mut reader, &mut out);
        (answers, String::from_utf8(out).unwrap())
    }

    #[test]
    fn enter_keeps_the_defaults() {
        let (answers, _) = run("\n\n");
        assert_eq!(answers.language, Language::En);
        assert_eq!(answers.server_name, "");
    }

    #[test]
    fn a_number_or_a_code_selects_the_language() {
        let (answers, output) = run("2\nStars\n");
        assert_eq!(answers.language, Language::Fa);
        assert_eq!(answers.server_name, "Stars");
        assert!(output.contains("1) en"), "{output}");
        let (answers, _) = run("de\nX\n");
        assert_eq!(lang::code(answers.language), "de");
    }

    #[test]
    fn an_unknown_language_is_asked_again_and_then_english_is_used() {
        let (answers, _) = run("xx\n99\n0\nName\n");
        assert_eq!(answers.language, Language::En);
        assert_eq!(answers.server_name, "Name");
    }

    #[test]
    fn closed_input_gives_the_defaults() {
        let (answers, _) = run("");
        assert_eq!(answers.language, Language::En);
        assert_eq!(answers.server_name, "");
    }
}
