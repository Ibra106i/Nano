use ropey::Rope;

#[derive(Debug, Clone)]
pub struct FindState {
    pub query: String,
    pub replace_text: String,
    pub show_find_bar: bool,
    pub show_replace: bool,
    pub current_match: Option<usize>,
    pub total_matches: usize,
}

impl FindState {
    pub fn new() -> Self {
        FindState {
            query: String::new(),
            replace_text: String::new(),
            show_find_bar: false,
            show_replace: false,
            current_match: None,
            total_matches: 0,
        }
    }

    pub fn toggle(&mut self) {
        self.show_find_bar = !self.show_find_bar;
        if !self.show_find_bar {
            self.query.clear();
            self.replace_text.clear();
            self.current_match = None;
            self.total_matches = 0;
        }
    }

    pub fn toggle_replace(&mut self) {
        self.show_replace = !self.show_replace;
    }

    fn query_chars_lower(&self) -> Vec<char> {
        self.query.to_lowercase().chars().collect()
    }

    pub fn find_all(&self, rope: &Rope) -> Vec<(usize, usize)> {
        if self.query.is_empty() {
            return Vec::new();
        }

        let query_lower = self.query_chars_lower();
        let query_len = query_lower.len();
        if query_len == 0 {
            return Vec::new();
        }

        let mut matches = Vec::new();
        let mut window: VecDeque<char> = VecDeque::with_capacity(query_len);
        let mut char_idx = 0;

        for ch in rope.chars() {
            window.push_back(ch.to_lowercase().next().unwrap());
            if window.len() > query_len {
                window.pop_front();
            }
            if window.len() == query_len && window.iter().zip(query_lower.iter()).all(|(a, b)| a == b) {
                let line = rope.char_to_line(char_idx);
                let line_start = rope.line_to_char(line);
                matches.push((line, char_idx - line_start));
            }
            char_idx += 1;
        }

        matches
    }

    pub fn find_next(&mut self, rope: &Rope) -> Option<(usize, usize)> {
        let matches = self.find_all(rope);
        self.total_matches = matches.len();
        
        if matches.is_empty() {
            self.current_match = None;
            return None;
        }

        let next = match self.current_match {
            Some(current) => {
                if current + 1 < matches.len() {
                    current + 1
                } else {
                    0
                }
            }
            None => 0,
        };

        self.current_match = Some(next);
        matches.get(next).copied()
    }

    pub fn find_previous(&mut self, rope: &Rope) -> Option<(usize, usize)> {
        let matches = self.find_all(rope);
        self.total_matches = matches.len();
        
        if matches.is_empty() {
            self.current_match = None;
            return None;
        }

        let prev = match self.current_match {
            Some(current) => {
                if current > 0 {
                    current - 1
                } else {
                    matches.len() - 1
                }
            }
            None => matches.len() - 1,
        };

        self.current_match = Some(prev);
        matches.get(prev).copied()
    }

    pub fn replace_one(&self, rope: &mut Rope, cursor_line: usize, cursor_col: usize) -> bool {
        if self.query.is_empty() {
            return false;
        }

        let query_lower = self.query_chars_lower();
        let query_len = query_lower.len();
        if query_len == 0 {
            return false;
        }

        let cursor_pos = rope.line_to_char(cursor_line) + cursor_col;
        let mut window: VecDeque<char> = VecDeque::with_capacity(query_len);
        let mut char_idx = 0;

        for ch in rope.chars() {
            window.push_back(ch.to_lowercase().next().unwrap());
            if window.len() > query_len {
                window.pop_front();
            }
            if char_idx >= cursor_pos && window.len() == query_len
                && window.iter().zip(query_lower.iter()).all(|(a, b)| a == b)
            {
                let start = char_idx + 1 - query_len;
                let end = char_idx + 1;
                rope.remove(start..end);
                rope.insert(start, &self.replace_text);
                return true;
            }
            char_idx += 1;
        }

        false
    }

    pub fn replace_all(&self, rope: &mut Rope) -> usize {
        if self.query.is_empty() {
            return 0;
        }

        let matches = self.find_all(rope);
        if matches.is_empty() {
            return 0;
        }

        let mut result = String::new();
        let mut chars = rope.chars().enumerate().peekable();
        let mut match_idx = 0;
        let mut count = 0;

        while let Some((i, ch)) = chars.next() {
            if match_idx < matches.len() {
                let (line, col) = matches[match_idx];
                let match_start = rope.line_to_char(line) + col;
                if i == match_start {
                    result.push_str(&self.replace_text);
                    count += 1;
                    let query_len = self.query_chars_lower().len();
                    for _ in 0..query_len - 1 {
                        chars.next();
                    }
                    match_idx += 1;
                    continue;
                }
            }
            result.push(ch);
        }

        if count > 0 {
            *rope = Rope::from_str(&result);
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use ropey::Rope;
use std::collections::VecDeque;

    fn rope(text: &str) -> Rope {
        Rope::from_str(text)
    }

    #[test]
    fn new_find_state() {
        let fs = FindState::new();
        assert!(fs.query.is_empty());
        assert!(fs.replace_text.is_empty());
        assert!(!fs.show_find_bar);
        assert!(!fs.show_replace);
        assert!(fs.current_match.is_none());
        assert_eq!(fs.total_matches, 0);
    }

    #[test]
    fn toggle_show_find_bar() {
        let mut fs = FindState::new();
        fs.toggle();
        assert!(fs.show_find_bar);
        fs.toggle();
        assert!(!fs.show_find_bar);
        assert!(fs.query.is_empty()); // clears on hide
    }

    #[test]
    fn toggle_replace() {
        let mut fs = FindState::new();
        fs.toggle_replace();
        assert!(fs.show_replace);
        fs.toggle_replace();
        assert!(!fs.show_replace);
    }

    #[test]
    fn find_all_basic() {
        let r = rope("hello world hello");
        let mut fs = FindState::new();
        fs.query = "hello".to_string();
        let matches = fs.find_all(&r);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], (0, 0));
        assert_eq!(matches[1], (0, 12));
    }

    #[test]
    fn find_all_case_insensitive() {
        let r = rope("Hello HELLO hello");
        let mut fs = FindState::new();
        fs.query = "hello".to_string();
        let matches = fs.find_all(&r);
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn find_all_empty_query() {
        let r = rope("hello");
        let fs = FindState::new();
        assert!(fs.find_all(&r).is_empty());
    }

    #[test]
    fn find_all_no_matches() {
        let r = rope("hello world");
        let mut fs = FindState::new();
        fs.query = "xyz".to_string();
        assert!(fs.find_all(&r).is_empty());
    }

    #[test]
    fn find_all_multiline() {
        let r = rope("line1\nline2\nline3");
        let mut fs = FindState::new();
        fs.query = "line".to_string();
        let matches = fs.find_all(&r);
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0], (0, 0));
        assert_eq!(matches[1], (1, 0));
        assert_eq!(matches[2], (2, 0));
    }

    #[test]
    fn find_next_first_call() {
        let r = rope("aaa");
        let mut fs = FindState::new();
        fs.query = "a".to_string();
        let result = fs.find_next(&r);
        assert_eq!(result, Some((0, 0)));
        assert_eq!(fs.current_match, Some(0));
    }

    #[test]
    fn find_next_advances() {
        let r = rope("a b a b");
        let mut fs = FindState::new();
        fs.query = "a".to_string();
        fs.find_next(&r); // match 0
        let result = fs.find_next(&r); // match 1
        assert_eq!(result, Some((0, 4)));
    }

    #[test]
    fn find_next_wraps_around() {
        let r = rope("a b a b");
        let mut fs = FindState::new();
        fs.query = "a".to_string();
        fs.find_next(&r); // 0
        fs.find_next(&r); // 1
        let result = fs.find_next(&r); // wraps to 0
        assert_eq!(result, Some((0, 0)));
    }

    #[test]
    fn find_previous_first_call() {
        let r = rope("a b a b");
        let mut fs = FindState::new();
        fs.query = "a".to_string();
        let result = fs.find_previous(&r);
        // current_match is None → prev = last match (wraps to end)
        assert_eq!(result, Some((0, 4)));
    }

    #[test]
    fn find_previous_goes_back() {
        let r = rope("a b a b");
        let mut fs = FindState::new();
        fs.query = "a".to_string();
        fs.find_next(&r); // match 0
        fs.find_next(&r); // match 1
        let result = fs.find_previous(&r); // back to 0
        assert_eq!(result, Some((0, 0)));
    }

    #[test]
    fn find_previous_wraps_around() {
        let r = rope("a b a b");
        let mut fs = FindState::new();
        fs.query = "a".to_string();
        fs.find_next(&r); // match 0
        let result = fs.find_previous(&r); // at 0, wraps to last (index 1)
        assert_eq!(result, Some((0, 4)));
    }

    #[test]
    fn replace_one_basic() {
        let mut r = rope("hello world hello");
        let mut fs = FindState::new();
        fs.query = "hello".to_string();
        fs.replace_text = "hi".to_string();
        let replaced = fs.replace_one(&mut r, 0, 0);
        assert!(replaced);
        let result: String = r.chars().collect();
        assert_eq!(result, "hi world hello");
    }

    #[test]
    fn replace_one_after_cursor() {
        let mut r = rope("hello world hello");
        let mut fs = FindState::new();
        fs.query = "hello".to_string();
        fs.replace_text = "hi".to_string();
        // cursor at col 6 (after first "hello"), should replace second "hello"
        let replaced = fs.replace_one(&mut r, 0, 6);
        assert!(replaced);
        let result: String = r.chars().collect();
        assert_eq!(result, "hello world hi");
    }

    #[test]
    fn replace_one_no_match() {
        let mut r = rope("hello world");
        let mut fs = FindState::new();
        fs.query = "xyz".to_string();
        fs.replace_text = "abc".to_string();
        assert!(!fs.replace_one(&mut r, 0, 0));
    }

    #[test]
    fn replace_one_empty_query() {
        let mut r = rope("hello");
        let fs = FindState::new();
        assert!(!fs.replace_one(&mut r, 0, 0));
    }

    #[test]
    fn replace_all_basic() {
        let mut r = rope("aaa");
        let mut fs = FindState::new();
        fs.query = "a".to_string();
        fs.replace_text = "b".to_string();
        let count = fs.replace_all(&mut r);
        assert_eq!(count, 3);
        let result: String = r.chars().collect();
        assert_eq!(result, "bbb");
    }

    #[test]
    fn replace_all_partial() {
        let mut r = rope("hello world hello");
        let mut fs = FindState::new();
        fs.query = "hello".to_string();
        fs.replace_text = "hi".to_string();
        let count = fs.replace_all(&mut r);
        assert_eq!(count, 2);
        let result: String = r.chars().collect();
        assert_eq!(result, "hi world hi");
    }

    #[test]
    fn replace_all_empty_query() {
        let mut r = rope("hello");
        let fs = FindState::new();
        assert_eq!(fs.replace_all(&mut r), 0);
    }

    #[test]
    fn replace_all_no_matches() {
        let mut r = rope("hello");
        let mut fs = FindState::new();
        fs.query = "xyz".to_string();
        assert_eq!(fs.replace_all(&mut r), 0);
    }
}