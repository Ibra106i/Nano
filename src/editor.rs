use iced::{Element, Task, Subscription};
use iced::widget::{button, column, container, horizontal_space, row, scrollable, text};
use iced::keyboard::{self, key};

use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::file_io::{self, FileInfo};
use crate::find::FindState;
use crate::syntax::SyntaxHighlighter;
use crate::theme::Theme as EditorTheme;

impl Default for Editor {
    fn default() -> Self {
        Editor::new().0
    }
}

#[derive(Debug, Clone)]
pub struct Editor {
    pub buffer: Buffer,
    pub cursor: Cursor,
    pub file_info: FileInfo,
    pub theme: EditorTheme,
    pub syntax: SyntaxHighlighter,
    pub find_state: FindState,
    pub scroll_offset: f32,
    pub line_numbers: bool,
    pub word_wrap: bool,
    pub viewport_height: f32,
    pub line_height: f32,
}

#[derive(Debug, Clone)]
pub enum Message {
    // File operations
    NewFile,
    OpenFile,
    SaveFile,
    SaveFileAs,
    FileOpened(Result<String, String>),
    FileSaved(Result<(), String>),
    
    // Editing
    InsertChar(char),
    DeleteBackward,
    DeleteForward,
    Newline,
    Tab,
    
    // Cursor movement
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,
    CursorHome,
    CursorEnd,
    PageUp,
    PageDown,
    
    // Selection
    StartSelection,
    ExtendSelection(Box<Message>),
    
    // Clipboard
    Copy,
    Cut,
    Paste,
    
    // Undo/Redo
    Undo,
    Redo,
    
    // Find & Replace
    FindToggle,
    ReplaceToggle,
    FindQueryChanged(String),
    ReplaceTextChanged(String),
    FindNext,
    FindPrevious,
    ReplaceOne,
    ReplaceAll,
    
    // View
    ToggleLineNumbers,
    ToggleWordWrap,
    ToggleTheme,
    
    // Scroll
    Scroll(f32),
    
    // UI
    Resized(iced::Size),
}

impl Editor {
    pub fn new() -> (Self, Task<Message>) {
        (
            Editor {
                buffer: Buffer::new(),
                cursor: Cursor::new(),
                file_info: FileInfo::new(),
                theme: EditorTheme::Dark,
                syntax: SyntaxHighlighter::new(),
                find_state: FindState::new(),
                scroll_offset: 0.0,
                line_numbers: true,
                word_wrap: false,
                viewport_height: 600.0,
                line_height: 20.0,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NewFile => {
                if self.file_info.is_modified {
                    // TODO: Show save confirmation dialog
                }
                self.buffer = Buffer::new();
                self.cursor = Cursor::new();
                self.file_info = FileInfo::new();
                self.syntax = SyntaxHighlighter::new();
            }
            
            Message::OpenFile => {
                if let Some(path) = file_io::open_file_dialog() {
                    let path_clone = path.clone();
                    return Task::perform(
                        async move {
                            file_io::read_file(&path_clone)
                        },
                        |result| Message::FileOpened(result),
                    );
                }
            }
            
            Message::FileOpened(result) => {
                match result {
                    Ok(content) => {
                        self.buffer = Buffer::from_str(&content);
                        self.cursor = Cursor::new();
                        // TODO: Set file path from dialog
                        self.file_info = FileInfo::new();
                        self.file_info.mark_saved();
                    }
                    Err(e) => {
                        // TODO: Show error dialog
                        eprintln!("Error opening file: {}", e);
                    }
                }
            }
            
            Message::SaveFile => {
                if let Some(path) = &self.file_info.path.clone() {
                    let content = self.buffer.to_string();
                    let path_clone = path.clone();
                    return Task::perform(
                        async move {
                            file_io::write_file(&path_clone, &content)
                        },
                        |result| Message::FileSaved(result),
                    );
                } else {
                    return self.update(Message::SaveFileAs);
                }
            }
            
            Message::SaveFileAs => {
                if let Some(path) = file_io::save_file_dialog() {
                    self.file_info.path = Some(path.clone());
                    let content = self.buffer.to_string();
                    return Task::perform(
                        async move {
                            file_io::write_file(&path, &content)
                        },
                        |result| Message::FileSaved(result),
                    );
                }
            }
            
            Message::FileSaved(result) => {
                match result {
                    Ok(()) => {
                        self.file_info.mark_saved();
                    }
                    Err(e) => {
                        // TODO: Show error dialog
                        eprintln!("Error saving file: {}", e);
                    }
                }
            }
            
            Message::InsertChar(ch) => {
                let offset = self.cursor.to_byte_offset(&self.buffer.rope);
                self.buffer.insert_char(offset, ch);
                self.cursor.move_right(&self.buffer.rope);
                self.file_info.mark_modified();
            }
            
            Message::DeleteBackward => {
                if self.cursor.col > 0 || self.cursor.line > 0 {
                    let offset = self.cursor.to_byte_offset(&self.buffer.rope);
                    if offset > 0 {
                        self.buffer.delete(offset - 1, 1);
                        self.cursor.move_left(&self.buffer.rope);
                        self.file_info.mark_modified();
                    }
                }
            }
            
            Message::DeleteForward => {
                let offset = self.cursor.to_byte_offset(&self.buffer.rope);
                if offset < self.buffer.len_chars() {
                    self.buffer.delete(offset, 1);
                    self.file_info.mark_modified();
                }
            }
            
            Message::Newline => {
                let offset = self.cursor.to_byte_offset(&self.buffer.rope);
                self.buffer.insert_str(offset, "\n");
                self.cursor.move_down(&self.buffer.rope);
                self.cursor.move_home();
                self.file_info.mark_modified();
            }
            
            Message::Tab => {
                let offset = self.cursor.to_byte_offset(&self.buffer.rope);
                self.buffer.insert_str(offset, "    ");
                self.cursor.col += 4;
                self.file_info.mark_modified();
            }
            
            Message::CursorUp => {
                if self.cursor.has_selection() {
                    self.cursor.move_up(&self.buffer.rope);
                } else {
                    self.cursor.move_up(&self.buffer.rope);
                }
            }
            
            Message::CursorDown => {
                if self.cursor.has_selection() {
                    self.cursor.move_down(&self.buffer.rope);
                } else {
                    self.cursor.move_down(&self.buffer.rope);
                }
            }
            
            Message::CursorLeft => {
                if self.cursor.has_selection() {
                    let range = self.cursor.selection_range().unwrap();
                    self.cursor.line = range.0 .0;
                    self.cursor.col = range.0 .1;
                    self.cursor.clear_selection();
                } else {
                    self.cursor.move_left(&self.buffer.rope);
                }
            }
            
            Message::CursorRight => {
                if self.cursor.has_selection() {
                    let range = self.cursor.selection_range().unwrap();
                    self.cursor.line = range.1 .0;
                    self.cursor.col = range.1 .1;
                    self.cursor.clear_selection();
                } else {
                    self.cursor.move_right(&self.buffer.rope);
                }
            }
            
            Message::CursorHome => {
                self.cursor.move_home();
            }
            
            Message::CursorEnd => {
                self.cursor.move_end(&self.buffer.rope);
            }
            
            Message::PageUp => {
                self.cursor.page_up(&self.buffer.rope, (self.viewport_height / self.line_height) as usize);
            }
            
            Message::PageDown => {
                self.cursor.page_down(&self.buffer.rope, (self.viewport_height / self.line_height) as usize);
            }
            
            Message::StartSelection => {
                self.cursor.start_selection();
            }
            
            Message::ExtendSelection(msg) => {
                if !self.cursor.has_selection() {
                    self.cursor.start_selection();
                }
                self.update(*msg);
            }
            
            Message::Copy => {
                if let Some(range) = self.cursor.selection_range() {
                    let start = self.buffer.rope.line_to_char(range.0 .0) + range.0 .1;
                    let end = self.buffer.rope.line_to_char(range.1 .0) + range.1 .1;
                    let _selected: String = self.buffer.rope.slice(start..end).chars().collect();
                    // TODO: Use copypasta crate for clipboard
                }
                return Task::none();
            }
            
            Message::Cut => {
                if let Some(range) = self.cursor.selection_range() {
                    let start = self.buffer.rope.line_to_char(range.0 .0) + range.0 .1;
                    let end = self.buffer.rope.line_to_char(range.1 .0) + range.1 .1;
                    let _selected: String = self.buffer.rope.slice(start..end).chars().collect();
                    // TODO: Use copypasta crate for clipboard
                    self.buffer.delete(start, end - start);
                    self.cursor.line = range.0 .0;
                    self.cursor.col = range.0 .1;
                    self.cursor.clear_selection();
                    self.file_info.mark_modified();
                }
                return Task::none();
            }
            
            Message::Paste => {
                // TODO: Use copypasta crate for clipboard
                return Task::none();
            }
            
            Message::Undo => {
                self.buffer.undo();
                self.file_info.mark_modified();
            }
            
            Message::Redo => {
                self.buffer.redo();
                self.file_info.mark_modified();
            }
            
            Message::FindToggle => {
                self.find_state.toggle();
            }
            
            Message::ReplaceToggle => {
                self.find_state.toggle_replace();
                if self.find_state.show_replace && !self.find_state.show_find_bar {
                    self.find_state.show_find_bar = true;
                }
            }
            
            Message::FindQueryChanged(query) => {
                self.find_state.query = query;
                self.find_state.current_match = None;
            }
            
            Message::ReplaceTextChanged(text) => {
                self.find_state.replace_text = text;
            }
            
            Message::FindNext => {
                if let Some((line, col)) = self.find_state.find_next(&self.buffer.rope) {
                    self.cursor.line = line;
                    self.cursor.col = col;
                    self.cursor.clear_selection();
                }
            }
            
            Message::FindPrevious => {
                if let Some((line, col)) = self.find_state.find_previous(&self.buffer.rope) {
                    self.cursor.line = line;
                    self.cursor.col = col;
                    self.cursor.clear_selection();
                }
            }
            
            Message::ReplaceOne => {
                self.find_state.replace_one(&mut self.buffer.rope, self.cursor.line, self.cursor.col);
                self.file_info.mark_modified();
            }
            
            Message::ReplaceAll => {
                let count = self.find_state.replace_all(&mut self.buffer.rope);
                if count > 0 {
                    self.file_info.mark_modified();
                }
            }
            
            Message::ToggleLineNumbers => {
                self.line_numbers = !self.line_numbers;
            }
            
            Message::ToggleWordWrap => {
                self.word_wrap = !self.word_wrap;
            }
            
            Message::ToggleTheme => {
                self.theme = self.theme.toggle();
            }
            
            Message::Scroll(offset) => {
                self.scroll_offset = offset;
            }
            
            Message::Resized(size) => {
                self.viewport_height = size.height;
            }
        }
        
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let menu_bar = self.view_menu_bar();
        let editor_area = self.view_editor_area();
        let status_bar = self.view_status_bar();
        
        let content = column![
            menu_bar,
            editor_area,
            status_bar,
        ];
        
        container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    fn view_menu_bar(&self) -> Element<Message> {
        let file_menu = row![
            button("New").on_press(Message::NewFile),
            button("Open").on_press(Message::OpenFile),
            button("Save").on_press(Message::SaveFile),
            button("Save As").on_press(Message::SaveFileAs),
        ].spacing(5);
        
        let edit_menu = row![
            button("Undo").on_press(Message::Undo),
            button("Redo").on_press(Message::Redo),
            button("Copy").on_press(Message::Copy),
            button("Cut").on_press(Message::Cut),
            button("Paste").on_press(Message::Paste),
        ].spacing(5);
        
        let view_menu = row![
            button("Find").on_press(Message::FindToggle),
            button("Replace").on_press(Message::ReplaceToggle),
            button("Line Numbers").on_press(Message::ToggleLineNumbers),
            button("Word Wrap").on_press(Message::ToggleWordWrap),
            button("Theme").on_press(Message::ToggleTheme),
        ].spacing(5);
        
        row![file_menu, edit_menu, view_menu]
            .spacing(20)
            .into()
    }

    fn view_editor_area(&self) -> Element<Message> {
        let line_count = self.buffer.len_lines();
        let visible_lines = (self.viewport_height / self.line_height) as usize;
        let start_line = (self.scroll_offset / self.line_height) as usize;
        let end_line = (start_line + visible_lines).min(line_count);
        
        let mut editor_content = column![].spacing(0);
        
        for line_idx in start_line..end_line {
            let line_text: String = self.buffer.line(line_idx).chars().collect();
            let line_number = if self.line_numbers {
                format!("{:>4} │ ", line_idx + 1)
            } else {
                String::new()
            };
            
            let line_element = row![
                text(line_number).style(|_| text::Style {
                    color: self.theme.line_number_fg(),
                    ..Default::default()
                }),
                text(line_text),
            ];
            
            editor_content = editor_content.push(line_element);
        }
        
        let scrollable_content = scrollable(editor_content)
            .height(iced::Length::Fill)
            .on_scroll(|scroll| Message::Scroll(scroll.relative_offset().y * self.line_height));
        
        container(scrollable_content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(|_| container::Style {
                background: Some(self.theme.editor_bg().into()),
                text_color: Some(self.theme.editor_fg()),
                ..Default::default()
            })
            .into()
    }

    fn view_status_bar(&self) -> Element<Message> {
        let position = format!("Ln {}, Col {}", self.cursor.line + 1, self.cursor.col + 1);
        let file_name = self.file_info.filename();
        let modified = if self.file_info.is_modified { "Modified" } else { "Saved" };
        
        row![
            text(position),
            horizontal_space(),
            text(file_name),
            horizontal_space(),
            text(modified),
        ]
        .spacing(10)
        .into()
    }

    pub fn subscription(_state: &Editor) -> Subscription<Message> {
        keyboard::on_key_press(|key, modifiers| {
            match key {
                keyboard::Key::Character(c) => {
                    if modifiers.control() {
                        match c.as_str() {
                            "z" => Some(Message::Undo),
                            "y" => Some(Message::Redo),
                            "c" => Some(Message::Copy),
                            "x" => Some(Message::Cut),
                            "v" => Some(Message::Paste),
                            "o" => Some(Message::OpenFile),
                            "s" => Some(Message::SaveFile),
                            "n" => Some(Message::NewFile),
                            "f" => Some(Message::FindToggle),
                            "h" => Some(Message::ReplaceToggle),
                            _ => None,
                        }
                    } else {
                        Some(Message::InsertChar(c.chars().next().unwrap()))
                    }
                }
                keyboard::Key::Named(named) => {
                    match named {
                        key::Named::Enter => Some(Message::Newline),
                        key::Named::Backspace => Some(Message::DeleteBackward),
                        key::Named::Delete => Some(Message::DeleteForward),
                        key::Named::Tab => Some(Message::Tab),
                        key::Named::ArrowUp => {
                            if modifiers.shift() {
                                Some(Message::ExtendSelection(Box::new(Message::CursorUp)))
                            } else {
                                Some(Message::CursorUp)
                            }
                        }
                        key::Named::ArrowDown => {
                            if modifiers.shift() {
                                Some(Message::ExtendSelection(Box::new(Message::CursorDown)))
                            } else {
                                Some(Message::CursorDown)
                            }
                        }
                        key::Named::ArrowLeft => {
                            if modifiers.shift() {
                                Some(Message::ExtendSelection(Box::new(Message::CursorLeft)))
                            } else {
                                Some(Message::CursorLeft)
                            }
                        }
                        key::Named::ArrowRight => {
                            if modifiers.shift() {
                                Some(Message::ExtendSelection(Box::new(Message::CursorRight)))
                            } else {
                                Some(Message::CursorRight)
                            }
                        }
                        key::Named::Home => Some(Message::CursorHome),
                        key::Named::End => Some(Message::CursorEnd),
                        key::Named::PageUp => Some(Message::PageUp),
                        key::Named::PageDown => Some(Message::PageDown),
                        _ => None,
                    }
                }
                _ => None,
            }
        })
    }
}
