use crate::letter::{fmt_letters, Letter};

use bitset_core::BitSet;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct LetterSet {
    bits: u32,
}

impl LetterSet {
    pub const fn new() -> Self {
        Self { bits: 0 }
    }

    pub fn insert(&mut self, letter: Letter) {
        self.bits.bit_set(letter.index() as usize);
    }

    pub fn contains(self, letter: Letter) -> bool {
        self.bits.bit_test(letter.index() as usize)
    }
}

impl IntoIterator for LetterSet {
    type Item = Letter;
    type IntoIter = Iter;
    fn into_iter(self) -> Self::IntoIter {
        Iter { bits: self.bits }
    }
}

pub struct Iter {
    bits: u32,
}

impl Iterator for Iter {
    type Item = Letter;
    fn next(&mut self) -> Option<Self::Item> {
        if self.bits.bit_any() {
            let index = self.bits.trailing_zeros();
            self.bits.bit_reset(index as usize);
            Some(Letter::from_index(index as u8))
        } else {
            None
        }
    }
}

impl std::fmt::Debug for LetterSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_letters(self.into_iter(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    use itertools::assert_equal;

    #[test]
    fn test_new() {
        assert_equal(LetterSet::new(), ls(""));
    }

    #[test]
    fn test_insert() {
        let mut set = LetterSet::new();
        assert_equal(set, ls(""));
        set.insert(l('M'));
        assert_equal(set, ls("M"));
        set.insert(l('A'));
        assert_equal(set, ls("AM"));
        set.insert(l('M'));
        assert_equal(set, ls("AM"));
        set.insert(l('M'));
        assert_equal(set, ls("AM"));
        set.insert(l('A'));
        assert_equal(set, ls("AM"));
    }

    #[test]
    fn test_contains() {
        let mut set = LetterSet::new();
        assert!(!set.contains(l('X')));
        assert!(!set.contains(l('Y')));
        assert!(!set.contains(l('Z')));

        set.insert(l('X'));
        assert!(set.contains(l('X')));
        assert!(!set.contains(l('Y')));
        assert!(!set.contains(l('Z')));

        set.insert(l('X'));
        assert!(set.contains(l('X')));
        assert!(!set.contains(l('Y')));
        assert!(!set.contains(l('Z')));

        set.insert(l('Z'));
        assert!(set.contains(l('X')));
        assert!(!set.contains(l('Y')));
        assert!(set.contains(l('Z')));
    }

    #[test]
    fn test_debug() {
        assert_eq!(dbg(lset("")), "");
        assert_eq!(dbg(lset("ACE")), "ACE");
        assert_eq!(dbg(lset("ACER")), "ACER");
    }
}
