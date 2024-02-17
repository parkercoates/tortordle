use colored::{Color, Colorize};
use std::fmt::{Display, Formatter, Write};

// Letter is a simple wrapper around a byte representing a letter by its zero indexed position in
// the alphabet.
//
// Storing them as zero-indexed rather than ASCII introduces a small about of overhead when
// converting to and from strings, but gains us a bit of performance when storing them and looking
// them up in a LetterSet. As the former is done only on startup and output and the latter is done
// literally millions of times, this is a worthwhile trade-off, especially considering that the
// majority of Letter::from_byte calls are at compile time.

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct Letter(u8);

impl Letter {
    pub const NO_LETTER: Self = Self(u8::MAX);

    pub const fn from_ascii(b: u8) -> Self {
        match b {
            b'A'..=b'Z' => Self(b - b'A'),
            b'a'..=b'z' => Self(b - b'a'),
            _ => Self::NO_LETTER,
        }
    }

    pub const fn index(&self) -> u8 {
        self.0
    }

    pub const fn char(&self) -> char {
        (self.0 + b'A') as char
    }

    pub const fn is_valid(&self) -> bool {
        self.0 < 26
    }
}

impl Display for Letter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char(self.char())
    }
}

pub const WORD_LENGTH: usize = 5;
pub type Word = [Letter; WORD_LENGTH];

pub fn make_word(input: &str) -> Option<Word> {
    let bytes: [_; WORD_LENGTH] = input.as_bytes().try_into().ok()?;
    let letters = bytes.map(Letter::from_ascii);
    letters.iter().all(Letter::is_valid).then_some(letters)
}

pub fn letters_to_string<I>(letters: I) -> String
where
    I: IntoIterator<Item = Letter>,
{
    letters.into_iter().map(|l| l.char()).collect()
}

pub fn letters_with_fg<I>(letters: I, color: Color) -> String
where
    I: IntoIterator<Item = Letter>,
{
    letters_to_string(letters).color(color).to_string()
}

pub fn letters_with_bg<I>(letters: I, color: Color) -> String
where
    I: IntoIterator<Item = Letter>,
{
    letters_to_string(letters)
        .on_color(color)
        .color(Color::Black)
        .to_string()
}

pub fn letter_with_fg(letter: Letter, color: Color) -> String {
    letter.to_string().color(color).to_string()
}

pub fn letter_with_bg(letter: Letter, color: Color) -> String {
    letter
        .to_string()
        .on_color(color)
        .color(Color::Black)
        .to_string()
}

pub fn str_with_fg(text: &str, color: Color) -> String {
    String::from(text).color(color).to_string()
}
