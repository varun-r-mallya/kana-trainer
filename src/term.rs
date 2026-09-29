//! Terminal helpers: colours, display width, padding, prompting.

use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};

pub static COLOR: AtomicBool = AtomicBool::new(true);

pub fn paint(code: &str, s: &str) -> String {
    if COLOR.load(Ordering::Relaxed) {
        format!("\x1b[{}m{}\x1b[0m", code, s)
    } else {
        s.to_string()
    }
}
pub fn bold(s: &str) -> String {
    paint("1", s)
}
pub fn dim(s: &str) -> String {
    paint("2", s)
}
pub fn red(s: &str) -> String {
    paint("1;31", s)
}
pub fn green(s: &str) -> String {
    paint("1;32", s)
}
pub fn yellow(s: &str) -> String {
    paint("1;33", s)
}
pub fn cyan(s: &str) -> String {
    paint("1;36", s)
}

/// Terminal display width (kana are double-width).
pub fn dw(s: &str) -> usize {
    s.chars()
        .map(|c| if (c as u32) >= 0x2E80 { 2 } else { 1 })
        .sum()
}

pub fn center(plain: &str, colored: &str, w: usize) -> String {
    let total = w.saturating_sub(dw(plain));
    let l = total / 2;
    format!("{}{}{}", " ".repeat(l), colored, " ".repeat(total - l))
}

pub fn pad_right(plain: &str, colored: &str, w: usize) -> String {
    format!("{}{}", colored, " ".repeat(w.saturating_sub(dw(plain))))
}

pub fn prompt(msg: &str) -> Option<String> {
    print!("{}", msg);
    io::stdout().flush().ok();
    let mut line = String::new();
    match io::stdin().lock().read_line(&mut line) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(line.trim().to_lowercase()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_counts_kana_as_double() {
        assert_eq!(dw("abc"), 3);
        assert_eq!(dw("あ"), 2);
        assert_eq!(dw("きゃ"), 4);
        assert_eq!(dw("ア b"), 4);
        assert_eq!(dw("-"), 1);
    }

    #[test]
    fn center_pads_to_display_width() {
        let s = center("あ", "あ", 8);
        assert_eq!(dw(&s), 8);
        assert_eq!(s, "   あ   ");
        assert_eq!(center("abcdefghij", "abcdefghij", 4), "abcdefghij");
    }

    #[test]
    fn pad_right_ignores_colour_codes() {
        let colored = "\x1b[1mhi\x1b[0m";
        assert_eq!(pad_right("hi", colored, 5), format!("{colored}   "));
    }
}
