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

#[derive(Clone, Copy, Ord, PartialOrd, PartialEq, Eq)]
pub struct Word {
    pub letters: [Letter; Self::LENGTH],
}

impl Word {
    pub const LENGTH: usize = 5;
    pub const fn from_str(s: &str) -> Option<Word> {
        Self::from_ascii(s.as_bytes())
    }

    pub const fn from_ascii(bytes: &[u8]) -> Option<Word> {
        if bytes.len() != Self::LENGTH {
            return None;
        }
        let mut letters = [Letter::NO_LETTER; Self::LENGTH];
        let mut i = 0;
        while i < Self::LENGTH {
            letters[i] = Letter::from_ascii(bytes[i]);
            if !letters[i].is_valid() {
                return None;
            }
            i += 1;
        }
        Some(Self { letters })
    }

    // Panics if conversion fails. Use only in const evaluations
    pub const fn expect_from_str(s: &str) -> Word {
        konst::option::unwrap!(Self::from_str(s))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Letter> {
        self.letters.iter()
    }
}

impl IntoIterator for Word {
    type Item = Letter;
    type IntoIter = core::array::IntoIter<Letter, 5>;
    fn into_iter(self) -> Self::IntoIter {
        self.letters.into_iter()
    }
}

impl<'a> IntoIterator for &'a Word {
    type Item = &'a Letter;
    type IntoIter = core::slice::Iter<'a, Letter>;
    fn into_iter(self) -> Self::IntoIter {
        self.letters.iter()
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for letter in self.letters {
            f.write_char(letter.char())?;
        }
        Ok(())
    }
}

fn letters_to_string<I>(letters: I) -> String
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
