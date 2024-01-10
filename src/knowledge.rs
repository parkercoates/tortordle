use crate::alphagram::Alphagram;
use crate::colored_guess::{ColoredGuess, GuessColor};
use crate::letter_set::LetterSet;
use crate::possibilities::PossibleAnswer;
use crate::word::*;
use arrayvec::ArrayVec;

use colored::Color;
use itertools::{izip, Itertools};

#[derive(Clone, Copy)]
enum LetterKnowledge {
    Is(Letter),
    IsNot(LetterSet),
}
use LetterKnowledge::*;

impl LetterKnowledge {
    fn set_letter(&mut self, letter: Letter) {
        *self = Is(letter);
    }

    fn remove_letter(&mut self, letter: Letter) {
        match self {
            Is(_) => (),
            IsNot(set) => set.insert(letter),
        }
    }

    fn formatted(self, yellows: Alphagram) -> String {
        match self {
            Is(letter) => letter_with_fg(letter, Color::Green),
            IsNot(set) => {
                letters_with_fg(
                    yellows.letters().dedup().filter(|&l| !set.contains(*l)),
                    Color::Yellow,
                ) + &letters_with_fg(&set.letters(), Color::Red)
            }
        }
    }

    const fn matches(self, letter: Letter) -> bool {
        match self {
            Is(known_letter) => letter == known_letter,
            IsNot(set) => !set.contains(letter),
        }
    }

    const fn could_still_be(self, letter: Letter) -> bool {
        match self {
            Is(_) => false,
            IsNot(set) => !set.contains(letter),
        }
    }
}

pub struct WordKnowledge {
    slots: [LetterKnowledge; WORD_LENGTH],
    yellows: Alphagram,
}

impl WordKnowledge {
    pub const fn new() -> Self {
        Self {
            slots: [IsNot(LetterSet::new()); WORD_LENGTH],
            yellows: Alphagram::new(),
        }
    }

    // from_guess is a stripped down version of add_guess that doesn't have to deal with the
    // complications of existing knowledge.
    pub fn from_guess(guess: &ColoredGuess) -> Self {
        let mut yellows = Alphagram::new();
        let mut slots = [IsNot(LetterSet::new()); WORD_LENGTH];
        for (i, (letter, color)) in guess.iter().enumerate() {
            match color {
                GuessColor::Green => {
                    slots[i].set_letter(*letter);
                }
                GuessColor::Yellow => {
                    yellows.add_letter(*letter);
                    slots[i].remove_letter(*letter);
                }
                GuessColor::Black => {
                    if yellows.contains(*letter) {
                        slots[i].remove_letter(*letter);
                    } else {
                        slots.iter_mut().for_each(|s| s.remove_letter(*letter));
                    }
                }
            }
        }
        Self { slots, yellows }
    }

    pub fn add_guess(&mut self, guess: &ColoredGuess) {
        // First, we compute the alphagram of all letters we already knew were in the word.
        let mut old_letters = self.yellows;
        for slot in &self.slots {
            if let Is(letter) = slot {
                old_letters.add_letter(*letter);
            }
        }

        // Second, we add information from the guess into the existing slots and build up new
        // all-letter and yellow-letter alphagrams for this guess.
        let mut new_letters = Alphagram::new();
        let mut new_yellows = Alphagram::new();
        for (i, (letter, color)) in guess.iter().enumerate() {
            match color {
                GuessColor::Green => {
                    new_letters.add_letter(*letter);
                    self.slots[i].set_letter(*letter);
                }
                GuessColor::Yellow => {
                    new_letters.add_letter(*letter);
                    new_yellows.add_letter(*letter);
                    self.slots[i].remove_letter(*letter);
                }
                GuessColor::Black => {
                    if new_yellows.contains(*letter) {
                        // If we've already seen this letter as yellow to the left of this slot in
                        // _this_ guess, seeing it again as black only tells us that that this
                        // specific slot can't be that letter.
                        self.slots[i].remove_letter(*letter);
                    } else {
                        // Otherwise, we know that letter occurs in no slot.
                        self.slots.iter_mut().for_each(|s| s.remove_letter(*letter));
                    }
                }
            }
        }

        // Third, we get the new all-letter alphagram by merging the previous alphagram with the
        // alphagram for this guess, taking the higher count of the two for each letter.
        //
        // Note that we can't just use `new_letters` because of non-hard mode players. `new_letters`
        // may be missing letters from previous guesses.
        new_letters.merge_via_max(old_letters);

        // Fourth, we compute the new set of yellows by starting with the new all-letter alphagram
        // and removing all the greens.
        //
        // Note that we can't just use `new_yellows` because of non-hard mode players. The new guess
        // could conflict with previous guesses, meaning `new_yellows` could contain letters that
        // were green in previous guesses or be missing yellow letters from previous guesses.
        self.yellows = new_letters;
        for slot in &self.slots {
            if let Is(letter) = slot {
                self.yellows.remove_letter(*letter);
            }
        }

        // Finally, we search for yellow letters whose count exactly equals the number of slots
        // where that letter could possibly be put, allowing us to place the letter in those slots
        // and remove it from yellows.
        //
        // Note that this has to support the extremely rare case of having two yellows of the same
        // letter and only two slots they could match. (Something I've never seen happen in a real
        // game.)
        const MAX_POTENTIAL_SLOTS: usize = WORD_LENGTH - 1;
        for (count, &letter) in self.yellows.clone().letters().dedup_with_count() {
            let potential_slots = self
                .slots
                .iter_mut()
                .filter(|slot| slot.could_still_be(letter))
                .collect::<ArrayVec<&mut LetterKnowledge, MAX_POTENTIAL_SLOTS>>();
            if potential_slots.len() == count {
                for slot in potential_slots {
                    slot.set_letter(letter);
                    self.yellows.remove_letter(letter);
                }
            }
        }
    }

    pub fn check_for_conflicts(&self, word: Word) -> Vec<Conflict> {
        let mut conflicts = Vec::new();
        for (i, letter, slot) in izip!(0..WORD_LENGTH, word, self.slots) {
            match slot {
                Is(known) => {
                    if known != letter {
                        conflicts.push(Conflict::MustBe(i, known));
                    }
                }
                IsNot(set) => {
                    if set.contains(letter) {
                        conflicts.push(Conflict::CannotBe(i, letter));
                    }
                }
            }
        }
        for (count_needed, &letter) in self.yellows.letters().dedup_with_count() {
            let count_found = word.into_iter().filter(|l| *l == letter).count();
            if count_found < count_needed {
                conflicts.push(Conflict::Missing(letter, count_needed));
            }
        }
        conflicts
    }

    // This is essentially just a much faster version of
    // `check_for_conflicts(possibility.word).is_empty()`
    pub fn matches(&self, possibility: &PossibleAnswer) -> bool {
        std::iter::zip(&self.slots, possibility.word).all(|(s, l)| s.matches(l))
            && possibility.alphagram.contains_other(self.yellows)
    }

    pub fn formatted(&self) -> String {
        format!(
            "[{}]",
            self.slots
                .iter()
                .map(|slot| slot.formatted(self.yellows))
                .join("|")
        )
    }

    pub fn format_word(&self, word: Word) -> String {
        std::iter::zip(word, &self.slots)
            .map(|(letter, slot)| match slot {
                Is(known_letter) if letter == *known_letter => letter_with_fg(letter, Color::Green),
                _ if self.yellows.contains(letter) => letter_with_fg(letter, Color::Yellow),
                _ => letter_to_string(letter),
            })
            .join("")
    }
}

#[derive(Clone, Copy)]
pub enum Conflict {
    MustBe(usize, Letter),
    CannotBe(usize, Letter),
    Missing(Letter, usize),
}

impl Conflict {
    pub fn as_text(self) -> String {
        match self {
            Self::MustBe(i, letter) => format!(
                "The {} letter must be {}.",
                index_to_ordinal(i),
                add_indefinite_article_to_letter(letter)
            ),
            Self::CannotBe(i, letter) => format!(
                "The {} letter cannot be {}.",
                index_to_ordinal(i),
                add_indefinite_article_to_letter(letter)
            ),
            Self::Missing(letter, needed) => {
                if needed == 1 {
                    format!(
                        "The word must contain {}.",
                        add_indefinite_article_to_letter(letter)
                    )
                } else {
                    format!("The word must contain {needed} {}'s.", letter as char)
                }
            }
        }
    }
}

fn index_to_ordinal(index: usize) -> &'static str {
    match index {
        0 => "first",
        1 => "second",
        2 => "third",
        3 => "fourth",
        4 => "fifth",
        _ => panic!("Wordle letter indexes never get bigger than 4!"),
    }
}

fn add_indefinite_article_to_letter(letter: Letter) -> String {
    let c = letter as char;
    match c {
        'A' | 'E' | 'F' | 'H' | 'I' | 'L' | 'M' | 'N' | 'O' | 'R' | 'S' | 'X' => format!("an {c}"),
        _ => format!("a {c}"),
    }
}
