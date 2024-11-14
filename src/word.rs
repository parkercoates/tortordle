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
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Letter> {
        self.into_iter()
    }
}

impl IntoIterator for Word {
    type Item = Letter;
    type IntoIter = core::array::IntoIter<Letter, { Self::LENGTH }>;
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

impl<'a> IntoIterator for &'a mut Word {
    type Item = &'a mut Letter;
    type IntoIter = core::slice::IterMut<'a, Letter>;
    fn into_iter(self) -> Self::IntoIter {
        self.letters.iter_mut()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    use itertools::assert_equal;

    #[test]
    fn test_from_str() {
        assert!(Word::from_str("").is_none());
        assert!(Word::from_str("a").is_none());
        assert!(Word::from_str("abcd").is_none());
        assert!(Word::from_str("abcdef").is_none());
        assert!(Word::from_str("ab cd").is_none());
        assert!(Word::from_str("ab.cd").is_none());
        assert!(Word::from_str("àbcdé").is_none());

        assert_eq!(
            Word::from_str("ABCDE"),
            Some(Word {
                letters: [l('A'), l('B'), l('C'), l('D'), l('E')]
            })
        );
        assert_eq!(
            Word::from_str("abcde"),
            Some(Word {
                letters: [l('A'), l('B'), l('C'), l('D'), l('E')]
            })
        );
    }

    #[test]
    fn test_expect_from_str() {
        assert!(std::panic::catch_unwind(|| Word::expect_from_str("abcd")).is_err());
        assert_eq!(Word::expect_from_str("ABCDE"), w("ABCDE"));
    }

    #[test]
    fn test_iter() {
        assert_equal(w("ZYXWV").iter().copied(), ls("ZYXWV"));
    }

    #[test]
    fn test_into_iter() {
        assert_equal(w("ZYXWV").into_iter(), ls("ZYXWV"));
    }

    #[test]
    fn test_display() {
        assert_eq!(w("LMNOP").to_string(), "LMNOP");
    }

    #[test]
    fn test_debug() {
        assert_eq!(dbg(w("LMNOP")), "LMNOP");
    }
}
