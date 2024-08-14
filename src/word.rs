use crate::letter::{fmt_letters, Letter};

use std::fmt::{Debug, Display};

#[derive(Clone, Copy, Ord, PartialOrd, PartialEq, Eq)]
pub struct Word {
    pub letters: [Letter; Self::LENGTH],
}

impl Word {
    pub const LENGTH: usize = 5;

    pub const fn from_str(s: &str) -> Option<Word> {
        let bytes = s.as_bytes();
        if bytes.len() != Self::LENGTH {
            return None;
        }
        let mut letters = [Letter::NO_LETTER; Self::LENGTH];
        let mut i = 0;
        while i < Self::LENGTH {
            letters[i] = Letter::from_char(bytes[i] as char);
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_letters(self.letters, f)
    }
}

impl Debug for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_letters(self.letters, f)
    }
}
