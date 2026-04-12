pub mod error;
pub mod highlight;
pub mod markup;
pub mod menu;
pub mod prompt;
pub mod table;

use self::markup as text;
use std::io::Write;

pub fn print_stdout(content: &str) {
    if content.is_empty() {
        return;
    }
    print!("{}", text::render_ansi(content));
    let _ = std::io::stdout().flush();
}

pub fn print_stderr(content: &str) {
    if content.is_empty() {
        return;
    }
    eprint!("{}", text::render_ansi(content));
    let _ = std::io::stderr().flush();
}

pub fn println_stdout(content: &str) {
    let mut rendered = text::render_ansi(content);
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    print!("{}", rendered);
    let _ = std::io::stdout().flush();
}
