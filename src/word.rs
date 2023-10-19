use colored::{Color, Colorize};

pub const WORD_LENGTH: usize = 5;
pub type Letter = u8;
pub type Word = [Letter; WORD_LENGTH];

pub fn make_word(input: &str) -> Option<Word> {
    let upper = input.to_ascii_uppercase();
    let bytes = upper.as_bytes();
    let valid = bytes.len() == 5 && bytes.iter().all(Letter::is_ascii_uppercase);
    if valid {
        bytes.try_into().ok()
    } else {
        None
    }
}

pub fn letter_to_string(letter: Letter) -> String {
    (letter as char).to_string()
}

pub fn letters_to_string<'a, I>(letters: I) -> String
where
    I: IntoIterator<Item = &'a u8>,
{
    letters.into_iter().map(|l| *l as char).collect()
}

pub fn letters_with_fg<'a, I>(letters: I, color: Color) -> String
where
    I: IntoIterator<Item = &'a u8>,
{
    letters_to_string(letters).color(color).to_string()
}

pub fn letters_with_bg<'a, I>(letters: I, color: Color) -> String
where
    I: IntoIterator<Item = &'a u8>,
{
    letters_to_string(letters)
        .on_color(color)
        .color(Color::Black)
        .to_string()
}

pub fn letter_with_fg(letter: Letter, color: Color) -> String {
    letters_with_fg(std::slice::from_ref(&letter), color)
}

pub fn letter_with_bg(letter: Letter, color: Color) -> String {
    letters_with_bg(std::slice::from_ref(&letter), color)
}

pub fn str_with_fg(text: &str, color: Color) -> String {
    String::from(text).color(color).to_string()
}
