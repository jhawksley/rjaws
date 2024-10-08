use terminal_size::{terminal_size, Height, Width};


pub fn tui_get_terminal_size() -> (usize, usize) {
    // We need to do unwrap_or here and supply a default, because if Jaws is run inside a unix
    // pipeline, there is no tty.
    let (Width(width), Height(height)) =
        terminal_size().unwrap_or((Width(120), Height(30)));

    (width as usize, height as usize)
}

pub fn tui_center_text(text: &String) -> String {
    let (width, _) = tui_get_terminal_size();
    format!("{: ^width$}", text, width = width)
}


pub fn tui_lcr_text(left: Option<String>, center: Option<String>, right: Option<String>) -> String {
    let (width, _) = tui_get_terminal_size();

    let effective_left = left.unwrap_or("".to_string());
    let effective_center = center.unwrap_or("".to_string());
    let effective_right = right.unwrap_or("".to_string());

    // Column width is naively 1/3 of the available columns.
    let col_width = width / 3;

    // However, this is integer math so there may be left over columns.
    // These are added to the center column.
    let center_col_width: usize = col_width + (width % 3);

    format!("{:<lwidth$}{:^cwidth$}{:>rwidth$}", effective_left, effective_center, effective_right,
            cwidth = center_col_width,
            lwidth = col_width,
            rwidth = col_width)
}

pub fn tui_separator_bar(in_char: &str) -> String {
    in_char.repeat(tui_get_terminal_size().0)
}
