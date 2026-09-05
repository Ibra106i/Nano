use iced::{Element, Task, Subscription, Length, Color};
use iced::widget::{button, column, container, horizontal_space, row, scrollable, text};

use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::file_io::{self, FileInfo};
use crate::find::FindState;
use crate::syntax::SyntaxHighlighter;
use crate::theme::Theme as EditorTheme;
use crate::measure::TextMeasurer;

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
    pub show_sidebar: bool,
    pub zoom: f32,
    pub font_family: String,
    pub font_size: u32,
    pub word_count: usize,
    pub char_count: usize,
    pub last_mouse_pos: iced::Point,
    pub text_measurer: TextMeasurer,
}

#[derive(Debug, Clone)]
pub enum Message {
    NewFile, OpenFile, SaveFile, SaveFileAs,
    FileOpened(Result<String, String>), FileSaved(Result<(), String>),
    InsertChar(char), DeleteBackward, DeleteForward, Newline, Tab,
    CursorUp, CursorDown, CursorLeft, CursorRight, CursorHome, CursorEnd, PageUp, PageDown,
    StartSelection, ExtendSelection(Box<Message>),
    Copy, Cut, Paste, Undo, Redo,
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
}

impl Editor {
    pub fn new() -> (Self, Task<Message>) {
        (
            Editor {
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
                word_count: 348,
                char_count: 2140,
                last_mouse_pos: iced::Point::ORIGIN,
                text_measurer: TextMeasurer::new(),
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NewFile => {
                self.buffer = Buffer::new();
                self.cursor = Cursor::new();
                self.file_info = FileInfo::new();
            }
            Message::OpenFile => {
                if let Some(path) = file_io::open_file_dialog() {
                    let p = path.clone();
                    return Task::perform(async move { file_io::read_file(&p) }, |r| Message::FileOpened(r));
                }
            }
            Message::FileOpened(Ok(c)) => {
                self.buffer = Buffer::from_str(&c);
                self.cursor = Cursor::new();
                self.file_info = FileInfo::new();
                self.file_info.mark_saved();
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
                let o = self.cursor.to_byte_offset(&self.buffer.rope);
                self.buffer.insert_char(o, ch);
                self.cursor.move_right(&self.buffer.rope);
                self.file_info.mark_modified();
                self.update_counts();
            }
            Message::DeleteBackward => {
                let o = self.cursor.to_byte_offset(&self.buffer.rope);
                if o > 0 { self.buffer.delete(o - 1, 1); self.cursor.move_left(&self.buffer.rope); self.file_info.mark_modified(); self.update_counts(); }
            }
            Message::DeleteForward => {
                let o = self.cursor.to_byte_offset(&self.buffer.rope);
                if o < self.buffer.len_chars() { self.buffer.delete(o, 1); self.file_info.mark_modified(); self.update_counts(); }
            }
            Message::Newline => {
                let o = self.cursor.to_byte_offset(&self.buffer.rope);
                self.buffer.insert_str(o, "\n");
                self.cursor.move_down(&self.buffer.rope);
                self.cursor.move_home();
                self.file_info.mark_modified();
                self.update_counts();
            }
            Message::Tab => {
                let o = self.cursor.to_byte_offset(&self.buffer.rope);
                self.buffer.insert_str(o, "    ");
                self.cursor.col += 4;
                self.file_info.mark_modified();
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
            Message::Copy | Message::Cut | Message::Paste => {}
            Message::Undo => { self.buffer.undo(); self.file_info.mark_modified(); self.update_counts(); }
            Message::Redo => { self.buffer.redo(); self.file_info.mark_modified(); self.update_counts(); }
            Message::Bold | Message::Italic | Message::Underline | Message::Strikethrough => {}
            Message::AlignLeft | Message::AlignCenter | Message::AlignRight | Message::AlignJustify => {}
            Message::FontFamilyChanged(f) => self.font_family = f,
            Message::FontSizeChanged(s) => self.font_size = s,
            Message::FindToggle => self.find_state.toggle(),
            Message::ReplaceToggle => { self.find_state.toggle_replace(); if self.find_state.show_replace && !self.find_state.show_find_bar { self.find_state.show_find_bar = true; } }
            Message::FindQueryChanged(q) => { self.find_state.query = q; self.find_state.current_match = None; }
            Message::ReplaceTextChanged(t) => self.find_state.replace_text = t,
            Message::FindNext => { if let Some((l, c)) = self.find_state.find_next(&self.buffer.rope) { self.cursor.line = l; self.cursor.col = c; self.cursor.clear_selection(); } }
            Message::FindPrevious => { if let Some((l, c)) = self.find_state.find_previous(&self.buffer.rope) { self.cursor.line = l; self.cursor.col = c; self.cursor.clear_selection(); } }
            Message::ReplaceOne => { self.find_state.replace_one(&mut self.buffer.rope, self.cursor.line, self.cursor.col); self.file_info.mark_modified(); self.update_counts(); }
            Message::ReplaceAll => { if self.find_state.replace_all(&mut self.buffer.rope) > 0 { self.file_info.mark_modified(); self.update_counts(); } }
            Message::ToggleLineNumbers => self.line_numbers = !self.line_numbers,
            Message::ToggleWordWrap => self.word_wrap = !self.word_wrap,
            Message::ToggleTheme => self.theme = self.theme.toggle(),
            Message::ToggleSidebar => self.show_sidebar = !self.show_sidebar,
            Message::ZoomIn => self.zoom = (self.zoom + 0.1).min(2.0),
            Message::ZoomOut => self.zoom = (self.zoom - 0.1).max(0.5),
            Message::ZoomReset => self.zoom = 1.0,
            Message::Scroll(o) => self.scroll_offset = o,
            Message::Resized(s) => self.viewport_height = s.height,
            Message::CursorMoved(pos) => self.last_mouse_pos = pos,
            Message::TextClicked => {
                self.cursor.clear_selection();
                let title_bar_h = 38.0;
                let menu_bar_h = 32.0;
                let toolbar_h = 38.0;
                let ruler_h = 22.0;
                let page_top = title_bar_h + menu_bar_h + toolbar_h + ruler_h;
                let page_padding = 24.0;
                let page_left = 40.0 + 16.0 + 1.0;

                let click_y = self.last_mouse_pos.y as f64 - page_top as f64 - page_padding as f64;
                let click_x = self.last_mouse_pos.x as f64 - page_left as f64 - page_padding as f64;

                if click_y < 0.0 || click_x < 0.0 { return Task::none(); }

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
        Task::none()
    }

    fn update_counts(&mut self) {
        let t = self.buffer.to_string();
        self.char_count = t.len();
        self.word_count = t.split_whitespace().count();
    }

    pub fn view(&self) -> Element<Message> {
        let dirty = if self.file_info.is_modified { " ●" } else { "" };
        let title = format!("Nano — {}{}", self.file_info.filename(), dirty);
        let title_bar = row![horizontal_space(), text(title).size(13).color(self.theme.text_secondary()), horizontal_space()]
            .height(38).align_y(iced::Alignment::Center);

        let menus = ["File", "Edit", "View", "Insert", "Format", "Tools", "Help"];
        let menu_items: Vec<Element<Message>> = menus.iter().map(|m| {
            button(text(*m).size(12).color(self.theme.text_primary())).padding([5, 10]).into()
        }).collect();
        let menu_bar = row(menu_items).spacing(2).padding([0, 8]);

        let tool_btn = |label: String, msg: Message, active: bool| -> Element<'static, Message> {
            if active {
                button(text(label).size(13).color(Color::WHITE)).padding([4, 8]).style(button::primary).on_press(msg).into()
            } else {
                button(text(label).size(13).color(Color::from_rgb(0.85, 0.85, 0.9))).padding([4, 8]).on_press(msg).into()
            }
        };

        let sep = || -> Element<'static, Message> { text(" | ").size(13).color(Color::from_rgb(0.3, 0.3, 0.4)).into() };

        let toolbar = row![
            tool_btn("New".into(), Message::NewFile, false),
            tool_btn("Open".into(), Message::OpenFile, false),
            tool_btn("Save".into(), Message::SaveFile, false),
            sep(),
            tool_btn("Cut".into(), Message::Cut, false),
            tool_btn("Copy".into(), Message::Copy, false),
            tool_btn("Paste".into(), Message::Paste, false),
            sep(),
            tool_btn("Undo".into(), Message::Undo, false),
            tool_btn("Redo".into(), Message::Redo, false),
            sep(),
            tool_btn("B".into(), Message::Bold, true),
            tool_btn("I".into(), Message::Italic, false),
            tool_btn("U".into(), Message::Underline, false),
            tool_btn("S".into(), Message::Strikethrough, false),
            sep(),
            tool_btn("Left".into(), Message::AlignLeft, true),
            tool_btn("Center".into(), Message::AlignCenter, false),
            tool_btn("Right".into(), Message::AlignRight, false),
            tool_btn("Justify".into(), Message::AlignJustify, false),
            sep(),
            tool_btn("List".into(), Message::ToggleLineNumbers, false),
            tool_btn("Nums".into(), Message::ToggleWordWrap, false),
            sep(),
            tool_btn("Find".into(), Message::FindToggle, false),
            tool_btn("Replace".into(), Message::ReplaceToggle, false),
        ].spacing(3).align_y(iced::Alignment::Center).padding([0, 8]);

        let ruler_marks: Vec<Element<Message>> = (0..8).map(|i| {
            row![horizontal_space().width(80), text(format!("{}", i)).size(10).color(Color::from_rgb(0.4, 0.4, 0.5))].into()
        }).collect();
        let ruler = row(ruler_marks).width(Length::Fill).height(22);

        let line_count = self.buffer.len_lines();
        let mut ln_col = column![].spacing(4).padding([0, 8]);
        for i in 1..=line_count.min(50) {
            let is_cur = i - 1 == self.cursor.line;
            ln_col = ln_col.push(text(format!("{:>3}", i)).size(12).color(
                if is_cur { Color::from_rgb(0.6, 0.8, 1.0) } else { Color::from_rgb(0.35, 0.35, 0.45) }
            ));
        }
        let line_numbers = container(ln_col).width(40).height(Length::Fill);

        let sel_color = self.theme.page_selection();
        let has_sel = self.cursor.has_selection();
        let sel_range = self.cursor.selection_range();

        let mut page_content = column![].spacing(6).padding(24);
        for li in 0..line_count.min(50) {
            let lt: String = self.buffer.line(li).chars().collect();
            let lt_len = lt.len();
            let sz = if li == 0 { 30.0 } else if lt.starts_with(|c: char| c.is_numeric()) { 18.0 } else { 15.0 };
            let clr = if li == 0 { Color::from_rgb(0.95, 0.95, 1.0) }
                      else if lt.starts_with(|c: char| c.is_numeric()) { Color::from_rgb(0.85, 0.85, 0.92) }
                      else { Color::from_rgb(0.75, 0.75, 0.82) };

            let is_cursor_line = li == self.cursor.line;

            if has_sel {
                let (s_line, s_col) = sel_range.unwrap().0;
                let (e_line, e_col) = sel_range.unwrap().1;

                let sel_start = if li == s_line { s_col } else { 0 };
                let sel_end = if li == e_line { e_col } else { lt_len };

                let has_sel_on_line = li >= s_line && li <= e_line && sel_start < sel_end;

                if has_sel_on_line {
                    let before: String = lt.chars().take(sel_start).collect();
                    let selected: String = lt.chars().skip(sel_start).take(sel_end - sel_start).collect();
                    let after: String = lt.chars().skip(sel_end).collect();

                    let mut parts: Vec<Element<Message>> = Vec::new();
                    if !before.is_empty() {
                        parts.push(text(before).size(sz).color(clr).into());
                    }
                    if !selected.is_empty() {
                        parts.push(
                            container(text(selected).size(sz).color(Color::WHITE))
                                .style(move |_: &iced::Theme| container::Style {
                                    background: Some(sel_color.into()),
                                    border: iced::Border::default().rounded(2),
                                    ..Default::default()
                                }).into()
                        );
                    }
                    if !after.is_empty() {
                        parts.push(text(after).size(sz).color(clr).into());
                    }

                    if is_cursor_line {
                        let before_cur: String = lt.chars().take(self.cursor.col).collect();
                        let after_cur: String = lt.chars().skip(self.cursor.col).collect();
                        parts = Vec::new();
                        if !before_cur.is_empty() {
                            parts.push(text(before_cur).size(sz).color(clr).into());
                        }
                        parts.push(
                            container(text(" ").size(sz)).width(2).height(iced::Length::Fixed(sz))
                                .style(|_: &iced::Theme| container::Style {
                                    background: Some(Color::from_rgb(0.4, 0.6, 1.0).into()),
                                    ..Default::default()
                                }).into()
                        );
                        if !after_cur.is_empty() {
                            parts.push(text(after_cur).size(sz).color(clr).into());
                        }
                    }

                    page_content = page_content.push(row(parts).align_y(iced::Alignment::Center));
                } else if is_cursor_line {
                    let before: String = lt.chars().take(self.cursor.col).collect();
                    let after: String = lt.chars().skip(self.cursor.col).collect();
                    let line_row = row![
                        text(before).size(sz).color(clr),
                        container(text(" ").size(sz)).width(2).height(iced::Length::Fixed(sz)).style(|_: &iced::Theme| container::Style {
                            background: Some(Color::from_rgb(0.4, 0.6, 1.0).into()),
                            ..Default::default()
                        }),
                        text(after).size(sz).color(clr),
                    ].align_y(iced::Alignment::Center);
                    page_content = page_content.push(line_row);
                } else {
                    page_content = page_content.push(text(lt).size(sz).color(clr));
                }
            } else if is_cursor_line {
                let before: String = lt.chars().take(self.cursor.col).collect();
                let after: String = lt.chars().skip(self.cursor.col).collect();
                let line_row = row![
                    text(before).size(sz).color(clr),
                    container(text(" ").size(sz)).width(2).height(iced::Length::Fixed(sz)).style(|_: &iced::Theme| container::Style {
                        background: Some(Color::from_rgb(0.4, 0.6, 1.0).into()),
                        ..Default::default()
                    }),
                    text(after).size(sz).color(clr),
                ].align_y(iced::Alignment::Center);
                page_content = page_content.push(line_row);
            } else {
                page_content = page_content.push(text(lt).size(sz).color(clr));
            }
        }

        let scroll_page = scrollable(page_content)
            .height(Length::Fill)
            .width(Length::Fill)
            .on_scroll(|s| Message::Scroll(s.relative_offset().y * self.line_height));

        let page = container(scroll_page)
            .width(614)
            .height(Length::Fill)
            .padding(1)
            .style(|_: &iced::Theme| container::Style {
                background: Some(Color::from_rgb(0.12, 0.12, 0.2).into()),
                border: iced::Border::default().rounded(2).color(Color::from_rgb(0.25, 0.25, 0.35)).width(1),
                ..Default::default()
            });

        let editor_row = row![line_numbers, horizontal_space().width(16), page, horizontal_space().width(16)].height(Length::Fill);

        let sidebar_content = column![
            row![text("FORMAT INSPECTOR").size(11).color(Color::from_rgb(0.8, 0.8, 0.9)), horizontal_space()],
            text("PARAGRAPH STYLE").size(11).color(Color::from_rgb(0.5, 0.5, 0.6)),
            row![
                tool_btn("Normal".into(), Message::AlignLeft, true),
                tool_btn("H1".into(), Message::Bold, false),
                tool_btn("H2".into(), Message::Bold, false),
            ].spacing(4),
            row![
                tool_btn("H3".into(), Message::Bold, false),
                tool_btn("Quote".into(), Message::AlignLeft, false),
                tool_btn("Code".into(), Message::AlignLeft, false),
            ].spacing(4),
            text("TYPOGRAPHY").size(11).color(Color::from_rgb(0.5, 0.5, 0.6)),
            row![text("Source Serif 4").size(12), horizontal_space(), text("Serif Editorial").size(10).color(Color::from_rgb(0.4, 0.4, 0.5))],
            row![
                tool_btn("B".into(), Message::Bold, false),
                tool_btn("I".into(), Message::Italic, false),
                tool_btn("U".into(), Message::Underline, false),
                tool_btn("S".into(), Message::Strikethrough, false),
            ].spacing(8),
            kv("Tracking", "-0.015 EM"),
            kv("Scale", "100%"),
            text("PARAGRAPH & SPACING").size(11).color(Color::from_rgb(0.5, 0.5, 0.6)),
            kv("Line", "1.65"),
            kv("Before", "0 pt"),
            kv("After", "6 pt"),
            kv("First Line Indent", "0.00 in"),
            text("PAGE CANVAS SETUP").size(11).color(Color::from_rgb(0.5, 0.5, 0.6)),
            kv("Page", "US Letter • Portrait"),
        ].spacing(6).padding(12);

        let sidebar = container(sidebar_content).width(280).height(Length::Fill)
            .style(container::bordered_box);

        let main_area = if self.show_sidebar {
            row![editor_row, sidebar].height(Length::Fill)
        } else {
            editor_row
        };

        let status_left = row![
            text(format!("Page 1 of 1 | Word count: {} | Characters: {} | UTF-8", self.word_count, self.char_count))
                .size(11).color(Color::from_rgb(0.6, 0.6, 0.7)),
        ];
        let zoom_pct = format!("{}%", (self.zoom * 100.0) as u32);
        let status_right = row![
            button(text("−").size(12)).padding(4).on_press(Message::ZoomOut),
            button(text(zoom_pct).size(11)).padding(4).on_press(Message::ZoomReset),
            button(text("+").size(12)).padding(4).on_press(Message::ZoomIn),
        ].spacing(4).align_y(iced::Alignment::Center);

        let status_bar = row![status_left, horizontal_space(), status_right]
            .padding([0, 12]).height(26).align_y(iced::Alignment::Center);

        let layout = column![title_bar, menu_bar, toolbar, ruler, main_area, status_bar];

        container(layout).width(Length::Fill).height(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(iced::Color::from_rgb(0.07, 0.07, 0.14).into()),
                text_color: Some(iced::Color::from_rgb(0.88, 0.88, 0.92)),
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

fn btn<'a>(label: &'a str, active: bool) -> Element<'a, Message> {
    if active {
        button(text(label).size(11).color(Color::WHITE)).padding([5, 12]).style(button::primary).into()
    } else {
        button(text(label).size(11)).padding([5, 12]).into()
    }
}

fn kv<'a>(key: &'a str, val: &'a str) -> Element<'a, Message> {
    row![text(key).size(11), horizontal_space(), text(val).size(11)].into()
}
