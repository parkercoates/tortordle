use colored::{Color, Colorize};
use std::fmt::{Debug, Display, Write};

// Letter is a simple wrapper around a byte representing a letter by its zero indexed position in
// the alphabet.
//
// Storing them as zero-indexed rather than ASCII introduces a small amount of overhead when
// converting to and from strings, but gains us a bit of performance when storing them and looking
// them up in a LetterSet. As the former is done only on startup and output and the latter is done
// literally millions of times, this is a worthwhile trade-off, especially considering that the
// majority of Letter::from_byte calls are at compile time.
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct Letter(u8);

impl Letter {
    pub const NO_LETTER: Self = Self(u8::MAX);

    pub const fn from_index(b: u8) -> Self {
        Self(b)
    }

    pub const fn from_char(c: char) -> Self {
        match c {
            'A'..='Z' => Self(c as u8 - b'A'),
            'a'..='z' => Self(c as u8 - b'a'),
            _ => Self::NO_LETTER,
        }
    }

    pub const fn index(&self) -> u8 {
        self.0
    }

    pub const fn char(&self) -> char {
        if self.is_valid() {
            (self.0 + b'A') as char
        } else {
            '_'
        }
    }

    pub const fn is_valid(&self) -> bool {
        self.0 < 26
    }
}

impl Display for Letter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(self.char())
    }
}

impl Debug for Letter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(self.char())
    }
}

pub fn fmt_letters<I>(letters: I, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
where
    I: IntoIterator<Item = Letter>,
{
    for letter in letters {
        f.write_char(letter.char())?;
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_from_index() {
        assert_eq!(Letter::from_index(0), l('A'));
        assert_eq!(Letter::from_index(15), l('P'));
        assert_eq!(Letter::from_index(25), l('Z'));
        assert!(!Letter::from_index(26).is_valid());
        assert!(!Letter::from_index(u8::MAX).is_valid());
    }

    #[test]
    fn test_from_char() {
        assert_eq!(Letter::from_char('a'), Letter::from_index(0));
        assert_eq!(Letter::from_char('A'), Letter::from_index(0));
        assert_eq!(Letter::from_char('P'), Letter::from_index(15));
        assert_eq!(Letter::from_char('p'), Letter::from_index(15));
        assert_eq!(Letter::from_char('Z'), Letter::from_index(25));
        assert_eq!(Letter::from_char('z'), Letter::from_index(25));
        assert_eq!(Letter::from_char('0'), Letter::NO_LETTER);
        assert_eq!(Letter::from_char(' '), Letter::NO_LETTER);
        assert_eq!(Letter::from_char('_'), Letter::NO_LETTER);
        assert_eq!(Letter::from_char('🙂'), Letter::NO_LETTER);
    }

    #[test]
    fn test_index() {
        for i in u8::MIN..=u8::MAX {
            assert_eq!(i, Letter::from_index(i).index())
        }
    }

    #[test]
    fn test_char() {
        assert_eq!(Letter::from_index(0).char(), 'A');
        assert_eq!(Letter::from_index(15).char(), 'P');
        assert_eq!(Letter::from_index(25).char(), 'Z');
        assert_eq!(Letter::from_index(26).char(), '_');
        assert_eq!(Letter::from_index(u8::MAX).char(), '_');
    }

    #[test]
    fn test_is_valid() {
        assert!(Letter::from_index(0).is_valid());
        assert!(Letter::from_char('p').is_valid());
        assert!(Letter::from_char('Z').is_valid());
        assert!(!Letter::from_index(27).is_valid());
        assert!(!Letter::from_char('7').is_valid());
        assert!(!Letter::NO_LETTER.is_valid());
    }

    #[test]
    fn test_display() {
        assert_eq!(Letter::from_index(0).to_string(), "A");
        assert_eq!(Letter::from_char('p').to_string(), "P");
        assert_eq!(Letter::from_char('Z').to_string(), "Z");
        assert_eq!(Letter::from_index(67).to_string(), "_");
        assert_eq!(Letter::NO_LETTER.to_string(), "_");
    }

    #[test]
    fn test_debug() {
        assert_eq!(dbg(Letter::from_index(0)), "A");
        assert_eq!(dbg(Letter::from_char('p')), "P");
        assert_eq!(dbg(Letter::from_char('Z')), "Z");
        assert_eq!(dbg(Letter::from_index(67)), "_");
        assert_eq!(dbg(Letter::NO_LETTER), "_");
    }
}
