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
