use ropey::Rope;

#[derive(Debug, Clone)]
pub enum EditOperation {
    Insert { offset: usize, text: String },
    Delete { offset: usize, text: String },
}

#[derive(Debug, Clone)]
pub struct Buffer {
    pub rope: Rope,
    pub undo_stack: Vec<EditOperation>,
    pub redo_stack: Vec<EditOperation>,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            rope: Rope::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn from_str(text: &str) -> Self {
        Buffer {
            rope: Rope::from_str(text),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn insert_char(&mut self, offset: usize, ch: char) {
        self.redo_stack.clear();
        let op = EditOperation::Insert {
            offset,
            text: ch.to_string(),
        };
        self.undo_stack.push(op);
        self.rope.insert_char(offset, ch);
    }

    pub fn insert_str(&mut self, offset: usize, text: &str) {
        self.redo_stack.clear();
        let op = EditOperation::Insert {
            offset,
            text: text.to_string(),
        };
        self.undo_stack.push(op);
        self.rope.insert(offset, text);
    }

    pub fn delete(&mut self, offset: usize, length: usize) -> String {
        self.redo_stack.clear();
        let deleted: String = self.rope.slice(offset..offset + length).chars().collect();
        let op = EditOperation::Delete {
            offset,
            text: deleted.clone(),
        };
        self.undo_stack.push(op);
        self.rope.remove(offset..offset + length);
        deleted
    }

    pub fn undo(&mut self) -> Option<(usize, usize)> {
        if let Some(op) = self.undo_stack.pop() {
            match op {
                EditOperation::Insert { offset, text } => {
                    let len = text.chars().count();
                    self.redo_stack.push(EditOperation::Delete {
                        offset,
                        text: text.clone(),
                    });
                    self.rope.remove(offset..offset + len);
                    Some((offset, offset))
                }
                EditOperation::Delete { offset, text } => {
                    self.redo_stack.push(EditOperation::Insert {
                        offset,
                        text: text.clone(),
                    });
                    self.rope.insert(offset, &text);
                    Some((offset, offset + text.chars().count()))
                }
            }
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<(usize, usize)> {
        if let Some(op) = self.redo_stack.pop() {
            match op {
                EditOperation::Insert { offset, text } => {
                    let len = text.chars().count();
                    self.undo_stack.push(EditOperation::Delete {
                        offset,
                        text: text.clone(),
                    });
                    self.rope.insert(offset, &text);
                    Some((offset, offset + len))
                }
                EditOperation::Delete { offset, text } => {
                    self.undo_stack.push(EditOperation::Insert {
                        offset,
                        text: text.clone(),
                    });
                    self.rope.remove(offset..offset + text.chars().count());
                    Some((offset, offset))
                }
            }
        } else {
            None
        }
    }

    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn line(&self, line_idx: usize) -> ropey::RopeSlice {
        self.rope.line(line_idx)
    }

    pub fn to_string(&self) -> String {
        self.rope.to_string()
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_is_empty() {
        let buf = Buffer::new();
        assert_eq!(buf.to_string(), "");
        assert_eq!(buf.len_chars(), 0);
        assert_eq!(buf.len_lines(), 1);
    }

    #[test]
    fn from_str_preserves_content() {
        let buf = Buffer::from_str("hello world");
        assert_eq!(buf.to_string(), "hello world");
        assert_eq!(buf.len_chars(), 11);
    }

    #[test]
    fn insert_char_at_start() {
        let mut buf = Buffer::from_str("bc");
        buf.insert_char(0, 'a');
        assert_eq!(buf.to_string(), "abc");
    }

    #[test]
    fn insert_char_at_end() {
        let mut buf = Buffer::from_str("ab");
        buf.insert_char(2, 'c');
        assert_eq!(buf.to_string(), "abc");
    }

    #[test]
    fn insert_char_in_middle() {
        let mut buf = Buffer::from_str("ac");
        buf.insert_char(1, 'b');
        assert_eq!(buf.to_string(), "abc");
    }

    #[test]
    fn insert_str_at_start() {
        let mut buf = Buffer::from_str("world");
        buf.insert_str(0, "hello ");
        assert_eq!(buf.to_string(), "hello world");
    }

    #[test]
    fn insert_str_at_end() {
        let mut buf = Buffer::from_str("hello");
        buf.insert_str(5, " world");
        assert_eq!(buf.to_string(), "hello world");
    }

    #[test]
    fn insert_str_in_middle() {
        let mut buf = Buffer::from_str("hd");
        buf.insert_str(1, "ello worl");
        assert_eq!(buf.to_string(), "hello world");
    }

    #[test]
    fn delete_returns_deleted_text() {
        let mut buf = Buffer::from_str("hello world");
        let deleted = buf.delete(5, 6);
        assert_eq!(deleted, " world");
        assert_eq!(buf.to_string(), "hello");
    }

    #[test]
    fn delete_from_start() {
        let mut buf = Buffer::from_str("abc");
        buf.delete(0, 1);
        assert_eq!(buf.to_string(), "bc");
    }

    #[test]
    fn delete_entire_content() {
        let mut buf = Buffer::from_str("abc");
        buf.delete(0, 3);
        assert_eq!(buf.to_string(), "");
        assert_eq!(buf.len_chars(), 0);
    }

    #[test]
    fn undo_insert_removes_char() {
        let mut buf = Buffer::from_str("ab");
        buf.insert_char(1, 'x');
        assert_eq!(buf.to_string(), "axb");
        buf.undo();
        assert_eq!(buf.to_string(), "ab");
    }

    #[test]
    fn undo_insert_removes_str() {
        let mut buf = Buffer::from_str("hello");
        buf.insert_str(5, " world");
        assert_eq!(buf.to_string(), "hello world");
        buf.undo();
        assert_eq!(buf.to_string(), "hello");
    }

    #[test]
    fn undo_delete_restores_text() {
        let mut buf = Buffer::from_str("hello world");
        buf.delete(5, 6);
        assert_eq!(buf.to_string(), "hello");
        buf.undo();
        assert_eq!(buf.to_string(), "hello world");
    }

    #[test]
    fn redo_reapplies_insert() {
        let mut buf = Buffer::from_str("ab");
        buf.insert_char(1, 'x');
        assert_eq!(buf.to_string(), "axb");
        buf.undo();
        assert_eq!(buf.to_string(), "ab");
        buf.redo();
        assert_eq!(buf.to_string(), "axb");
    }

    #[test]
    fn redo_reapplies_delete() {
        let mut buf = Buffer::from_str("hello world");
        buf.delete(5, 6);
        assert_eq!(buf.to_string(), "hello");
        buf.undo();
        assert_eq!(buf.to_string(), "hello world");
        buf.redo();
        assert_eq!(buf.to_string(), "hello");
    }

    #[test]
    fn new_edit_clears_redo_stack() {
        let mut buf = Buffer::from_str("ab");
        buf.insert_char(1, 'x');
        buf.undo();
        assert_eq!(buf.to_string(), "ab");
        buf.insert_char(1, 'y');
        assert_eq!(buf.to_string(), "ayb");
        assert!(buf.redo().is_none(), "redo stack should be cleared");
    }

    #[test]
    fn undo_empty_stack_returns_none() {
        let mut buf = Buffer::from_str("hello");
        assert!(buf.undo().is_none());
    }

    #[test]
    fn redo_empty_stack_returns_none() {
        let mut buf = Buffer::from_str("hello");
        assert!(buf.redo().is_none());
    }

    #[test]
    fn undo_redo_roundtrip() {
        let mut buf = Buffer::from_str("original");
        buf.insert_str(8, " text");
        buf.delete(0, 4);
        assert_eq!(buf.to_string(), "inal text");
        buf.undo();
        buf.undo();
        assert_eq!(buf.to_string(), "original");
    }

    #[test]
    fn len_chars_after_insert() {
        let mut buf = Buffer::from_str("ab");
        buf.insert_char(1, 'x');
        assert_eq!(buf.len_chars(), 3);
    }

    #[test]
    fn len_chars_after_delete() {
        let mut buf = Buffer::from_str("abc");
        buf.delete(1, 1);
        assert_eq!(buf.len_chars(), 2);
    }

    #[test]
    fn len_lines_multiline() {
        let buf = Buffer::from_str("line1\nline2\nline3");
        assert_eq!(buf.len_lines(), 3);
    }

    #[test]
    fn line_returns_correct_content() {
        let buf = Buffer::from_str("first\nsecond\nthird");
        let line1: String = buf.line(1).chars().collect();
        assert_eq!(line1, "second");
    }

    #[test]
    fn insert_unicode_char() {
        let mut buf = Buffer::from_str("ab");
        buf.insert_char(1, 'é');
        assert_eq!(buf.to_string(), "aéb");
        assert_eq!(buf.len_chars(), 3);
    }

    #[test]
    fn delete_unicode_text() {
        let mut buf = Buffer::from_str("héllo");
        let deleted = buf.delete(1, 1);
        assert_eq!(deleted, "é");
        assert_eq!(buf.to_string(), "hllo");
    }
}
