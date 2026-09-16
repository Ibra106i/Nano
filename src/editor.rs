use iced::{Element, Task, Subscription, Length, Color};
use iced::widget::{column, container, horizontal_space, row, scrollable};

use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::file_io::{self, FileInfo};
use crate::find::FindState;
use crate::syntax::SyntaxHighlighter;
use crate::theme::Theme as EditorTheme;
use crate::measure::TextMeasurer;
use crate::layout;
use crate::ui;

use std::path::PathBuf;
use copypasta::{ClipboardContext, ClipboardProvider};

impl Default for Editor {
    fn default() -> Self {
        Editor::new().0
    }
}

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
    pub show_sidebar: bool,
    pub zoom: f32,
    pub font_family: String,
    pub font_size: u32,
    pub word_count: usize,
    pub char_count: usize,
    pub last_mouse_pos: iced::Point,
    pub text_measurer: TextMeasurer,
    clipboard: Option<ClipboardContext>,
    pub bold_active: bool,
    pub italic_active: bool,
    pub underline_active: bool,
    pub strikethrough_active: bool,
    pub left_margin: f32,
    pub right_margin: f32,
    pub mouse_dragging: bool,
    clipboard_status: Option<String>,
    pending_file_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum Message {
    NewFile, OpenFile, SaveFile, SaveFileAs,
    FileOpened(Result<String, String>), FileSaved(Result<(), String>),
    InsertChar(char), DeleteBackward, DeleteForward, Newline, Tab,
    CursorUp, CursorDown, CursorLeft, CursorRight, CursorHome, CursorEnd, PageUp, PageDown,
    StartSelection, ExtendSelection(Box<Message>),
    SelectAll, Copy, Cut, Paste, Undo, Redo,
    Bold, Italic, Underline, Strikethrough,
    AlignLeft, AlignCenter, AlignRight, AlignJustify,
    FontFamilyChanged(String), FontSizeChanged(u32),
    FindToggle, ReplaceToggle, FindQueryChanged(String), ReplaceTextChanged(String),
    FindNext, FindPrevious, ReplaceOne, ReplaceAll,
    ToggleLineNumbers, ToggleWordWrap, ToggleTheme, ToggleSidebar,
    ZoomIn, ZoomOut, ZoomReset,
    Scroll(f32), Resized(iced::Size),
    CursorMoved(iced::Point),
    TextClicked,
    MouseDragEnd,
}

impl Editor {
    pub fn new() -> (Self, Task<Message>) {
        let mut editor = Editor {
            buffer: Buffer::from_str("The Architectural Future of Modular Interfaces\n\nModern application viewports have transcended the static bounding box of early GUI metaphors. As workflows converge toward instant collaboration and high-density state manipulation, interface layers require a decoupled architecture capable of contextual mutation without cognitive disruption.\n\n1. Contextual Surface Reconfiguration\n\nBy isolating the canvas sheet from ancillary tool surfaces, typography rendering engines preserve deterministic layout stability. This tactile boundary guarantees that document scaling remains exact regardless of external zoom multipliers or multi-window docking arrangements.\n\n2. Empirical Composition Performance\n\nComparative benchmarks across three production editorial pipelines highlight substantial latency drops when decoupled state trees govern character layout."),
            cursor: Cursor::new(),
            file_info: FileInfo::new(),
            theme: EditorTheme::Dark,
            syntax: SyntaxHighlighter::new(),
            find_state: FindState::new(),
            scroll_offset: 0.0,
            line_numbers: true,
            word_wrap: false,
            viewport_height: 800.0,
            line_height: 26.0,
            show_sidebar: true,
            zoom: 1.0,
            font_family: "Source Serif 4".to_string(),
            font_size: 12,
            word_count: 0,
            char_count: 0,
            last_mouse_pos: iced::Point::ORIGIN,
            text_measurer: TextMeasurer::new(),
            clipboard: ClipboardContext::new().ok(),
            bold_active: false,
            italic_active: false,
            underline_active: false,
            strikethrough_active: false,
            left_margin: 72.0,
            right_margin: 542.0,
            mouse_dragging: false,
            clipboard_status: None,
            pending_file_path: None,
        };
        editor.update_counts();
        (editor, Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NewFile => {
                self.buffer = Buffer::new();
                self.cursor = Cursor::new();
                self.file_info = FileInfo::new();
                self.update_counts();
            }
            Message::OpenFile => {
                if let Some(path) = file_io::open_file_dialog() {
                    self.pending_file_path = Some(path.clone());
                    let p = path.clone();
                    return Task::perform(async move { file_io::read_file(&p) }, |r| Message::FileOpened(r));
                }
            }
            Message::FileOpened(Ok(c)) => {
                self.buffer = Buffer::from_str(&c);
                self.cursor = Cursor::new();
                if let Some(ref path) = self.pending_file_path {
                    self.file_info = FileInfo::with_path(path.clone());
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    self.syntax.language = SyntaxHighlighter::detect_language(filename);
                } else {
                    self.file_info = FileInfo::new();
                }
                self.file_info.mark_saved();
                self.pending_file_path = None;
                self.update_counts();
            }
            Message::FileOpened(_) => {}
            Message::SaveFile => {
                if let Some(path) = &self.file_info.path.clone() {
                    let c = self.buffer.to_string();
                    let p = path.clone();
                    return Task::perform(async move { file_io::write_file(&p, &c) }, |r| Message::FileSaved(r));
                }
                return self.update(Message::SaveFileAs);
            }
            Message::SaveFileAs => {
                if let Some(path) = file_io::save_file_dialog() {
                    self.file_info.path = Some(path.clone());
                    let c = self.buffer.to_string();
                    return Task::perform(async move { file_io::write_file(&path, &c) }, |r| Message::FileSaved(r));
                }
            }
            Message::FileSaved(Ok(())) => self.file_info.mark_saved(),
            Message::FileSaved(_) => {}
            Message::InsertChar(ch) => {
                if ch == '\0' { return Task::none(); }
                self.delete_selection();
                let o = self.cursor.to_char_offset(&self.buffer.rope);
                let delta = self.compute_insert_word_delta(o, &ch.to_string());
                self.buffer.insert_char(o, ch);
                self.cursor.move_right(&self.buffer.rope);
                self.file_info.mark_modified();
                self.char_count = self.buffer.len_chars();
                self.apply_word_delta(delta);
            }
            Message::DeleteBackward => {
                if self.cursor.has_selection() {
                    self.delete_selection();
                    self.char_count = self.buffer.len_chars();
                    self.update_counts();
                } else {
                    let o = self.cursor.to_char_offset(&self.buffer.rope);
                    if o > 0 {
                        let delta = self.compute_delete_word_delta(o - 1, 1);
                        self.buffer.delete(o - 1, 1);
                        self.cursor.move_left(&self.buffer.rope);
                        self.apply_word_delta(delta);
                    }
                }
                self.file_info.mark_modified();
                self.char_count = self.buffer.len_chars();
            }
            Message::DeleteForward => {
                if self.cursor.has_selection() {
                    self.delete_selection();
                    self.char_count = self.buffer.len_chars();
                    self.update_counts();
                } else {
                    let o = self.cursor.to_char_offset(&self.buffer.rope);
                    if o < self.buffer.len_chars() {
                        let delta = self.compute_delete_word_delta(o, 1);
                        self.buffer.delete(o, 1);
                        self.apply_word_delta(delta);
                    }
                }
                self.file_info.mark_modified();
                self.char_count = self.buffer.len_chars();
            }
            Message::Newline => {
                self.delete_selection();
                let o = self.cursor.to_char_offset(&self.buffer.rope);
                let delta = self.compute_insert_word_delta(o, "\n");
                self.buffer.insert_str(o, "\n");
                self.cursor.move_down(&self.buffer.rope);
                self.cursor.move_home();
                self.file_info.mark_modified();
                self.char_count = self.buffer.len_chars();
                self.apply_word_delta(delta);
            }
            Message::Tab => {
                self.delete_selection();
                let o = self.cursor.to_char_offset(&self.buffer.rope);
                let delta = self.compute_insert_word_delta(o, "    ");
                self.buffer.insert_str(o, "    ");
                let line_len = self.buffer.line(self.cursor.line).len_chars();
                self.cursor.col = (self.cursor.col + 4).min(line_len);
                self.file_info.mark_modified();
                self.char_count = self.buffer.len_chars();
                self.apply_word_delta(delta);
            }
            Message::CursorUp => self.cursor.move_up(&self.buffer.rope),
            Message::CursorDown => self.cursor.move_down(&self.buffer.rope),
            Message::CursorLeft => self.cursor.move_left(&self.buffer.rope),
            Message::CursorRight => self.cursor.move_right(&self.buffer.rope),
            Message::CursorHome => self.cursor.move_home(),
            Message::CursorEnd => self.cursor.move_end(&self.buffer.rope),
            Message::PageUp => self.cursor.page_up(&self.buffer.rope, 20),
            Message::PageDown => self.cursor.page_down(&self.buffer.rope, 20),
            Message::StartSelection => self.cursor.start_selection(),
            Message::ExtendSelection(msg) => { if !self.cursor.has_selection() { self.cursor.start_selection(); } self.update(*msg); }
            Message::SelectAll => {
                let last_line = self.buffer.len_lines().saturating_sub(1);
                let last_col = self.buffer.line(last_line).len_chars();
                self.cursor.selection_start = Some((0, 0));
                self.cursor.line = last_line;
                self.cursor.col = last_col;
            }
            Message::Copy => {
                if let Some((start, end)) = self.cursor.selection_range() {
                    let text = self.buffer.rope.slice(
                        self.buffer.rope.line_to_char(start.0) + start.1
                            ..self.buffer.rope.line_to_char(end.0) + end.1
                    ).to_string();
                    if let Some(ref mut cb) = self.clipboard {
                        let _ = cb.set_contents(text);
                        self.clipboard_status = None;
                    } else {
                        self.clipboard_status = Some("Clipboard unavailable".to_string());
                    }
                }
            }
            Message::Cut => {
                if let Some((start, end)) = self.cursor.selection_range() {
                    let start_offset = self.buffer.rope.line_to_char(start.0) + start.1;
                    let end_offset = self.buffer.rope.line_to_char(end.0) + end.1;
                    let text = self.buffer.rope.slice(start_offset..end_offset).to_string();
                    if let Some(ref mut cb) = self.clipboard {
                        let _ = cb.set_contents(text);
                        self.clipboard_status = None;
                    } else {
                        self.clipboard_status = Some("Clipboard unavailable".to_string());
                    }
                    let delta = self.compute_delete_word_delta(start_offset, end_offset - start_offset);
                    self.buffer.delete(start_offset, end_offset - start_offset);
                    self.cursor.line = start.0;
                    self.cursor.col = start.1;
                    self.cursor.clear_selection();
                    self.file_info.mark_modified();
                    self.char_count = self.buffer.len_chars();
                    self.apply_word_delta(delta);
                }
            }
            Message::Paste => {
                if let Some(ref mut cb) = self.clipboard {
                    if let Ok(text) = cb.get_contents() {
                        self.delete_selection();
                        let o = self.cursor.to_char_offset(&self.buffer.rope);
                        let delta = self.compute_insert_word_delta(o, &text);
                        self.buffer.insert_str(o, &text);
                        for _ in 0..text.chars().count() {
                            self.cursor.move_right(&self.buffer.rope);
                        }
                        self.file_info.mark_modified();
                        self.char_count = self.buffer.len_chars();
                        self.apply_word_delta(delta);
                        self.clipboard_status = None;
                    } else {
                        self.clipboard_status = Some("Clipboard unavailable".to_string());
                    }
                } else {
                    self.clipboard_status = Some("Clipboard unavailable".to_string());
                }
            }
            Message::Undo => { self.buffer.undo(); self.file_info.mark_modified(); self.char_count = self.buffer.len_chars(); self.update_counts(); }
            Message::Redo => { self.buffer.redo(); self.file_info.mark_modified(); self.char_count = self.buffer.len_chars(); self.update_counts(); }
            Message::Bold => self.bold_active = !self.bold_active,
            Message::Italic => self.italic_active = !self.italic_active,
            Message::Underline => self.underline_active = !self.underline_active,
            Message::Strikethrough => self.strikethrough_active = !self.strikethrough_active,
            Message::AlignLeft | Message::AlignCenter | Message::AlignRight | Message::AlignJustify => {}
            Message::FontFamilyChanged(f) => self.font_family = f,
            Message::FontSizeChanged(s) => self.font_size = s,
            Message::FindToggle => self.find_state.toggle(),
            Message::ReplaceToggle => { self.find_state.toggle_replace(); if self.find_state.show_replace && !self.find_state.show_find_bar { self.find_state.show_find_bar = true; } }
            Message::FindQueryChanged(q) => { self.find_state.query = q; self.find_state.current_match = None; }
            Message::ReplaceTextChanged(t) => self.find_state.replace_text = t,
            Message::FindNext => { if let Some((l, c)) = self.find_state.find_next(&self.buffer.rope) { self.cursor.line = l; self.cursor.col = c; self.cursor.clear_selection(); } }
            Message::FindPrevious => { if let Some((l, c)) = self.find_state.find_previous(&self.buffer.rope) { self.cursor.line = l; self.cursor.col = c; self.cursor.clear_selection(); } }
            Message::ReplaceOne => { self.find_state.replace_one(&mut self.buffer.rope, self.cursor.line, self.cursor.col); self.file_info.mark_modified(); self.char_count = self.buffer.len_chars(); self.update_counts(); }
            Message::ReplaceAll => { if self.find_state.replace_all(&mut self.buffer.rope) > 0 { self.file_info.mark_modified(); self.char_count = self.buffer.len_chars(); self.update_counts(); } }
            Message::ToggleLineNumbers => self.line_numbers = !self.line_numbers,
            Message::ToggleWordWrap => self.word_wrap = !self.word_wrap,
            Message::ToggleTheme => self.theme = self.theme.toggle(),
            Message::ToggleSidebar => self.show_sidebar = !self.show_sidebar,
            Message::ZoomIn => self.zoom = (self.zoom + 0.1).min(2.0),
            Message::ZoomOut => self.zoom = (self.zoom - 0.1).max(0.5),
            Message::ZoomReset => self.zoom = 1.0,
            Message::Scroll(o) => self.scroll_offset = o,
            Message::Resized(s) => self.viewport_height = s.height,
            Message::CursorMoved(pos) => {
                self.last_mouse_pos = pos;
                if self.mouse_dragging {
                    let click_y = pos.y as f64 - layout::page_top() as f64 - layout::PAGE_PADDING as f64;
                    let click_x = pos.x as f64 - layout::page_left() as f64 - layout::PAGE_PADDING as f64;

                    if click_y >= 0.0 && click_x >= 0.0 {
                        let line_h = self.line_height as f64;
                        let clicked_line = (click_y / line_h) as usize;
                        let line_count = self.buffer.len_lines();
                        if clicked_line < line_count {
                            self.cursor.line = clicked_line;
                            let line_str: String = self.buffer.line(clicked_line).chars().collect();
                            let line_len = line_str.len();
                            let font_size = if clicked_line == 0 { 30.0 } else if line_str.starts_with(|c: char| c.is_numeric()) { 18.0 } else { 15.0 };
                            let clicked_col = self.text_measurer.col_from_x(&line_str, click_x as f32, font_size);
                            self.cursor.col = clicked_col.min(line_len);
                        }
                    }
                }
            }
            Message::TextClicked => {
                self.mouse_dragging = true;
                let click_y = self.last_mouse_pos.y as f64 - layout::page_top() as f64 - layout::PAGE_PADDING as f64;
                let click_x = self.last_mouse_pos.x as f64 - layout::page_left() as f64 - layout::PAGE_PADDING as f64;

                if click_y < 0.0 || click_x < 0.0 { return Task::none(); }

                let line_h = self.line_height as f64;
                let clicked_line = (click_y / line_h) as usize;
                let line_count = self.buffer.len_lines();
                if clicked_line < line_count {
                    self.cursor.clear_selection();
                    self.cursor.line = clicked_line;
                    let line_str: String = self.buffer.line(clicked_line).chars().collect();
                    let line_len = line_str.len();
                    let font_size = if clicked_line == 0 { 30.0 } else if line_str.starts_with(|c: char| c.is_numeric()) { 18.0 } else { 15.0 };
                    let clicked_col = self.text_measurer.col_from_x(&line_str, click_x as f32, font_size);
                    self.cursor.col = clicked_col.min(line_len);
                    self.cursor.start_selection();
                }
            }
            Message::MouseDragEnd => {
                self.mouse_dragging = false;
            }
        }
        Task::none()
    }

    fn update_counts(&mut self) {
        self.char_count = self.buffer.len_chars();
        let t = self.buffer.to_string();
        self.word_count = t.split_whitespace().count();
    }

    fn compute_insert_word_delta(&self, offset: usize, inserted: &str) -> i32 {
        if inserted.trim().is_empty() {
            return 0;
        }
        let context_chars = 200;
        let start = offset.saturating_sub(context_chars);
        let end = (offset + context_chars).min(self.buffer.len_chars());
        let context: String = self.buffer.rope.slice(start..end).chars().collect();
        let local_offset = offset - start;

        let before_text = &context[..local_offset];
        let after_text = &context[local_offset..];

        let before_words = before_text.split_whitespace().count();
        let after_words = (before_text.to_string() + inserted + after_text).split_whitespace().count();

        (after_words as i32) - (before_words as i32)
    }

    fn compute_delete_word_delta(&self, offset: usize, length: usize) -> i32 {
        if length == 0 {
            return 0;
        }
        let context_chars = 200;
        let start = offset.saturating_sub(context_chars);
        let end = (offset + length + context_chars).min(self.buffer.len_chars());
        let context: String = self.buffer.rope.slice(start..end).chars().collect();
        let local_offset = offset - start;
        let local_end = local_offset + length.min(end - offset);

        let before_text = &context[..local_offset];
        let deleted_text = &context[local_offset..local_end];
        let after_text = &context[local_end..];

        let before_words = before_text.split_whitespace().count();
        let after_words = (before_text.to_string() + after_text).split_whitespace().count();

        (after_words as i32) - (before_words as i32)
    }

    fn apply_word_delta(&mut self, delta: i32) {
        self.word_count = (self.word_count as i32 + delta).max(0) as usize;
    }

    fn delete_selection(&mut self) {
        if let Some((start, end)) = self.cursor.selection_range() {
            let start_offset = self.buffer.rope.line_to_char(start.0) + start.1;
            let end_offset = self.buffer.rope.line_to_char(end.0) + end.1;
            self.buffer.delete(start_offset, end_offset - start_offset);
            self.cursor.line = start.0;
            self.cursor.col = start.1;
            self.cursor.clear_selection();
        }
    }

    pub fn view(&self) -> Element<Message> {
        let ctx = ui::ViewContext {
            file_info_name: &self.file_info.filename(),
            file_is_modified: self.file_info.is_modified,
            theme: &self.theme,
            font_family: &self.font_family,
            font_size: self.font_size,
            bold_active: self.bold_active,
            italic_active: self.italic_active,
            underline_active: self.underline_active,
            strikethrough_active: self.strikethrough_active,
            buffer: &self.buffer,
            cursor: &self.cursor,
            line_numbers: self.line_numbers,
            show_sidebar: self.show_sidebar,
            word_count: self.word_count,
            char_count: self.char_count,
            zoom: self.zoom,
            clipboard_status: self.clipboard_status.as_deref(),
            find_state: &self.find_state,
            syntax: &self.syntax,
            line_height: self.line_height,
        };

        let title_bar = ui::title_bar(&ctx);
        let menu_bar = ui::menu_bar(&ctx);
        let toolbar = ui::toolbar(&ctx);
        let ruler = ui::ruler(&ctx);
        let find_bar = ui::find_bar(&ctx);
        let ln_gutter = ui::line_numbers_gutter(&ctx);
        let page = ui::page_content(&ctx);
        let status_bar = ui::status_bar(&ctx);

        let scroll_page = scrollable(page)
            .height(Length::Fill)
            .width(Length::Fill)
            .on_scroll(|s| Message::Scroll(s.relative_offset().y * self.line_height));

        let page_container = container(scroll_page)
            .width(layout::PAGE_WIDTH)
            .height(Length::Fill)
            .padding(1)
            .style(|_: &iced::Theme| container::Style {
                background: Some(self.theme.canvas_bg().into()),
                border: iced::Border::default().rounded(2).color(self.theme.border()).width(1),
                ..Default::default()
            });

        let editor_row = row![ln_gutter, horizontal_space().width(layout::GUTTER_SPACING), page_container, horizontal_space().width(layout::GUTTER_SPACING)].height(Length::Fill);

        let main_area = if self.show_sidebar {
            let sidebar = ui::sidebar_container(&ctx);
            row![editor_row, sidebar].height(Length::Fill)
        } else {
            editor_row
        };

        let layout_col = column![title_bar, menu_bar, toolbar, ruler, find_bar, main_area, status_bar];

        container(layout_col).width(Length::Fill).height(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(self.theme.canvas_bg().into()),
                text_color: Some(self.theme.text_primary()),
                ..Default::default()
            }).into()
    }
}

impl Editor {
    pub fn subscription(_state: &Editor) -> Subscription<Message> {
        iced::event::listen().map(|event| {
            match event {
                iced::event::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                    Message::CursorMoved(position)
                }
                iced::event::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                    Message::TextClicked
                }
                iced::event::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                    Message::MouseDragEnd
                }
                iced::event::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                    match key {
                        iced::keyboard::Key::Character(c) => {
                            if modifiers.control() {
                                match c.as_ref() {
                                    "z" => Message::Undo,
                                    "y" => Message::Redo,
                                    "c" => Message::Copy,
                                    "x" => Message::Cut,
                                    "v" => Message::Paste,
                                    "a" => Message::SelectAll,
                                    "o" => Message::OpenFile,
                                    "s" => Message::SaveFile,
                                    "n" => Message::NewFile,
                                    "f" => Message::FindToggle,
                                    "h" => Message::ReplaceToggle,
                                    "=" => Message::ZoomIn,
                                    "-" => Message::ZoomOut,
                                    "0" => Message::ZoomReset,
                                    _ => Message::InsertChar('\0'),
                                }
                            } else {
                                Message::InsertChar(c.chars().next().unwrap())
                            }
                        }
                        iced::keyboard::Key::Named(n) => match n {
                            iced::keyboard::key::Named::Enter => Message::Newline,
                            iced::keyboard::key::Named::Backspace => Message::DeleteBackward,
                            iced::keyboard::key::Named::Delete => Message::DeleteForward,
                            iced::keyboard::key::Named::Tab => Message::Tab,
                            iced::keyboard::key::Named::Space => Message::InsertChar(' '),
                            iced::keyboard::key::Named::ArrowUp => {
                                if modifiers.shift() { Message::ExtendSelection(Box::new(Message::CursorUp)) } else { Message::CursorUp }
                            }
                            iced::keyboard::key::Named::ArrowDown => {
                                if modifiers.shift() { Message::ExtendSelection(Box::new(Message::CursorDown)) } else { Message::CursorDown }
                            }
                            iced::keyboard::key::Named::ArrowLeft => {
                                if modifiers.shift() { Message::ExtendSelection(Box::new(Message::CursorLeft)) } else { Message::CursorLeft }
                            }
                            iced::keyboard::key::Named::ArrowRight => {
                                if modifiers.shift() { Message::ExtendSelection(Box::new(Message::CursorRight)) } else { Message::CursorRight }
                            }
                            iced::keyboard::key::Named::Home => Message::CursorHome,
                            iced::keyboard::key::Named::End => Message::CursorEnd,
                            iced::keyboard::key::Named::PageUp => Message::PageUp,
                            iced::keyboard::key::Named::PageDown => Message::PageDown,
                            _ => Message::InsertChar('\0'),
                        },
                        _ => Message::InsertChar('\0'),
                    }
                }
                _ => Message::InsertChar('\0'),
            }
        })
    }
}
