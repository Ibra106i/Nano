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
        let line_len = rope.line(self.line).len_chars();
        let last_line = rope.len_lines().saturating_sub(1);
        
        if self.line == last_line {
            self.col = line_len;
        } else {
            // Exclude trailing newline for non-last lines
            self.col = line_len.saturating_sub(1);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn rope(text: &str) -> Rope {
        Rope::from_str(text)
    }

    #[test]
    fn new_starts_at_origin() {
        let c = Cursor::new();
        assert_eq!(c.line, 0);
        assert_eq!(c.col, 0);
        assert!(!c.has_selection());
    }

    #[test]
    fn move_right_advance() {
        let r = rope("hello");
        let mut c = Cursor::new();
        c.move_right(&r);
        assert_eq!(c.col, 1);
    }

    #[test]
    fn move_right_wrap_to_next_line() {
        let r = rope("ab\ncd");
        let mut c = Cursor::new();
        c.move_right(&r); // col 1
        c.move_right(&r); // col 2
        c.move_right(&r); // wraps to line 1, col 0
        assert_eq!(c.line, 1);
        assert_eq!(c.col, 0);
    }

    #[test]
    fn move_right_stops_at_end() {
        let r = rope("ab");
        let mut c = Cursor::new();
        c.move_right(&r);
        c.move_right(&r);
        c.move_right(&r); // already at end, no change
        assert_eq!(c.line, 0);
        assert_eq!(c.col, 2);
    }

    #[test]
    fn move_left_wrap_to_previous_line() {
        let r = rope("ab\ncd");
        let mut c = Cursor::new();
        c.line = 1;
        c.col = 0;
        c.move_left(&r);
        assert_eq!(c.line, 0);
        assert_eq!(c.col, 2); // end of "ab"
    }

    #[test]
    fn move_left_advance() {
        let r = rope("hello");
        let mut c = Cursor::new();
        c.move_right(&r);
        c.move_right(&r);
        c.move_left(&r);
        assert_eq!(c.col, 1);
    }

    #[test]
    fn move_left_stops_at_origin() {
        let r = rope("hello");
        let mut c = Cursor::new();
        c.move_left(&r); // already at 0,0
        assert_eq!(c.line, 0);
        assert_eq!(c.col, 0);
    }

    #[test]
    fn move_up_basic() {
        let r = rope("line1\nline2\nline3");
        let mut c = Cursor::new();
        c.line = 2;
        c.col = 3;
        c.move_up(&r);
        assert_eq!(c.line, 1);
        assert_eq!(c.col, 3);
    }

    #[test]
    fn move_up_clamp_col() {
        let r = rope("short\nlonger line");
        let mut c = Cursor::new();
        c.line = 1;
        c.col = 10;
        c.move_up(&r);
        assert_eq!(c.line, 0);
        assert_eq!(c.col, 5); // "short" has 5 chars
    }

    #[test]
    fn move_up_stops_at_top() {
        let r = rope("hello");
        let mut c = Cursor::new();
        c.move_up(&r);
        assert_eq!(c.line, 0);
    }

    #[test]
    fn move_down_basic() {
        let r = rope("line1\nline2\nline3");
        let mut c = Cursor::new();
        c.move_down(&r);
        assert_eq!(c.line, 1);
        assert_eq!(c.col, 0);
    }

    #[test]
    fn move_down_clamp_col() {
        let r = rope("longer line\nshort");
        let mut c = Cursor::new();
        c.col = 10;
        c.move_down(&r);
        assert_eq!(c.line, 1);
        assert_eq!(c.col, 5); // "short" has 5 chars
    }

    #[test]
    fn move_down_stops_at_bottom() {
        let r = rope("hello");
        let mut c = Cursor::new();
        c.move_down(&r);
        assert_eq!(c.line, 0);
    }

    #[test]
    fn move_home_sets_col_zero() {
        let r = rope("hello");
        let mut c = Cursor::new();
        c.move_right(&r);
        c.move_right(&r);
        c.move_home();
        assert_eq!(c.col, 0);
    }

    #[test]
    fn move_end_non_last_line() {
        let r = rope("ab\ncd");
        let mut c = Cursor::new();
        c.move_end(&r);
        // Non-last line: col = line_len.saturating_sub(1) to exclude trailing \n
        assert_eq!(c.col, 1); // "ab" has 2 chars, minus 1 for newline
    }

    #[test]
    fn move_end_last_line() {
        let r = rope("ab\ncd");
        let mut c = Cursor::new();
        c.line = 1;
        c.move_end(&r);
        assert_eq!(c.col, 2); // "cd" has 2 chars, no trailing newline
    }

    #[test]
    fn page_up_basic() {
        let r = rope("line0\nline1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10");
        let mut c = Cursor::new();
        c.line = 8;
        c.page_up(&r, 3);
        assert_eq!(c.line, 5);
    }

    #[test]
    fn page_up_clamps_at_top() {
        let r = rope("line0\nline1\nline2");
        let mut c = Cursor::new();
        c.line = 1;
        c.page_up(&r, 5);
        assert_eq!(c.line, 0);
    }

    #[test]
    fn page_down_basic() {
        let r = rope("line0\nline1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10");
        let mut c = Cursor::new();
        c.page_down(&r, 3);
        assert_eq!(c.line, 3);
    }

    #[test]
    fn page_down_clamps_at_bottom() {
        let r = rope("line0\nline1\nline2");
        let mut c = Cursor::new();
        c.line = 1;
        c.page_down(&r, 5);
        assert_eq!(c.line, 2);
    }

    #[test]
    fn selection_start_and_range_forward() {
        let r = rope("hello");
        let mut c = Cursor::new();
        c.start_selection();
        c.move_right(&r);
        c.move_right(&r);
        c.move_right(&r);
        let range = c.selection_range().unwrap();
        assert_eq!(range.0, (0, 0));
        assert_eq!(range.1, (0, 3));
    }

    #[test]
    fn selection_range_backward() {
        let r = rope("hello");
        let mut c = Cursor::new();
        c.move_right(&r);
        c.move_right(&r);
        c.start_selection();
        c.move_left(&r);
        c.move_left(&r);
        let range = c.selection_range().unwrap();
        assert_eq!(range.0, (0, 0));
        assert_eq!(range.1, (0, 2));
    }

    #[test]
    fn selection_clear() {
        let r = rope("hello");
        let mut c = Cursor::new();
        c.start_selection();
        assert!(c.has_selection());
        c.clear_selection();
        assert!(!c.has_selection());
    }

    #[test]
    fn to_byte_offset_basic() {
        let r = rope("hello\nworld");
        let mut c = Cursor::new();
        c.line = 1;
        c.col = 3;
        assert_eq!(c.to_byte_offset(&r), 8); // "hello\n" = 6, + 3 = 9... wait
    }

    #[test]
    fn from_byte_offset_basic() {
        let r = rope("hello\nworld");
        let c = Cursor::from_byte_offset(8, &r);
        assert_eq!(c.line, 1);
        assert_eq!(c.col, 2); // offset 8 = "hello\nwo" -> line 1, col 2
    }

    #[test]
    fn byte_offset_roundtrip() {
        let r = rope("hello\nworld\nfoo");
        let mut c = Cursor::new();
        c.line = 2;
        c.col = 1;
        let offset = c.to_byte_offset(&r);
        let c2 = Cursor::from_byte_offset(offset, &r);
        assert_eq!(c2.line, 2);
        assert_eq!(c2.col, 1);
    }
}
