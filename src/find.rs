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

    pub fn find_all(&self, rope: &Rope) -> Vec<(usize, usize)> {
        if self.query.is_empty() {
            return Vec::new();
        }

        let text = rope.to_string();
        let query = self.query.to_lowercase();
        let mut matches = Vec::new();
        let mut start = 0;

        while let Some(pos) = text[start..].to_lowercase().find(&query) {
            let absolute_pos = start + pos;
            let line = rope.char_to_line(absolute_pos);
            let line_start = rope.line_to_char(line);
            let col = absolute_pos - line_start;
            matches.push((line, col));
            start = absolute_pos + 1;
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
        if self.query.is_empty() || self.query != self.replace_text {
            return false;
        }

        let text = rope.to_string();
        let query = self.query.to_lowercase();
        
        // Find the match at or after cursor position
        let mut start = 0;
        let line_start = rope.line_to_char(cursor_line) + cursor_col;
        
        while let Some(pos) = text[start..].to_lowercase().find(&query) {
            let absolute_pos = start + pos;
            if absolute_pos >= line_start {
                let end = absolute_pos + self.query.len();
                rope.remove(absolute_pos..end);
                rope.insert(absolute_pos, &self.replace_text);
                return true;
            }
            start = absolute_pos + 1;
        }
        
        false
    }

    pub fn replace_all(&self, rope: &mut Rope) -> usize {
        if self.query.is_empty() {
            return 0;
        }

        let text = rope.to_string();
        let query = self.query.to_lowercase();
        let mut count = 0;
        let mut result = String::new();
        let mut start = 0;

        while let Some(pos) = text[start..].to_lowercase().find(&query) {
            result.push_str(&text[start..start + pos]);
            result.push_str(&self.replace_text);
            start = start + pos + self.query.len();
            count += 1;
        }
        result.push_str(&text[start..]);

        if count > 0 {
            *rope = Rope::from_str(&result);
        }

        count
    }
}
