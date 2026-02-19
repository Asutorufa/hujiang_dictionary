pub const MARKDOWN_ESCAPE_CHARS: [char; 19] = [
    '\\', '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!',
];

pub fn markdown_escape(s: &str) -> String {
    s.chars().fold(String::with_capacity(s.len()), |mut s, c| {
        if MARKDOWN_ESCAPE_CHARS.contains(&c) {
            s.push('\\');
        }
        s.push(c);
        s
    })
}

pub fn html_escape(s: &str) -> String {
    s.chars().fold(String::with_capacity(s.len()), |mut s, c| {
        match c {
            '&' => s.push_str("&amp;"),
            '<' => s.push_str("&lt;"),
            '>' => s.push_str("&gt;"),
            c => s.push(c),
        }
        s
    })
}

pub fn vec_string_markdown_escape(v: &Vec<String>) -> String {
    let mut s = String::new();
    for i in v {
        s.push_str(markdown_escape(i.as_str()).as_str());
        s.push_str("\n");
    }
    s
}

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

pub fn get_utf16_slice(s: &str, offset: usize, length: usize) -> String {
    let wide: Vec<u16> = s.encode_utf16().collect();
    if offset >= wide.len() {
        return "".to_string();
    }
    let end = (offset + length).min(wide.len());
    String::from_utf16_lossy(&wide[offset..end])
}

pub fn get_utf16_slice_rest(s: &str, offset: usize) -> String {
    let wide: Vec<u16> = s.encode_utf16().collect();
    if offset >= wide.len() {
        return "".to_string();
    }
    String::from_utf16_lossy(&wide[offset..])
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
    fn test_utf16_slice() {
        let text = "你好 world"; // 你(1) 好(1) ' '(1)
        // utf16: 4f60 597d 0020 ...

        let s = get_utf16_slice(text, 0, 1);
        assert_eq!(s, "你");

        let s = get_utf16_slice(text, 1, 1);
        assert_eq!(s, "好");

        let s = get_utf16_slice(text, 2, 5);
        assert_eq!(s, " worl");

        let s = get_utf16_slice_rest(text, 2);
        assert_eq!(s, " world");
    }
}
