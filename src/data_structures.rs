use crate::word::{Letter, Word, WORD_LENGTH};

use std::cmp::Ordering;

#[derive(Clone, Copy)]
pub struct LetterSet {
    bits: u32,
}

impl LetterSet {
    pub const fn new() -> Self {
        Self { bits: 0 }
    }

    pub fn insert(&mut self, letter: Letter) {
        self.bits |= 1 << (letter - b'A');
    }

    pub const fn contains(self, letter: Letter) -> bool {
        self.bits & (1 << (letter - b'A')) != 0
    }

    pub fn letters(self) -> Vec<Letter> {
        (b'A'..=b'Z').filter(|&l| self.contains(l)).collect()
    }
}

#[derive(Clone)]
pub struct LetterHistogram {
    slots: [Letter; WORD_LENGTH],
}

impl LetterHistogram {
    // This value is specifically chosen to be larger than b'Z'.
    const NO_DATA: Letter = b'_';

    pub const fn new() -> Self {
        Self {
            slots: [Self::NO_DATA; WORD_LENGTH],
        }
    }

    pub fn from_word(word: Word) -> Self {
        let mut histogram = Self::new();
        for letter in word {
            histogram.add_letter(letter);
        }
        histogram
    }

    pub fn add_letter(&mut self, mut letter: Letter) {
        for i in 0..WORD_LENGTH {
            if letter < self.slots[i] {
                std::mem::swap(&mut self.slots[i], &mut letter);
            }
        }
    }

    pub fn remove_letter(&mut self, letter: Letter) {
        for i in 0..WORD_LENGTH {
            if self.slots[i] == letter {
                self.slots[i] = Self::NO_DATA;
                self.slots.sort_unstable();
                break;
            }
        }
    }

    pub fn letters(&self) -> impl Iterator<Item = &Letter> {
        self.slots.iter().take_while(|&l| *l != Self::NO_DATA)
    }

    pub fn contains(&self, letter: Letter) -> bool {
        self.slots.contains(&letter)
    }

    pub fn contains_other(&self, subset: &Self) -> bool {
        let mut i: usize = 0;
        let mut j: usize = 0;
        loop {
            match self.slots[i].cmp(&subset.slots[j]) {
                Ordering::Equal => {
                    i += 1;
                    j += 1;
                }
                Ordering::Greater => {
                    return false;
                }
                Ordering::Less => {
                    i += 1;
                }
            }

            if j == WORD_LENGTH || subset.slots[j] == Self::NO_DATA {
                return true;
            } else if i == WORD_LENGTH || self.slots[i] == Self::NO_DATA {
                return false;
            }
        }
    }

    pub fn merge_via_max(&mut self, other: &Self) {
        let mut new = [b'_'; WORD_LENGTH];
        let mut i = 0;
        let mut j = 0;
        let mut k = 0;
        while k < WORD_LENGTH {
            match self.slots[i].cmp(&other.slots[j]) {
                Ordering::Equal => {
                    new[k] = self.slots[i];
                    i += 1;
                    j += 1;
                    k += 1;
                }
                Ordering::Greater => {
                    new[k] = other.slots[j];
                    j += 1;
                    k += 1;
                }
                Ordering::Less => {
                    new[k] = self.slots[i];
                    i += 1;
                    k += 1;
                }
            }

            if i == WORD_LENGTH {
                while j < WORD_LENGTH && k < WORD_LENGTH {
                    new[k] = other.slots[j];
                    j += 1;
                    k += 1;
                }
                break;
            } else if j == WORD_LENGTH {
                while i < WORD_LENGTH && k < WORD_LENGTH {
                    new[k] = self.slots[i];
                    i += 1;
                    k += 1;
                }
                break;
            }
        }
        self.slots = new;
    }
}
