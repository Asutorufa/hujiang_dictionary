use std::borrow::Cow;

pub const MARKDOWN_ESCAPE_CHARS: [char; 19] = [
    '\\', '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!',
];

pub fn split_message(text: &str, max_len: usize) -> Vec<String> {
    let mut chunks = Vec::with_capacity(text.len() / max_len.max(1) + 1);
    let mut start = 0;

    for (idx, c) in text.char_indices() {
        if idx - start + c.len_utf8() > max_len && idx > start {
            chunks.push(text[start..idx].to_string());
            start = idx;
        }
    }

    if start < text.len() {
        chunks.push(text[start..].to_string());
    }

    chunks
}

pub fn markdown_escape(s: &str) -> Cow<'_, str> {
    if s.chars().any(|c| MARKDOWN_ESCAPE_CHARS.contains(&c)) {
        let mut result = String::with_capacity(s.len() + 10);
        for c in s.chars() {
            if MARKDOWN_ESCAPE_CHARS.contains(&c) {
                result.push('\\');
            }
            result.push(c);
        }
        Cow::Owned(result)
    } else {
        Cow::Borrowed(s)
    }
}

pub fn html_escape(s: &str) -> Cow<'_, str> {
    if s.chars().any(|c| matches!(c, '&' | '<' | '>')) {
        let mut result = String::with_capacity(s.len() + 10);
        for c in s.chars() {
            match c {
                '&' => result.push_str("&amp;"),
                '<' => result.push_str("&lt;"),
                '>' => result.push_str("&gt;"),
                c => result.push(c),
            }
        }
        Cow::Owned(result)
    } else {
        Cow::Borrowed(s)
    }
}

pub fn vec_string_markdown_escape(v: &[String]) -> String {
    let mut s = String::new();
    for i in v {
        s.push_str(&markdown_escape(i.as_str()));
        s.push('\n');
    }
    s
}

pub fn utf16_slice(text: &str, start_utf16: usize, len_utf16: usize) -> Option<&str> {
    let mut current_utf16_idx = 0;
    let mut start_byte = None;
    let mut end_byte = None;

    for (byte_idx, c) in text.char_indices() {
        if current_utf16_idx == start_utf16 {
            start_byte = Some(byte_idx);
        }
        if current_utf16_idx == start_utf16 + len_utf16 {
            end_byte = Some(byte_idx);
            break;
        }
        current_utf16_idx += c.len_utf16();
    }

    if start_byte.is_some() && end_byte.is_none() && current_utf16_idx == start_utf16 + len_utf16 {
        end_byte = Some(text.len());
    }

    if let (Some(start), Some(end)) = (start_byte, end_byte) {
        Some(&text[start..end])
    } else {
        None
    }
}

pub fn utf16_slice_from(text: &str, start_utf16: usize) -> Option<&str> {
    let mut current_utf16_idx = 0;

    for (byte_idx, c) in text.char_indices() {
        if current_utf16_idx == start_utf16 {
            return Some(&text[byte_idx..]);
        }
        current_utf16_idx += c.len_utf16();
    }

    if current_utf16_idx == start_utf16 {
        return Some(&text[text.len()..]);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_message() {
        let text = "hello world";
        let chunks = split_message(text, 5);
        assert_eq!(chunks, vec!["hello", " worl", "d"]);

        let text = "你好世界";
        let chunks = split_message(text, 6);
        assert_eq!(chunks, vec!["你好", "世界"]);

        let text = "嗨";
        let chunks = split_message(text, 2);
        assert_eq!(chunks, vec!["嗨"]);
    }

    #[test]
    fn test_escape() {
        assert_eq!(markdown_escape("hello_world"), "hello\\_world");
        assert_eq!(html_escape("<p>&</p>"), "&lt;p&gt;&amp;&lt;/p&gt;");
        assert_eq!(
            vec_string_markdown_escape(&vec!["a_b".to_string(), "c*d".to_string()]),
            "a\\_b\nc\\*d\n"
        );
    }

    #[test]
    fn test_utf16_slice() {
        let s = "こんにちは世界";
        // Each character in s is 1 utf16 code unit
        assert_eq!(utf16_slice(s, 0, 5), Some("こんにちは"));
        assert_eq!(utf16_slice(s, 5, 2), Some("世界"));

        let s2 = "😊😊😊";
        // Each emoji is 2 utf16 code units
        assert_eq!(utf16_slice(s2, 0, 2), Some("😊"));
        assert_eq!(utf16_slice(s2, 2, 2), Some("😊"));
        assert_eq!(utf16_slice(s2, 0, 4), Some("😊😊"));
    }

    #[test]
    fn test_utf16_slice_from() {
        let s = "こんにちは世界";
        assert_eq!(utf16_slice_from(s, 5), Some("世界"));
        let s2 = "😊😊😊";
        assert_eq!(utf16_slice_from(s2, 2), Some("😊😊"));
    }
}
