use std::io::{stdout, Write};

use terminal_size::{Height as TerminalHeight, Height, terminal_size, Width as TerminalWidth, Width};
use termion::clear::CurrentLine;
use termion::color::{Blue, Fg};
use termion::style;

use crate::Options;

const LINECOLOR: Fg<Blue> = Fg(Blue);

pub struct Textutil {
    /// If true, causes output to be suppressed.
    mute: bool,
}

impl Textutil {
    pub(crate) fn new(options: &Options) -> Textutil {
        Self {
            mute: !options.output_format.unwrap().supports_free_text_output()
        }
    }


    pub fn txt_line_output(&self, message: String) {
        self.txt_line_clear();
        if !self.mute {
            print!("\r{}{}{}", LINECOLOR, message, style::Reset);
            _ = stdout().flush();
        }
    }


    pub fn txt_line_clear(&self) {
        if !self.mute {
            print!("\r{}", CurrentLine);
            _ = stdout().flush();
        }
    }

    pub fn report_title(&self, title: String) {
        if !self.mute {
            println!("{}", self.center_text(format!("J A W S - {}", crate::VERSION)));
            println!("{}\n", self.center_text(title));
        }
    }

    pub fn center_text(&self, text: String) -> String {
        let (width, _) = get_terminal_size();
        format!("{: ^width$}", text, width = width)
    }

    // ---------------------------------------------------------------------------------------------
    // This is the moved version of the Command mod output driver

    pub fn notify_comms(&self, action: Option<String>) {
        match action {
            Some(action) => self.txt_line_output(format!("Talking to AWS ({})...", action)),
            None => self.txt_line_output("Talking to AWS...".to_string())
        }
    }


    pub fn notify(&self, string: String) {
        self.txt_line_output(string);
    }

    pub fn notify_working(&self) {
        self.txt_line_output("Marshalling data...".to_string());
    }

    pub fn notify_clear(&self) {
        self.txt_line_clear();
    }

    pub fn to_hms(&self, duration: u64) -> String {
        let seconds = duration % 60;
        let minutes = (duration / 60) % 60;
        let hours = (duration / 60) / 60;
        format!("{}h{}m{}s", hours, minutes, seconds)
    }
}

pub fn get_terminal_size() -> (usize, usize) {
    // We need to do unwrap_or here and supply a default, because if Jaws is run inside a unix
    // pipeline, there is no tty.
    let (TerminalWidth(width), TerminalHeight(height)) =
        terminal_size().unwrap_or((Width(120), Height(30)));

    (width as usize, height as usize)
}
