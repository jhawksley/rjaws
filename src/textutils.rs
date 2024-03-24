use std::io::{stdout, Write};

use terminal_size::{Height as TerminalHeight, terminal_size, Width as TerminalWidth};
use termion::clear::CurrentLine;
use termion::color::{Blue, Fg};
use termion::style;

const LINECOLOR: Fg<Blue> = Fg(Blue);

pub fn txt_line_output(message: String) {
    txt_line_clear();
    print!("\r{}{}{}", LINECOLOR, message, style::Reset);
    _ = stdout().flush();
}

pub fn txt_line_clear() {
    print!("\r{}", CurrentLine);
    _ = stdout().flush();
}

pub fn get_terminal_size() -> (usize, usize) {
    let (TerminalWidth(width), TerminalHeight(height)) =
        terminal_size().expect("failed to obtain a terminal size");

    (width as usize, height as usize)
}


pub fn report_title(title: String) {
    println!("{}", center_text(format!("J A W S - {}", crate::VERSION)));
    println!("{}\n", center_text(title));
}

pub fn center_text(text: String) -> String {
    let (width, _) = get_terminal_size();

    format!("{: ^width$}", text, width = width)
}
