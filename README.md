# Rustle: A Terminal Text Editor in Rust

A minimal, modern, and feature-rich text editor that runs entirely in your terminal, built from scratch in Rust. It features simple syntax highlighting (currently only for Rust, non-exhaustive), robust search capabilities, and accurate handling of Unicode graphemes.

---

## ⚠️ Disclaimer: Not for Production Use

Rustle was built as a learning exercise for educational purposes. While it implements many core features of a text editor, it has not undergone extensive testing and is _not_ recommended for use as your primary editor in critical workflows.

You _may_ encounter bugs, performance issues, or crashes that could result in data loss. **Use at your own risk.**

---

## About the Project

Rustle draws inspiration from the classic _kilo_ editor tutorial, extending it with modern features and written in the safe, concurrent, and high-performance Rust programming language. Text management, rendering, and user input are implemented manually, using `crossterm` for low-level terminal control.

### Features

- **File I/O**
  - Open and save files directly from the command line.
- **Mode-Based Interface**
  - _Edit Mode_: Normal text editing.
  - _Search Mode_: Input search queries.
  - _Search Navigation Mode_: Navigate between search results.
- **Robust Search**
  - Find the next match from the current cursor position.
  - Navigate between matches using arrow keys.
  - Wraparound search that loops past file boundaries.
- **Layered Highlighting**
  - _Syntax Highlighting_: Full Rust syntax coverage (keywords, types, strings, numbers, comments) with extensibility for other languages.
  - _Search Highlighting_: All search matches highlighted. Active match uses a more prominent background.
- **Grapheme-Aware Rendering**
  - Accurate rendering of complex Unicode characters, including multi-character graphemes.
- **UI Components**
  - Status Bar: Shows filename, total lines, current cursor position, and modification status.
  - Message Bar: Contextual help and feedback display.

---

## Building and Running

Rustle uses Rust's package manager, Cargo.

### Prerequisites

- Rust (version 1.70 or newer) installed on your system.

### Installation

```bash
git clone https://github.com/angshumanHalder/Rustle.git
cd rustle
```

Build the project in release mode:

```bash
cargo build --release
```

The executable will be located at:

```
target/release/rustle
```

---

## Usage

Open a file by providing its path:

```bash
./target/release/rustle src/main.rs
```

Create a new empty buffer:

```bash
./target/release/rustle
```

---

## Keybindings

| Keybinding        | Action                                | Mode(s)                |
| :---------------- | :------------------------------------ | :--------------------- |
| Ctrl + Q          | Quit the editor                       | All                    |
| Ctrl + S          | Save the current file                 | All                    |
| Ctrl + F          | Enter Search Mode                     | Edit, SearchNavigate   |
| Enter             | Commit search and begin navigation    | Search                 |
| Esc               | Exit Search or Search Navigation mode | Search, SearchNavigate |
| ↑ / ↓ Arrow Keys  | Navigate between search results       | SearchNavigate         |
| Arrow Keys        | Move cursor                           | Edit                   |
| Backspace         | Delete character before cursor        | Edit, Search           |
| Delete            | Delete character at cursor            | Edit                   |
| Home / End        | Move to start/end of line             | Edit                   |
| PageUp / PageDown | Move view one screen up/down          | Edit                   |

---

## License

This project is licensed under the MIT License. See the `LICENSE.md` file for details.
