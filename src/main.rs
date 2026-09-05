mod editor;
mod buffer;
mod cursor;
mod ui;
mod file_io;
mod syntax;
mod find;
mod theme;
mod measure;

use editor::Editor;

fn main() -> iced::Result {
    iced::application("Rust Editor", Editor::update, Editor::view)
        .subscription(Editor::subscription)
        .run()
}
