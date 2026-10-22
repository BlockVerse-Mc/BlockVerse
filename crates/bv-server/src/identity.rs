const MAX_NAME_CHARS: usize = 32;
const MAX_INPUT_BYTES: usize = 65_536;

// Base64url
pub fn base64url_decode(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;
    for byte in input.bytes() {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'-' | b'+' => 62,
            b'_' | b'/' => 63,
            b'=' => break,
            _ => return None,
        } as u32;
        buffer = (buffer << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
            buffer &= (1u32 << bits) - 1;
        }
    }
    Some(out)
}

// JSON strings
fn read_json_string(body: &str) -> String {
    let mut out = String::new();
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => break,
            '\\' => match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('u') => {
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Some(ch) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                        out.push(ch);
                    }
                }
                Some(other) => out.push(other),
                None => break,
            },
            _ => out.push(c),
        }
    }
    out
}

pub fn json_string_value(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let mut from = 0;
    while let Some(pos) = json[from..].find(&needle) {
        let after = from + pos + needle.len();
        if let Some(rest) = json[after..].trim_start().strip_prefix(':') {
            if let Some(body) = rest.trim_start().strip_prefix('"') {
                return Some(read_json_string(body));
            }
        }
        from = after;
    }
    None
}

// Names
pub fn sanitize_name(raw: &str) -> Option<String> {
    let cleaned: String = raw.chars().filter(|c| !c.is_control()).take(MAX_NAME_CHARS).collect();
    let cleaned = cleaned.trim().to_string();
    if cleaned.is_empty() { None } else { Some(cleaned) }
}

pub fn extract_display_name(data: &[u8]) -> Option<String> {
    let limit = data.len().min(MAX_INPUT_BYTES);
    let text = String::from_utf8_lossy(&data[..limit]);
    let payloads: Vec<String> = text
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.'))
        .filter_map(|token| {
            let mut parts = token.split('.');
            let (_, payload) = (parts.next()?, parts.next()?);
            let bytes = base64url_decode(payload)?;
            Some(String::from_utf8_lossy(&bytes).into_owned())
        })
        .collect();
    for key in ["displayName", "ThirdPartyName"] {
        for payload in &payloads {
            if let Some(name) = json_string_value(payload, key).and_then(|n| sanitize_name(&n)) {
                return Some(name);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAIN: &str = "eyJleHRyYURhdGEiOnsiZGlzcGxheU5hbWUiOiJTdGV2ZSIsImlkZW50aXR5IjoiYWJjIiwiWFVJRCI6IjEyMyJ9LCJpYXQiOjF9";
    const CLIENT: &str = "eyJUaGlyZFBhcnR5TmFtZSI6IkFsZXhfOTkiLCJTZWxmU2lnbmVkSWQiOiJ4In0";
    const ESCAPED: &str = "eyJleHRyYURhdGEiOnsiZGlzcGxheU5hbWUiOiJTdGVcInZlIFx1MDBlOSJ9fQ";

    fn blob(chain_payload: &str, client_payload: &str) -> Vec<u8> {
        format!(
            "\x20\0\0\0{{\"chain\":[\"head.{chain_payload}.sig\"]}}\x30\0\0\0head2.{client_payload}.sig2"
        )
        .into_bytes()
    }

    #[test]
    fn base64url_decodes_unpadded_input() {
        assert_eq!(base64url_decode("eyJhIjoxfQ").unwrap(), b"{\"a\":1}");
        assert_eq!(base64url_decode("").unwrap(), Vec::<u8>::new());
        assert!(base64url_decode("ab$d").is_none());
    }

    #[test]
    fn display_name_comes_from_the_chain() {
        assert_eq!(extract_display_name(&blob(CHAIN, CLIENT)), Some("Steve".to_string()));
    }

    #[test]
    fn third_party_name_is_the_fallback() {
        let data = format!("\x01\0\0\0head2.{CLIENT}.sig2").into_bytes();
        assert_eq!(extract_display_name(&data), Some("Alex_99".to_string()));
    }

    #[test]
    fn escaped_characters_are_decoded() {
        let name = extract_display_name(&blob(ESCAPED, CLIENT)).unwrap();
        assert_eq!(name, "Ste\"ve \u{e9}");
    }

    #[test]
    fn garbage_gives_no_name() {
        assert_eq!(extract_display_name(b""), None);
        assert_eq!(extract_display_name(b"no tokens here"), None);
        assert_eq!(extract_display_name(&[0xff, 0xfe, 0x00, 0x41]), None);
        assert_eq!(extract_display_name(b"a.b.c d.e.f"), None);
    }

    #[test]
    fn names_are_cleaned_and_limited() {
        assert_eq!(sanitize_name("  Ste\u{7}ve  "), Some("Steve".to_string()));
        assert_eq!(sanitize_name(&"a".repeat(40)).unwrap().chars().count(), 32);
        assert_eq!(sanitize_name("   "), None);
        assert_eq!(sanitize_name(""), None);
    }

    #[test]
    fn json_lookup_ignores_non_string_values() {
        assert_eq!(json_string_value("{\"displayName\":5,\"displayName\":\"Z\"}", "displayName"), Some("Z".to_string()));
        assert_eq!(json_string_value("{\"x\":\"y\"}", "displayName"), None);
    }
}
