use ropey::Rope;

#[derive(Debug, Clone)]
pub struct Cursor {
    pub line: usize,
    pub col: usize,
    pub selection_start: Option<(usize, usize)>,
}

impl Cursor {
    pub fn new() -> Self {
        Cursor {
            line: 0,
            col: 0,
            selection_start: None,
        }
    }

    pub fn start_selection(&mut self) {
        self.selection_start = Some((self.line, self.col));
    }

    pub fn clear_selection(&mut self) {
        self.selection_start = None;
    }

    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some()
    }

    pub fn selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        self.selection_start.map(|start| {
            if start <= (self.line, self.col) {
                (start, (self.line, self.col))
            } else {
                ((self.line, self.col), start)
            }
        })
    }

    pub fn move_up(&mut self, rope: &Rope) {
        if self.line > 0 {
            self.line -= 1;
            let line_len = rope.line(self.line).len_chars();
            self.col = self.col.min(line_len);
        }
    }

    pub fn move_down(&mut self, rope: &Rope) {
        let last_line = rope.len_lines() - 1;
        if self.line < last_line {
            self.line += 1;
            let line_len = rope.line(self.line).len_chars();
            self.col = self.col.min(line_len);
        }
    }

    pub fn move_left(&mut self, rope: &Rope) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.line > 0 {
            self.line -= 1;
            self.col = rope.line(self.line).len_chars();
        }
    }

    pub fn move_right(&mut self, rope: &Rope) {
        let line_len = rope.line(self.line).len_chars();
        if self.col < line_len {
            self.col += 1;
        } else if self.line < rope.len_lines() - 1 {
            self.line += 1;
            self.col = 0;
        }
    }

    pub fn move_home(&mut self) {
        self.col = 0;
    }

    pub fn move_end(&mut self, rope: &Rope) {
        self.col = rope.line(self.line).len_chars();
    }

    pub fn page_up(&mut self, rope: &Rope, page_size: usize) {
        self.line = self.line.saturating_sub(page_size);
        let line_len = rope.line(self.line).len_chars();
        self.col = self.col.min(line_len);
    }

    pub fn page_down(&mut self, rope: &Rope, page_size: usize) {
        let last_line = rope.len_lines() - 1;
        self.line = (self.line + page_size).min(last_line);
        let line_len = rope.line(self.line).len_chars();
        self.col = self.col.min(line_len);
    }

    pub fn to_byte_offset(&self, rope: &Rope) -> usize {
        let line_start = rope.line_to_char(self.line);
        line_start + self.col
    }

    pub fn from_byte_offset(offset: usize, rope: &Rope) -> Self {
        let line = rope.char_to_line(offset);
        let line_start = rope.line_to_char(line);
        let col = offset - line_start;
        Cursor {
            line,
            col,
            selection_start: None,
        }
    }
}
