# Rust GUI Text Editor

A simple GUI text editor built with Rust and the iced framework.

## Features

- **Text Editing** — Type, delete, copy, cut, paste
- **Cursor Movement** — Arrow keys, Home/End, Page Up/Down
- **File Operations** — Open, Save, Save As, New
- **Find & Replace** — Search text, replace one or all occurrences
- **Line Numbers** — Toggle line number gutter
- **Word Wrap** — Toggle word wrap
- **Dark/Light Theme** — Switch between themes
- **Undo/Redo** — Basic undo/redo support

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

## Running

```bash
cargo run
```

The binary will be at `target/release/rust-editor.exe`.

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| New File | Button click |
| Open File | Button click |
| Save | Button click |
| Undo | Button click |
| Redo | Button click |
| Copy | Button click |
| Cut | Button click |
| Paste | Button click |
| Find | Button click |
| Replace | Button click |

## Dependencies

- `iced` — GUI framework
- `ropey` — Efficient text buffer (O(log n) operations)
- `rfd` — Native file dialogs
- `copypasta` — Clipboard support

## Project Structure

```
src/
├── main.rs          # Entry point
├── editor.rs        # Main editor logic and UI
├── buffer.rs        # Text buffer operations
├── cursor.rs        # Cursor and selection handling
├── file_io.rs       # File open/save
├── syntax.rs        # Syntax highlighting
├── find.rs          # Find and replace
├── theme.rs         # Theme definitions
└── ui.rs            # UI module
```

## Notes

- Uses `ropey::Rope` for efficient text storage
- Only renders visible lines for performance
- Native file dialogs via `rfd`
