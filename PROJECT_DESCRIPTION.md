# Nano — Rust GUI Text Editor

## What We Built

Nano is a Word/LibreOffice-style text editor written in Rust using the `iced` GUI framework. It renders a centered white page on a dark canvas with a full formatting toolbar, ruler, sidebar inspector, and status bar — matching a Material Design 3-inspired design spec. The editor supports real text editing with keyboard input, selection, clipboard operations, find/replace, and basic formatting toggles.

## Architecture

The project lives at `D:\Ibrahim106\Desktop\Nano` and is structured as a single-binary Rust application. The entry point is `src/main.rs` which launches an iced application using `Editor::update` and `Editor::view`. All editor state and logic lives in `src/editor.rs` — a 830-line file containing the `Editor` struct, `Message` enum, `update()`, `view()`, and `subscription()`.

### Core Files

- **`src/editor.rs`** — Main module. Contains the `Editor` struct (all state), `Message` enum (all UI events), `update()` (event handling), `view()` (UI rendering), and `subscription()` (keyboard/mouse event capture). This is where 90% of the code lives.
- **`src/cursor.rs`** — `Cursor` struct with line/col position, selection anchor (`selection_start: Option<(usize, usize)>`), and movement methods (`move_up`, `move_down`, `move_left`, `move_right`, `move_home`, `move_end`, `page_up`, `page_down`). Has `selection_range()` that returns normalized `((start_line, start_col), (end_line, end_col))` and `to_byte_offset()` for rope indexing.
- **`src/buffer.rs`** — `Buffer` struct wrapping a `ropey::Rope`. Provides `insert_char`, `insert_str`, `delete` (returns deleted text), `undo`/`redo` via `EditOperation` stack. The rope stores the full document text as a gap buffer for efficient insertion/deletion.
- **`src/theme.rs`** — `Theme` enum (Dark/Light) with color accessor methods matching the design spec: `surface()`, `primary()`, `text_primary()`, `page_selection()`, `cursor_color()`, etc. Colors derived from Material Design 3 palette (`#111224`, `#a4c9ff`, `#4a9eff`, `#e1e0fb`).
- **`src/measure.rs`** — `TextMeasurer` struct using `cosmic-text` for proportional font glyph measurement. `col_from_x()` converts a pixel X coordinate to a character column by laying out glyphs and summing widths. Used for click-to-position accuracy.
- **`src/find.rs`** — `FindState` struct with `query`, `replace_text`, `show_find_bar`, `show_replace`, match tracking. Methods: `find_all()` returns all `(line, col)` matches, `find_next()`/`find_previous()` cycle through them, `replace_one()`/`replace_all()` modify the rope.
- **`src/file_io.rs`** — `FileInfo` (path, modified state, filename) and `FileOperation` enum. `open_file_dialog()`/`save_file_dialog()` use `rfd` (native file dialogs). `read_file()`/`write_file()` handle async I/O.
- **`src/syntax.rs`** — `SyntaxHighlighter` with `Language` enum. Currently stubbed out — has `highlight_line()` infrastructure but not wired into the view.
- **`Cargo.toml`** — Dependencies: `iced` (tokio), `ropey`, `rfd`, `copypasta`, `cosmic-text 0.12`, `fontdb 0.16`.

## Features Implemented

1. **Text Editing** — Full keyboard input (characters, space, enter, tab, backspace, delete). Cursor moves with arrow keys, home/end, page up/down.
2. **Text Selection** — Shift+arrow keys extend selection. Mouse click-and-drag selects text. Blue highlight (`rgba(74, 158, 255, 0.28)`) renders behind selected text. Click clears selection.
3. **Clipboard** — Ctrl+C copies, Ctrl+X cuts, Ctrl+V pastes using system clipboard via `copypasta`. Ctrl+A selects all. All editing operations (type, delete, enter) replace selected text first.
4. **Find/Replace** — Ctrl+F opens floating search bar with next/prev/close and match count. Ctrl+H toggles replace row with Replace One/Replace All buttons.
5. **Toolbar** — Unicode icon buttons for file ops (📄📂💾), edit (✂📋📏), undo/redo (↩↪), formatting (B/I/U/S), alignment (≡), find (🔍🔄). B/I/U/S toggle active state with blue highlight.
6. **Font Controls** — "Source Serif 4 ▾" and "12 pt ▾" displays in toolbar. Font size +/- stepper in sidebar.
7. **Ruler** — Dark ruler with tick marks at 10px intervals, inch numbers 0-7.
8. **Format Inspector Sidebar** — 280px right panel with Paragraph Style (Normal/H1/H2/H3/Quote/Code), Typography (font, size, B/I/U/S/x², tracking, scale), Paragraph & Spacing (alignment, line/Before/After/indents), Page Canvas Setup.
9. **Status Bar** — Word count, character count, UTF-8, zoom controls (-/100%/+).
10. **Line Numbers** — Left gutter with line numbers, current line highlighted in blue.
11. **Page View** — Centered white page (614px wide) on dark canvas with cursor indicator (2px blue vertical bar).

## Design Spec Reference

The design files are at `D:\Ibrahim106\Downloads\stitch_nano_text_editor_ui\`:
- `DESIGN.md` — Full color palette, typography tokens, spacing, component specs
- `screen.png` — Visual reference screenshot showing the target UI

Key design colors: surface `#111224`, primary `#a4c9ff`, primary-container `#4a9eff`, on-surface `#e1e0fb`, page `#ffffff`, selection `rgba(74, 158, 255, 0.28)`.

## Running

Double-click `run.bat` in the project root. It runs `cargo build --release` and launches `target\release\rust-editor.exe`.

## Known Limitations

- Font family/size dropdowns are display-only (no actual dropdown menu yet)
- Bold/Italic/Underline/Strikethrough toggle visual state but don't affect text rendering
- No actual font rendering changes (all text renders in Source Serif 4 at fixed sizes)
- Syntax highlighting not wired in
- No undo/redo visual feedback
- Ruler margin handles not draggable yet
- Blockquote and table rendering not implemented
