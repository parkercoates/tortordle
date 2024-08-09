use crate::word::{fmt_letters, Letter, Word};

use std::cmp::Ordering;
use std::fmt::Debug;

// An alphagram is the set of letters, listed in alphabetical order. The term is
// taken from competitive Scrabble, where it is the most common way of racking
// one's tiles. Apparently alphagrams are also used in memorising the Scrabble
// word list.

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Alphagram {
    slots: [Letter; Self::LENGTH],
}

impl Alphagram {
    const LENGTH: usize = Word::LENGTH;

    pub const fn new() -> Self {
        Self {
            slots: [Letter::NO_LETTER; Self::LENGTH],
        }
    }

    pub const fn from_word(word: Word) -> Self {
        Self {
            slots: sorted5(word.letters),
        }
    }

    pub fn is_full(self) -> bool {
        self.slots[Self::LENGTH - 1] != Letter::NO_LETTER
    }

    pub fn insert(&mut self, mut letter: Letter) {
        debug_assert!(!self.is_full());
        for slot in &mut self.slots {
            if letter < *slot {
                std::mem::swap(slot, &mut letter);
            }
        }
    }

    pub fn remove(&mut self, letter: Letter) -> bool {
        for i in 0..Self::LENGTH {
            if self.slots[i] == letter {
                for j in i..Self::LENGTH - 1 {
                    self.slots[j] = self.slots[j + 1];
                }
                self.slots[Self::LENGTH - 1] = Letter::NO_LETTER;
                return true;
            }
        }
        false
    }

    pub fn contains(self, letter: Letter) -> bool {
        self.slots.contains(&letter)
    }

    pub fn contains_other(self, other: Self) -> bool {
        let mut letters_to_find = other.into_iter();
        let Some(mut letter_to_find) = letters_to_find.next() else {
            // If there are no letters to find, we've already succeeded.
            return true;
        };

        // We walk through `self` looking for the elements of `letters_to_find`. Because both are
        // sorted, we can do this in a single pass, considering only the smallest remaining element
        // of each alphagram.
        for letter in self.slots {
            match letter.cmp(&letter_to_find) {
                Ordering::Greater => {
                    // The current letter is greater than `letter_to_find`, meaning we have no hope
                    // of ever finding it.
                    return false;
                }
                Ordering::Equal => {
                    // We found the letter we were looking for, so we advance to the next. If
                    // `letters_to_find` is done, we have found them all.
                    letter_to_find = match letters_to_find.next() {
                        Some(letter) => letter,
                        None => return true,
                    };
                }
                Ordering::Less => {
                    // The current letter is not one we are looking for. Continue.
                }
            }
        }
        // We reached the end of `self` without finding all the letters.
        false
    }

    // Merges the two Alphagrams, keeping the higher count for every letter encountered. Since
    // Alphagram has a fixed capacity, the result is undefined if the merged capacity exceeds it.
    pub fn merged(self, other: Self) -> Self {
        let mut i = 0;
        let mut j = 0;
        let mut slots = [Letter::NO_LETTER; Self::LENGTH];
        for slot in &mut slots {
            match self.slots[i].cmp(&other.slots[j]) {
                Ordering::Equal => {
                    *slot = self.slots[i];
                    i += 1;
                    j += 1;
                }
                Ordering::Greater => {
                    *slot = other.slots[j];
                    j += 1;
                }
                Ordering::Less => {
                    *slot = self.slots[i];
                    i += 1;
                }
            }
        }
        // Check that no letters went unused.
        debug_assert!(i == Self::LENGTH || self.slots[i] == Letter::NO_LETTER);
        debug_assert!(j == Self::LENGTH || other.slots[j] == Letter::NO_LETTER);
        Self { slots }
    }
}

impl Debug for Alphagram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_letters(self.slots, f)
    }
}

impl IntoIterator for Alphagram {
    type Item = Letter;
    type IntoIter = Iter;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            slots: self.slots,
            index: 0,
        }
    }
}

pub struct Iter {
    slots: [Letter; Alphagram::LENGTH],
    index: usize,
}

impl Iterator for Iter {
    type Item = Letter;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.slots.len() {
            let letter = self.slots[self.index];
            self.index += 1;
            letter.is_valid().then_some(letter)
        } else {
            None
        }
    }
}

// An optimal sorting network for 5 elements.
// Taken from https://bertdobbelaere.github.io/sorting_networks.html#N5L9D5
const fn sorted5(mut arr: [Letter; 5]) -> [Letter; 5] {
    macro_rules! compare_exchange {
        ($arr:ident, $i:literal, $j:literal) => {
            if $arr[$j].index() < $arr[$i].index() {
                let temp = $arr[$i];
                $arr[$i] = $arr[$j];
                $arr[$j] = temp;
            }
        };
    }

    compare_exchange!(arr, 0, 3);
    compare_exchange!(arr, 1, 4);
    compare_exchange!(arr, 0, 2);
    compare_exchange!(arr, 1, 3);
    compare_exchange!(arr, 0, 1);
    compare_exchange!(arr, 2, 4);
    compare_exchange!(arr, 1, 2);
    compare_exchange!(arr, 3, 4);
    compare_exchange!(arr, 2, 3);
    arr
}
