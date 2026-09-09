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
        let query_len = query_len(&query_lower);
        if query_len == 0 {
            return Vec::new();
        }

        let text_lower: Vec<char> = rope.chars().map(|c| c.to_lowercase().next().unwrap()).collect();
        let mut matches = Vec::new();

        for i in 0..=text_lower.len().saturating_sub(query_len) {
            if text_lower[i..i + query_len] == query_lower[..] {
                let line = rope.char_to_line(i);
                let line_start = rope.line_to_char(line);
                let col = i - line_start;
                matches.push((line, col));
            }
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
            None => 0,
        };

        self.current_match = Some(prev);
        matches.get(prev).copied()
    }

    pub fn replace_one(&self, rope: &mut Rope, cursor_line: usize, cursor_col: usize) -> bool {
        if self.query.is_empty() {
            return false;
        }

        let query_lower = self.query_chars_lower();
        let query_len = query_len(&query_lower);
        if query_len == 0 {
            return false;
        }

        let text_lower: Vec<char> = rope.chars().map(|c| c.to_lowercase().next().unwrap()).collect();
        let cursor_pos = rope.line_to_char(cursor_line) + cursor_col;

        for i in cursor_pos..=text_lower.len().saturating_sub(query_len) {
            if text_lower[i..i + query_len] == query_lower[..] {
                let end = i + query_len;
                rope.remove(i..end);
                rope.insert(i, &self.replace_text);
                return true;
            }
        }

        false
    }

    pub fn replace_all(&self, rope: &mut Rope) -> usize {
        if self.query.is_empty() {
            return 0;
        }

        let query_lower = self.query_chars_lower();
        let query_len = query_len(&query_lower);
        if query_len == 0 {
            return 0;
        }

        let text_lower: Vec<char> = rope.chars().map(|c| c.to_lowercase().next().unwrap()).collect();
        let text_chars: Vec<char> = rope.chars().collect();
        let mut result = String::new();
        let mut count = 0;
        let mut i = 0;

        while i <= text_lower.len().saturating_sub(query_len) {
            if text_lower[i..i + query_len] == query_lower[..] {
                result.push_str(&self.replace_text);
                i += query_len;
                count += 1;
            } else {
                result.push(text_chars[i]);
                i += 1;
            }
        }

        while i < text_chars.len() {
            result.push(text_chars[i]);
            i += 1;
        }

        if count > 0 {
            *rope = Rope::from_str(&result);
        }

        count
    }
}

fn query_len(chars: &[char]) -> usize {
    chars.len()
}