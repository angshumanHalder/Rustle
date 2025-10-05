Rustle: A Terminal Text Editor in Rust
A minimal, modern, and feature-rich text editor that runs in your terminal, built from scratch in Rust. It features syntax highlighting (only for rust language - non exhaustive), robust search, and correct handling of Unicode graphemes.

## ⚠️ Disclaimer: Not for Production Use

This project was built as a learning exercise and is intended for educational purposes. While it implements many core features of a text editor, it has not been extensively tested and is not recommended for use as your primary editor for critical work.

You may encounter bugs, performance issues, or unexpected crashes that could potentially lead to data loss. Please use it at your own risk.

## About The Project

Rustle is an attempt to build a text editor from the ground up, inspired by the classic "kilo" editor tutorial but extended with modern features and written in the safe, concurrent, and performant Rust programming language. This project manages text, rendering, and user input manually, using crossterm for low-level terminal manipulation.

## Features

File I/O: Open and save files from the command line.

Mode-Based Interface:

Edit Mode: For normal text editing.

Search Mode: For typing search queries.

Search Navigation Mode: For navigating between search results.

Robust Search:

Find the next match from the current cursor position.

Navigate between matches using arrow keys.

Wraps around the end of the file.

Layered Highlighting:

Syntax Highlighting: Full syntax highlighting for Rust files, including keywords, types, strings, numbers, and comments. The system is extensible for other languages.

Search Highlighting: All search matches are highlighted with a background color, with the active match getting a more prominent highlight.

Grapheme-Aware Rendering: Correctly handles and renders complex Unicode characters, including multi-character graphemes.

UI Components:

Status Bar: Displays file name, line count, current position, and modification status.

Message Bar: Provides contextual help and feedback.

## Building and Running

This project is built and managed with Rust's package manager, Cargo.

### Prerequisites

Rust programming language (version 1.70 or newer)

### Installation

Clone the repository:

Bash

git clone https://github.com/angshumanHalder/Rustle.git

cd rustle

Build the project in release mode:

Bash

cargo build --release
The executable will be located at target/release/rustle.

### Usage

To open a file, provide its path as a command-line argument:

Bash

./target/release/rustle src/main.rs
To create a new, empty buffer, run without arguments:

Bash

./target/release/rustle

## Keybindings

Keybinding Action Mode(s)
Ctrl + Q Quit the editor. All
Ctrl + S Save the current file. All
Ctrl + F Enter Search mode to type a query. Edit, SearchNavigate
Enter In Search mode, commits the search and starts navigation. Search
Esc Exit Search or Search Navigation mode. Search, SearchNavigate
↑ / ↓ Arrow Keys Navigate between search results. SearchNavigate
Arrow Keys Move the cursor. Edit
Backspace Delete character before the cursor. Edit, Search
Delete Delete character at the cursor. Edit
Home / End Move to the start/end of the line. Edit
PageUp / PageDown Move the view up or down by a full screen. Edit

Export to Sheets

## License

This project is licensed under the MIT License - see the LICENSE.md file for details.
