use colored::{Colorize, ColoredString};

// Some formatting stuff
pub fn indent(text: &str) -> String {
    text.lines()
        .map(|l| format!("    {}", l))
        .collect::<Vec<String>>()
        .join("\n")
}
pub fn green(text: &str) -> ColoredString {
    text.truecolor(0, 175, 135)
}
pub fn red(text: &str) -> ColoredString {
    text.truecolor(255, 47, 109)
}
pub fn muted(text: &str) -> ColoredString {
    text.truecolor(68, 68, 68)
}
pub fn on_red(text: &str) -> ColoredString {
    text.on_truecolor(255, 47, 109).truecolor(28, 28, 28)
}
pub fn on_green(text: &str) -> ColoredString {
    text.on_truecolor(0, 175, 135).truecolor(28, 28, 28)
}

