# Translating the console

The console messages live in `crates/bv-server/lang/<code>.properties`, one `key=value` per line. Built-in languages: en, fa, ar, tr, ru, es, pt, fr, de, it, id, zh, ja, ko.

## Improve a language

1. Edit the values in `crates/bv-server/lang/<code>.properties`. Keep every key and every `{placeholder}` exactly as in `en.properties`; a placeholder is replaced by the server (`{name}`, `{seconds}`, ...).
2. Command names (`help`, `status`, `stop`) are not translated, only their descriptions.
3. Run `cd crates/bv-server && cargo test`. The test `every_language_has_the_keys_and_placeholders_of_english` fails when a key or a placeholder is missing or different.

## Add a language

1. Copy `en.properties` to `lang/<code>.properties` (use the two-letter ISO 639-1 code) and translate the values.
2. In `crates/bv-server/src/lang.rs` add an `Entry` to `LANGUAGES` (code, native name, aliases, the `include_str!` line) and raise the array length. Keep `en` first and `fa` second.
3. Add the code to the comment of `language=` in `DEFAULT_FILE` in `config.rs` and to the lists in `README.md` and `docs/CONFIGURATION.md`.
4. Run `cargo test`.

Missing keys fall back to English at run time, but the test requires every key, so a language is never half done.

## Notes

- Languages are written by the project and need review by native speakers. Open a pull request with corrections.
- Right-to-left languages (fa, ar) depend on the terminal. Windows Terminal, Termux and most Linux terminals show them; the old Windows console may not.
