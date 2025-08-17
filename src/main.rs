#![warn(clippy::all, clippy::pedantic)]

mod editor;

use editor::Editor;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    Editor::new(args.get(1)).unwrap().run();
}
