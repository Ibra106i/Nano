use iced::{Element, Length, Color};
use iced::widget::{button, column, container, horizontal_space, row, text, text_input};

use crate::editor::Message;
use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::find::FindState;
use crate::syntax::SyntaxHighlighter;
use crate::theme::Theme as EditorTheme;
use crate::layout;

pub struct ViewContext<'a> {
    pub file_info_name: &'a str,
    pub file_is_modified: bool,
    pub theme: &'a EditorTheme,
    pub font_family: &'a str,
    pub font_size: u32,
    pub bold_active: bool,
    pub italic_active: bool,
    pub underline_active: bool,
    pub strikethrough_active: bool,
    pub buffer: &'a Buffer,
    pub cursor: &'a Cursor,
    pub line_numbers: bool,
    pub show_sidebar: bool,
    pub word_count: usize,
    pub char_count: usize,
    pub zoom: f32,
    pub clipboard_status: Option<&'a str>,
    pub find_state: &'a FindState,
    pub syntax: &'a SyntaxHighlighter,
    pub line_height: f32,
}

fn sep() -> Element<'static, Message> {
    text(" | ").size(13).color(Color::from_rgb(0.3, 0.3, 0.4)).into()
}

fn section_label(s: &str) -> Element<'static, Message> {
    text(s.to_string()).size(11).color(Color::from_rgb(0.5, 0.5, 0.6)).into()
}

pub fn tool_btn<'a>(label: String, msg: Message, active: bool) -> Element<'a, Message> {
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
}

fn kv<'a>(key: &'a str, val: &'a str) -> Element<'a, Message> {
    row![text(key).size(11), horizontal_space(), text(val).size(11)].into()
}

pub fn title_bar(ctx: &ViewContext) -> Element<Message> {
    let dirty = if ctx.file_is_modified { " ●" } else { "" };
    let title = format!("Nano — {}{}", ctx.file_info_name, dirty);
    row![horizontal_space(), text(title).size(13).color(ctx.theme.text_secondary()), horizontal_space()]
        .height(38).align_y(iced::Alignment::Center)
        .into()
}

pub fn menu_bar(ctx: &ViewContext) -> Element<Message> {
    let menus = ["File", "Edit", "View", "Insert", "Format", "Tools", "Help"];
    let menu_items: Vec<Element<Message>> = menus.iter().map(|m| {
        button(text(*m).size(12).color(ctx.theme.text_primary())).padding([5, 10]).into()
    }).collect();
    row(menu_items).spacing(2).padding([0, 8]).into()
}

pub fn toolbar(ctx: &ViewContext) -> Element<Message> {
    row![
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
                text(ctx.font_family.to_string()).size(12).color(Color::from_rgb(0.85, 0.85, 0.9)),
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
                text(format!("{} pt", ctx.font_size)).size(12).color(Color::from_rgb(0.85, 0.85, 0.9)),
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
        tool_btn("B".into(), Message::Bold, ctx.bold_active),
        tool_btn("I".into(), Message::Italic, ctx.italic_active),
        tool_btn("U".into(), Message::Underline, ctx.underline_active),
        tool_btn("S".into(), Message::Strikethrough, ctx.strikethrough_active),
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
    ].spacing(3).align_y(iced::Alignment::Center).padding([0, 8]).into()
}

pub fn ruler(_ctx: &ViewContext) -> Element<Message> {
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

    container(ruler_inner)
        .width(Length::Fill)
        .height(22)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(ruler_bg.into()),
            border: iced::Border::default().rounded(0).color(ruler_border).width(1),
            ..Default::default()
        }).into()
}

pub fn line_numbers_gutter(ctx: &ViewContext) -> Element<Message> {
    let line_count = ctx.buffer.len_lines();
    let mut ln_col = column![].spacing(4).padding([0, 8]);
    for i in 1..=line_count.min(50) {
        let is_cur = i - 1 == ctx.cursor.line;
        ln_col = ln_col.push(text(format!("{:>3}", i)).size(12).color(
            if is_cur { Color::from_rgb(0.6, 0.8, 1.0) } else { ctx.theme.line_number_fg().unwrap_or(Color::from_rgb(0.35, 0.35, 0.45)) }
        ));
    }
    container(ln_col).width(layout::LINE_NUMBER_WIDTH).height(Length::Fill).into()
}

pub fn page_content(ctx: &ViewContext) -> Element<Message> {
    let line_count = ctx.buffer.len_lines();
    let sel_color = ctx.theme.page_selection();
    let has_sel = ctx.cursor.has_selection();
    let sel_range = ctx.cursor.selection_range();

    let mut content = column![].spacing(6).padding(24);
    for li in 0..line_count.min(50) {
        let lt: String = ctx.buffer.line(li).chars().collect();
        let lt_len = lt.len();
        let sz = if li == 0 { 30.0 } else if lt.starts_with(|c: char| c.is_numeric()) { 18.0 } else { 15.0 };

        let is_cursor_line = li == ctx.cursor.line;
        let highlighted = ctx.syntax.highlight_line(&lt);

        if has_sel {
            let (s_line, s_col) = sel_range.unwrap().0;
            let (e_line, e_col) = sel_range.unwrap().1;

            let sel_start = if li == s_line { s_col } else { 0 };
            let sel_end = if li == e_line { e_col } else { lt_len };
            let has_sel_on_line = li >= s_line && li <= e_line && sel_start < sel_end;

            if has_sel_on_line {
                let parts = render_line_with_selection(&highlighted, sz, sel_color, sel_start, sel_end, is_cursor_line, ctx.cursor.col, ctx.theme.cursor_color());
                content = content.push(row(parts).align_y(iced::Alignment::Center));
            } else if is_cursor_line {
                let parts = render_line_with_cursor(&highlighted, sz, ctx.cursor.col, ctx.theme.cursor_color());
                content = content.push(row(parts).align_y(iced::Alignment::Center));
            } else {
                let parts: Vec<Element<Message>> = highlighted.into_iter()
                    .map(|(t, c)| text(t).size(sz).color(c).into())
                    .collect();
                content = content.push(row(parts).align_y(iced::Alignment::Center));
            }
        } else if is_cursor_line {
            let parts = render_line_with_cursor(&highlighted, sz, ctx.cursor.col, ctx.theme.cursor_color());
            content = content.push(row(parts).align_y(iced::Alignment::Center));
        } else {
            let parts: Vec<Element<Message>> = highlighted.into_iter()
                .map(|(t, c)| text(t).size(sz).color(c).into())
                .collect();
            content = content.push(row(parts).align_y(iced::Alignment::Center));
        }
    }
    content.into()
}

fn render_line_with_selection(
    highlighted: &[(String, Color)],
    sz: f32,
    sel_color: Color,
    sel_start: usize,
    sel_end: usize,
    has_cursor: bool,
    cursor_col: usize,
    cursor_color: Color,
) -> Vec<Element<'static, Message>> {
    let mut parts: Vec<Element<Message>> = Vec::new();
    let mut char_pos = 0;

    for (seg_text, seg_color) in highlighted {
        let seg_len = seg_text.chars().count();
        let seg_end = char_pos + seg_len;

        if seg_end <= sel_start || char_pos >= sel_end {
            let outside = char_pos < sel_start;
            let color = if outside { *seg_color } else { *seg_color };
            if has_cursor && char_pos <= cursor_col && cursor_col <= seg_end && !outside {
                let before_cur: String = seg_text.chars().take(cursor_col - char_pos).collect();
                let after_cur: String = seg_text.chars().skip(cursor_col - char_pos).collect();
                if !before_cur.is_empty() {
                    parts.push(text(before_cur).size(sz).color(Color::WHITE).into());
                }
                parts.push(cursor_block(sz, cursor_color));
                if !after_cur.is_empty() {
                    parts.push(text(after_cur).size(sz).color(Color::WHITE).into());
                }
            } else {
                parts.push(text(seg_text.clone()).size(sz).color(color).into());
            }
        } else {
            let before_sel: String = seg_text.chars().take(sel_start.saturating_sub(char_pos)).collect();
            let in_sel: String = seg_text.chars().skip(sel_start.saturating_sub(char_pos)).take(seg_len - sel_start.saturating_sub(char_pos)).take(sel_end.saturating_sub(char_pos + (sel_start.saturating_sub(char_pos)))).collect();
            let after_sel: String = seg_text.chars().skip(seg_len - (seg_end.saturating_sub(sel_end))).collect();

            if !before_sel.is_empty() {
                parts.push(text(before_sel).size(sz).color(*seg_color).into());
            }
            if !in_sel.is_empty() {
                if has_cursor && char_pos <= cursor_col && cursor_col <= seg_end {
                    let sel_before_cur: String = in_sel.chars().take(cursor_col - char_pos - (sel_start.saturating_sub(char_pos))).collect();
                    let sel_after_cur: String = in_sel.chars().skip(cursor_col - char_pos - (sel_start.saturating_sub(char_pos))).collect();
                    if !sel_before_cur.is_empty() {
                        parts.push(selected_text(sz, sel_before_cur, sel_color));
                    }
                    parts.push(cursor_block(sz, cursor_color));
                    if !sel_after_cur.is_empty() {
                        parts.push(selected_text(sz, sel_after_cur, sel_color));
                    }
                } else {
                    parts.push(selected_text(sz, in_sel, sel_color));
                }
            }
            if !after_sel.is_empty() {
                parts.push(text(after_sel).size(sz).color(*seg_color).into());
            }
        }

        char_pos = seg_end;
    }
    parts
}

fn render_line_with_cursor(
    highlighted: &[(String, Color)],
    sz: f32,
    cursor_col: usize,
    cursor_color: Color,
) -> Vec<Element<'static, Message>> {
    let mut parts: Vec<Element<Message>> = Vec::new();
    let mut char_pos = 0;

    for (seg_text, seg_color) in highlighted {
        let seg_len = seg_text.chars().count();
        let seg_end = char_pos + seg_len;

        if cursor_col < char_pos || cursor_col > seg_end {
            parts.push(text(seg_text.clone()).size(sz).color(*seg_color).into());
        } else {
            let before: String = seg_text.chars().take(cursor_col - char_pos).collect();
            let after: String = seg_text.chars().skip(cursor_col - char_pos).collect();
            if !before.is_empty() {
                parts.push(text(before).size(sz).color(*seg_color).into());
            }
            parts.push(cursor_block(sz, cursor_color));
            if !after.is_empty() {
                parts.push(text(after).size(sz).color(*seg_color).into());
            }
        }

        char_pos = seg_end;
    }
    parts
}

fn selected_text<'a>(sz: f32, t: String, sel_color: Color) -> Element<'a, Message> {
    container(text(t).size(sz).color(Color::WHITE))
        .style(move |_: &iced::Theme| container::Style {
            background: Some(sel_color.into()),
            border: iced::Border::default().rounded(2),
            ..Default::default()
        }).into()
}

fn cursor_block<'a>(sz: f32, cursor_color: Color) -> Element<'a, Message> {
    container(text(" ").size(sz)).width(2).height(iced::Length::Fixed(sz))
        .style(move |_: &iced::Theme| container::Style {
            background: Some(cursor_color.into()),
            ..Default::default()
        }).into()
}

pub fn sidebar_container(ctx: &ViewContext) -> Element<Message> {
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
            button(text(format!("-")).size(12)).padding([2, 6]).on_press(Message::FontSizeChanged(ctx.font_size.saturating_sub(1))),
            button(text(format!("{} pt", ctx.font_size)).size(11)).padding([2, 8]),
            button(text("+").size(12)).padding([2, 6]).on_press(Message::FontSizeChanged(ctx.font_size + 1)),
            horizontal_space().width(8),
            text("\u{25CF}").size(10).color(Color::from_rgb(0.4, 0.4, 0.5)),
            text("\u{25CB}").size(10).color(Color::from_rgb(0.4, 0.4, 0.5)),
        ].spacing(4).align_y(iced::Alignment::Center),
        row![
            tool_btn("B".into(), Message::Bold, ctx.bold_active),
            tool_btn("I".into(), Message::Italic, ctx.italic_active),
            tool_btn("U".into(), Message::Underline, ctx.underline_active),
            tool_btn("S".into(), Message::Strikethrough, ctx.strikethrough_active),
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

    container(sidebar_content).width(280).height(Length::Fill)
        .style(container::bordered_box)
        .into()
}

pub fn status_bar(ctx: &ViewContext) -> Element<Message> {
    let status_left = row![
        text(format!("Page 1 of 1 | Word count: {} | Characters: {} | UTF-8", ctx.word_count, ctx.char_count))
            .size(11).color(Color::from_rgb(0.6, 0.6, 0.7)),
    ].push_maybe(
        ctx.clipboard_status.map(|msg| {
            text(msg).size(11).color(Color::from_rgb(1.0, 0.4, 0.4))
        })
    );
    let zoom_pct = format!("{}%", (ctx.zoom * 100.0) as u32);
    let status_right = row![
        button(text("\u{2212}").size(12)).padding(4).on_press(Message::ZoomOut),
        button(text(zoom_pct).size(11)).padding(4).on_press(Message::ZoomReset),
        button(text("+").size(12)).padding(4).on_press(Message::ZoomIn),
    ].spacing(4).align_y(iced::Alignment::Center);

    row![status_left, horizontal_space(), status_right]
        .padding([0, 12]).height(layout::STATUS_BAR_H).align_y(iced::Alignment::Center)
        .into()
}

pub fn find_bar(ctx: &ViewContext) -> Element<Message> {
    if !ctx.find_state.show_find_bar {
        return horizontal_space().height(0).into();
    }

    let close_btn = button(text("\u{2715}").size(12).color(Color::from_rgb(0.6, 0.6, 0.7)))
        .padding([4, 8]).on_press(Message::FindToggle);

    let find_row = row![
        text_input("Find...", &ctx.find_state.query)
            .on_input(Message::FindQueryChanged)
            .width(200)
            .padding([4, 8])
            .size(12),
        button(text("\u{25B6}").size(10)).padding([4, 6]).on_press(Message::FindNext),
        button(text("\u{25C0}").size(10)).padding([4, 6]).on_press(Message::FindPrevious),
        text(format!("{}/{}", ctx.find_state.current_match.map(|m| m + 1).unwrap_or(0), ctx.find_state.total_matches))
            .size(11).color(Color::from_rgb(0.5, 0.5, 0.6)),
        close_btn,
    ].spacing(6).align_y(iced::Alignment::Center);

    let bar_style = |_: &iced::Theme| container::Style {
        background: Some(Color::from_rgb(0.12, 0.125, 0.208).into()),
        border: iced::Border::default().rounded(0).color(Color::from_rgb(0.176, 0.18, 0.29)).width(1),
        ..Default::default()
    };

    if ctx.find_state.show_replace {
        let replace_row = row![
            text_input("Replace...", &ctx.find_state.replace_text)
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
            .style(bar_style)
            .into()
    } else {
        container(find_row)
            .width(Length::Fill)
            .padding([8, 12])
            .style(bar_style)
            .into()
    }
}
