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

    pub fn insert(&mut self, mut letter: Letter) {
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

    pub fn into_iter(self) -> impl Iterator<Item = Letter> {
        self.slots.into_iter().take_while(Letter::is_valid)
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

    pub fn merge_via_max(&mut self, other: Self) {
        let mut new = [Letter::NO_LETTER; Self::LENGTH];
        let mut i = 0;
        let mut j = 0;
        let mut k = 0;
        while k < Self::LENGTH {
            match self.slots[i].cmp(&other.slots[j]) {
                std::cmp::Ordering::Equal => {
                    new[k] = self.slots[i];
                    i += 1;
                    j += 1;
                    k += 1;
                }
                std::cmp::Ordering::Greater => {
                    new[k] = other.slots[j];
                    j += 1;
                    k += 1;
                }
                std::cmp::Ordering::Less => {
                    new[k] = self.slots[i];
                    i += 1;
                    k += 1;
                }
            }

            if i == Self::LENGTH {
                while j < Self::LENGTH && k < Self::LENGTH {
                    new[k] = other.slots[j];
                    j += 1;
                    k += 1;
                }
                break;
            } else if j == Self::LENGTH {
                while i < Self::LENGTH && k < Self::LENGTH {
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

impl Debug for Alphagram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_letters(self.slots, f)
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
