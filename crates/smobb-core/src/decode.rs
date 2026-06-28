//! Hexadecimal → ASCII/UTF-8 decoding for Jaffar's cipher lines.

/// Decode an even-length all-hex string to printable UTF-8, else None.
pub fn decode_hex(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() || s.len() % 2 != 0 {
        return None;
    }
    if !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    let sb = s.as_bytes();
    let mut i = 0;
    while i < sb.len() {
        let hi = (sb[i] as char).to_digit(16)? as u8;
        let lo = (sb[i + 1] as char).to_digit(16)? as u8;
        bytes.push((hi << 4) | lo);
        i += 2;
    }
    let text = String::from_utf8(bytes).ok()?;
    if text
        .chars()
        .all(|c| !c.is_control() || c == '\n' || c == '\t')
    {
        Some(text)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decodes_valid_hex() {
        assert_eq!(decode_hex("48656C6C6F").as_deref(), Some("Hello"));
        // real Jaffar line: "NO! HOW DID YOU KNOW?!"
        assert_eq!(
            decode_hex("4E4F2120484F572044494420594F55204B4E4F573F21").as_deref(),
            Some("NO! HOW DID YOU KNOW?!")
        );
    }
    #[test]
    fn rejects_non_hex() {
        assert_eq!(decode_hex("hello world"), None); // spaces + non-hex
        assert_eq!(decode_hex("ABC"), None); // odd length
        assert_eq!(decode_hex(""), None); // empty
        assert_eq!(decode_hex("00"), None); // NUL control char -> not printable
    }
}
