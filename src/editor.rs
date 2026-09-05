use iced::{Element, Task, Subscription, Length, Color};
use iced::widget::{button, column, container, horizontal_space, row, scrollable, text, text_input};

use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::file_io::{self, FileInfo};
use crate::find::FindState;
use crate::syntax::SyntaxHighlighter;
use crate::theme::Theme as EditorTheme;
use crate::measure::TextMeasurer;

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
    clipboard: ClipboardContext,
    pub bold_active: bool,
    pub italic_active: bool,
    pub underline_active: bool,
    pub strikethrough_active: bool,
    pub left_margin: f32,
    pub right_margin: f32,
    pub mouse_dragging: bool,
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
                clipboard: ClipboardContext::new().unwrap(),
                bold_active: false,
                italic_active: false,
                underline_active: false,
                strikethrough_active: false,
                left_margin: 72.0,
                right_margin: 542.0,
                mouse_dragging: false,
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
                self.delete_selection();
                let o = self.cursor.to_byte_offset(&self.buffer.rope);
                self.buffer.insert_char(o, ch);
                self.cursor.move_right(&self.buffer.rope);
                self.file_info.mark_modified();
                self.update_counts();
            }
            Message::DeleteBackward => {
                if self.cursor.has_selection() {
                    self.delete_selection();
                } else {
                    let o = self.cursor.to_byte_offset(&self.buffer.rope);
                    if o > 0 { self.buffer.delete(o - 1, 1); self.cursor.move_left(&self.buffer.rope); }
                }
                self.file_info.mark_modified();
                self.update_counts();
            }
            Message::DeleteForward => {
                if self.cursor.has_selection() {
                    self.delete_selection();
                } else {
                    let o = self.cursor.to_byte_offset(&self.buffer.rope);
                    if o < self.buffer.len_chars() { self.buffer.delete(o, 1); }
                }
                self.file_info.mark_modified();
                self.update_counts();
            }
            Message::Newline => {
                self.delete_selection();
                let o = self.cursor.to_byte_offset(&self.buffer.rope);
                self.buffer.insert_str(o, "\n");
                self.cursor.move_down(&self.buffer.rope);
                self.cursor.move_home();
                self.file_info.mark_modified();
                self.update_counts();
            }
            Message::Tab => {
                self.delete_selection();
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
                    let _ = self.clipboard.set_contents(text);
                }
            }
            Message::Cut => {
                if let Some((start, end)) = self.cursor.selection_range() {
                    let start_offset = self.buffer.rope.line_to_char(start.0) + start.1;
                    let end_offset = self.buffer.rope.line_to_char(end.0) + end.1;
                    let text = self.buffer.rope.slice(start_offset..end_offset).to_string();
                    let _ = self.clipboard.set_contents(text);
                    self.buffer.delete(start_offset, end_offset - start_offset);
                    self.cursor.line = start.0;
                    self.cursor.col = start.1;
                    self.cursor.clear_selection();
                    self.file_info.mark_modified();
                    self.update_counts();
                }
            }
            Message::Paste => {
                if let Ok(text) = self.clipboard.get_contents() {
                    self.delete_selection();
                    let o = self.cursor.to_byte_offset(&self.buffer.rope);
                    self.buffer.insert_str(o, &text);
                    for _ in 0..text.chars().count() {
                        self.cursor.move_right(&self.buffer.rope);
                    }
                    self.file_info.mark_modified();
                    self.update_counts();
                }
            }
            Message::Undo => { self.buffer.undo(); self.file_info.mark_modified(); self.update_counts(); }
            Message::Redo => { self.buffer.redo(); self.file_info.mark_modified(); self.update_counts(); }
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
            Message::CursorMoved(pos) => {
                self.last_mouse_pos = pos;
                if self.mouse_dragging {
                    let title_bar_h = 38.0;
                    let menu_bar_h = 32.0;
                    let toolbar_h = 38.0;
                    let ruler_h = 22.0;
                    let page_top = title_bar_h + menu_bar_h + toolbar_h + ruler_h;
                    let page_padding = 24.0;
                    let page_left = 40.0 + 16.0 + 1.0;

                    let click_y = pos.y as f64 - page_top as f64 - page_padding as f64;
                    let click_x = pos.x as f64 - page_left as f64 - page_padding as f64;

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
        let t = self.buffer.to_string();
        self.char_count = t.len();
        self.word_count = t.split_whitespace().count();
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
            let style = move |theme: &iced::Theme, status: button::Status| -> button::Style {
                if active {
                    button::primary(theme, status)
                } else {
                    button::Style {
                        background: Some(Color::from_rgba(0.17, 0.18, 0.30, 0.6).into()),
                        text_color: Color::from_rgb(0.85, 0.85, 0.9),
                        border: iced::Border::default().rounded(4).color(Color::from_rgba(0.25, 0.25, 0.35, 0.5)).width(1),
                        ..button::primary(theme, status)
                    }
                }
            };
            button(text(label).size(14).color(if active { Color::WHITE } else { Color::from_rgb(0.85, 0.85, 0.9) }))
                .padding([4, 8]).style(style).on_press(msg).into()
        };

        let sep = || -> Element<'static, Message> { text(" | ").size(13).color(Color::from_rgb(0.3, 0.3, 0.4)).into() };

        let toolbar = row![
            tool_btn("\u{1F4C4}".into(), Message::NewFile, false),
            tool_btn("\u{1F4C2}".into(), Message::OpenFile, false),
            tool_btn("\u{1F4BE}".into(), Message::SaveFile, false),
            sep(),
            tool_btn("\u{2702}".into(), Message::Cut, false),
            tool_btn("\u{1F4CB}".into(), Message::Copy, false),
            tool_btn("\u{1F4CF}".into(), Message::Paste, false),
            sep(),
            tool_btn("\u{21A9}".into(), Message::Undo, false),
            tool_btn("\u{21AA}".into(), Message::Redo, false),
            sep(),
            {
                let font_label: Element<'static, Message> = container(row![
                    text(self.font_family.clone()).size(12).color(Color::from_rgb(0.85, 0.85, 0.9)),
                    text(" \u{25BE}").size(10).color(Color::from_rgb(0.5, 0.5, 0.6)),
                ].align_y(iced::Alignment::Center))
                    .padding([4, 8])
                    .style(|_: &iced::Theme| container::Style {
                        background: Some(Color::from_rgba(0.17, 0.18, 0.30, 0.6).into()),
                        border: iced::Border::default().rounded(4).color(Color::from_rgba(0.25, 0.25, 0.35, 0.5)).width(1),
                        ..Default::default()
                    }).into();
                font_label
            },
            {
                let size_label: Element<'static, Message> = container(row![
                    text(format!("{} pt", self.font_size)).size(12).color(Color::from_rgb(0.85, 0.85, 0.9)),
                    text(" \u{25BE}").size(10).color(Color::from_rgb(0.5, 0.5, 0.6)),
                ].align_y(iced::Alignment::Center))
                    .padding([4, 8])
                    .style(|_: &iced::Theme| container::Style {
                        background: Some(Color::from_rgba(0.17, 0.18, 0.30, 0.6).into()),
                        border: iced::Border::default().rounded(4).color(Color::from_rgba(0.25, 0.25, 0.35, 0.5)).width(1),
                        ..Default::default()
                    }).into();
                size_label
            },
            sep(),
            tool_btn("B".into(), Message::Bold, self.bold_active),
            tool_btn("I".into(), Message::Italic, self.italic_active),
            tool_btn("U".into(), Message::Underline, self.underline_active),
            tool_btn("S".into(), Message::Strikethrough, self.strikethrough_active),
            sep(),
            tool_btn("\u{2261}".into(), Message::AlignLeft, true),
            tool_btn("\u{2261}".into(), Message::AlignCenter, false),
            tool_btn("\u{2261}".into(), Message::AlignRight, false),
            tool_btn("\u{2261}".into(), Message::AlignJustify, false),
            sep(),
            tool_btn("\u{2022}".into(), Message::ToggleLineNumbers, false),
            tool_btn("\u{2263}".into(), Message::ToggleWordWrap, false),
            sep(),
            tool_btn("\u{1F50D}".into(), Message::FindToggle, false),
            tool_btn("\u{1F504}".into(), Message::ReplaceToggle, false),
        ].spacing(3).align_y(iced::Alignment::Center).padding([0, 8]);

        let ruler_bg = Color::from_rgb(0.122, 0.125, 0.208);
        let ruler_border = Color::from_rgb(0.176, 0.18, 0.29);
        let tick_color = Color::from_rgb(0.42, 0.42, 0.541);

        let mut ruler_row: Vec<Element<Message>> = Vec::new();
        for inch in 0..8 {
            let inch_x = inch as f32 * 80.0;
            let mut seg_parts: Vec<Element<Message>> = Vec::new();

            for sub in 0..8 {
                let _sub_x = inch_x + sub as f32 * 10.0;
                let is_half = sub == 4;
                let tick_h = if sub == 0 { 10.0 } else if is_half { 7.0 } else { 4.0 };
                let tick_w = 1.0;
                seg_parts.push(
                    container(horizontal_space().width(tick_w).height(tick_h))
                        .width(tick_w).height(tick_h)
                        .style(move |_: &iced::Theme| container::Style {
                            background: Some(tick_color.into()),
                            ..Default::default()
                        }).into()
                );
            }

            let inch_label = text(format!("{}", inch)).size(9).color(tick_color);
            seg_parts.push(horizontal_space().width(8.0).into());
            seg_parts.push(inch_label.into());

            ruler_row.push(row(seg_parts).align_y(iced::Alignment::End).into());
        }

        let ruler_inner = row(ruler_row)
            .width(Length::Fill)
            .height(22)
            .align_y(iced::Alignment::Center)
            .padding([0, 4]);

        let ruler = container(ruler_inner)
            .width(Length::Fill)
            .height(22)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(ruler_bg.into()),
                border: iced::Border::default().rounded(0).color(ruler_border).width(1),
                ..Default::default()
            });

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

        let section_label = |s: &str| -> Element<'static, Message> {
            text(s.to_string()).size(11).color(Color::from_rgb(0.5, 0.5, 0.6)).into()
        };

        let sidebar_content = column![
            row![text("FORMAT INSPECTOR").size(12).color(Color::from_rgb(0.8, 0.8, 0.9)), horizontal_space(), text("\u{2699}").size(14).color(Color::from_rgb(0.5, 0.5, 0.6))]
                .align_y(iced::Alignment::Center),
            text(""),
            section_label("PARAGRAPH STYLE"),
            row![
                tool_btn("Normal".into(), Message::AlignLeft, true),
                tool_btn("H1 Lead".into(), Message::Bold, false),
                tool_btn("H2 Sub".into(), Message::Bold, false),
            ].spacing(4),
            row![
                tool_btn("H3 Head".into(), Message::Bold, false),
                tool_btn("Quote".into(), Message::AlignLeft, false),
                tool_btn("Code".into(), Message::AlignLeft, false),
            ].spacing(4),
            text(""),
            section_label("TYPOGRAPHY"),
            row![text("Source Serif 4").size(12), horizontal_space(), text("Serif Editorial").size(10).color(Color::from_rgb(0.4, 0.4, 0.5))],
            row![
                button(text(format!("-")).size(12)).padding([2, 6]).on_press(Message::FontSizeChanged(self.font_size.saturating_sub(1))),
                button(text(format!("{} pt", self.font_size)).size(11)).padding([2, 8]),
                button(text("+").size(12)).padding([2, 6]).on_press(Message::FontSizeChanged(self.font_size + 1)),
                horizontal_space().width(8),
                text("\u{25CF}").size(10).color(Color::from_rgb(0.4, 0.4, 0.5)),
                text("\u{25CB}").size(10).color(Color::from_rgb(0.4, 0.4, 0.5)),
            ].spacing(4).align_y(iced::Alignment::Center),
            row![
                tool_btn("B".into(), Message::Bold, self.bold_active),
                tool_btn("I".into(), Message::Italic, self.italic_active),
                tool_btn("U".into(), Message::Underline, self.underline_active),
                tool_btn("S".into(), Message::Strikethrough, self.strikethrough_active),
                text("x\u{00B2}").size(12).color(Color::from_rgb(0.6, 0.6, 0.7)),
            ].spacing(8),
            row![kv("Tracking", "-0.015 EM")].spacing(0),
            row![kv("Scale", "100%")].spacing(0),
            text(""),
            section_label("PARAGRAPH & SPACING"),
            row![
                tool_btn("\u{2261}".into(), Message::AlignLeft, true),
                tool_btn("\u{2261}".into(), Message::AlignCenter, false),
                tool_btn("\u{2261}".into(), Message::AlignRight, false),
                tool_btn("\u{2261}".into(), Message::AlignJustify, false),
            ].spacing(4),
            row![kv("Line", "1.65")].spacing(0),
            row![kv("Before", "0 pt")].spacing(0),
            row![kv("After", "6 pt")].spacing(0),
            row![kv("First Line Indent", "0.00 in")].spacing(0),
            row![kv("Left Gutter Indent", "1.00 in")].spacing(0),
            row![kv("Right Gutter Indent", "1.00 in")].spacing(0),
            text(""),
            section_label("PAGE CANVAS SETUP"),
            row![
                text("\u{1F4C4}").size(16).color(Color::from_rgb(0.5, 0.5, 0.6)),
                horizontal_space().width(8),
                column![
                    text("US Letter \u{2022} Portrait").size(11),
                    text("8.5 \u{00D7} 11 inches (1.0\" margins)").size(10).color(Color::from_rgb(0.4, 0.4, 0.5)),
                ].spacing(2),
            ].spacing(4).align_y(iced::Alignment::Center),
        ].spacing(4).padding(12);

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

        let find_bar: Element<Message> = if self.find_state.show_find_bar {
            let close_btn = button(text("\u{2715}").size(12).color(Color::from_rgb(0.6, 0.6, 0.7)))
                .padding([4, 8]).on_press(Message::FindToggle);

            let find_row = row![
                text_input("Find...", &self.find_state.query)
                    .on_input(Message::FindQueryChanged)
                    .width(200)
                    .padding([4, 8])
                    .size(12),
                button(text("\u{25B6}").size(10)).padding([4, 6]).on_press(Message::FindNext),
                button(text("\u{25C0}").size(10)).padding([4, 6]).on_press(Message::FindPrevious),
                text(format!("{}/{}", self.find_state.current_match.map(|m| m + 1).unwrap_or(0), self.find_state.total_matches))
                    .size(11).color(Color::from_rgb(0.5, 0.5, 0.6)),
                close_btn,
            ].spacing(6).align_y(iced::Alignment::Center);

            if self.find_state.show_replace {
                let replace_row = row![
                    text_input("Replace...", &self.find_state.replace_text)
                        .on_input(Message::ReplaceTextChanged)
                        .width(200)
                        .padding([4, 8])
                        .size(12),
                    button(text("Replace").size(11)).padding([4, 8]).on_press(Message::ReplaceOne),
                    button(text("All").size(11)).padding([4, 8]).on_press(Message::ReplaceAll),
                ].spacing(6).align_y(iced::Alignment::Center);

                container(column![find_row, replace_row].spacing(4))
                    .width(Length::Fill)
                    .padding([8, 12])
                    .style(|_: &iced::Theme| container::Style {
                        background: Some(Color::from_rgb(0.12, 0.125, 0.208).into()),
                        border: iced::Border::default().rounded(0).color(Color::from_rgb(0.176, 0.18, 0.29)).width(1),
                        ..Default::default()
                    }).into()
            } else {
                container(find_row)
                    .width(Length::Fill)
                    .padding([8, 12])
                    .style(|_: &iced::Theme| container::Style {
                        background: Some(Color::from_rgb(0.12, 0.125, 0.208).into()),
                        border: iced::Border::default().rounded(0).color(Color::from_rgb(0.176, 0.18, 0.29)).width(1),
                        ..Default::default()
                    }).into()
            }
        } else {
            horizontal_space().height(0).into()
        };

        let layout = column![title_bar, menu_bar, toolbar, ruler, find_bar, main_area, status_bar];

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
