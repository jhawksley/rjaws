use std::io::{stdout, Write};

use termion::{style};
use termion::clear::CurrentLine;
use termion::color::{Blue, Fg};

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