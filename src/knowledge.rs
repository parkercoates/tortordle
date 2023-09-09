use crate::colored_guess::{ColoredGuess, GuessColor};
use crate::data_structures::{LetterHistogram, LetterSet};
use crate::possibilities::PossibleAnswer;
use crate::word::*;

use colored::Color;
use itertools::Itertools;

#[derive(Clone, Copy)]
enum LetterKnowledge {
    Is(Letter),
    IsNot(LetterSet),
}

impl LetterKnowledge {
    fn formatted(self, yellows: LetterHistogram) -> String {
        match self {
            Self::Is(letter) => letter_with_fg(letter, Color::Green),
            Self::IsNot(set) => {
                letters_with_fg(
                    yellows.letters().dedup().filter(|&l| !set.contains(*l)),
                    Color::Yellow,
                ) + &letters_with_fg(&set.letters(), Color::Red)
            }
        }
    }

    const fn matches(self, letter: Letter) -> bool {
        match self {
            Self::Is(known_letter) => letter == known_letter,
            Self::IsNot(set) => !set.contains(letter),
        }
    }
}

pub struct WordKnowledge {
    slots: [LetterKnowledge; WORD_LENGTH],
    histogram: LetterHistogram,
    yellows: LetterHistogram,
}

impl WordKnowledge {
    pub const fn new() -> Self {
        Self {
            slots: [LetterKnowledge::IsNot(LetterSet::new()); 5],
            histogram: LetterHistogram::new(),
            yellows: LetterHistogram::new(),
        }
    }

    pub fn from_guess(guess: &ColoredGuess) -> Self {
        let mut result = Self::new();
        result.add_guess(guess);
        result
    }

    pub fn add_guess(&mut self, guess: &ColoredGuess) {
        let mut new_histogram = LetterHistogram::new();
        let mut new_yellows = LetterHistogram::new();
        for (i, (letter, color)) in guess.iter().enumerate() {
            match color {
                GuessColor::Black => {
                    // If we've already seen a particular letter in yellow in this
                    // guess, seeing it in black only tells us that that this
                    // specific slot can't be that letter.
                    if new_yellows.contains(*letter) {
                        if let LetterKnowledge::IsNot(set) = &mut self.slots[i] {
                            set.insert(*letter);
                        }
                    // Otherwise, we know that letter occurs in no slot.
                    } else {
                        for slot in &mut self.slots {
                            if let LetterKnowledge::IsNot(set) = slot {
                                set.insert(*letter);
                            }
                        }
                    }
                }
                GuessColor::Yellow => {
                    new_histogram.add_letter(*letter);
                    new_yellows.add_letter(*letter);
                    if let LetterKnowledge::IsNot(set) = &mut self.slots[i] {
                        set.insert(*letter);
                    }
                }
                GuessColor::Green => {
                    new_histogram.add_letter(*letter);
                    self.slots[i] = LetterKnowledge::Is(*letter);
                }
            }
        }

        self.histogram.merge_via_max(new_histogram);

        self.yellows = self.histogram;
        for slot in &self.slots {
            if let LetterKnowledge::Is(letter) = slot {
                self.yellows.remove_letter(*letter);
            }
        }

        for (count, &letter) in self.yellows.clone().letters().dedup_with_count() {
            let matches = self
                .slots
                .iter()
                .filter(|slot| slot.matches(letter))
                .count();
            if matches == count {
                for slot in &mut self.slots {
                    if let LetterKnowledge::IsNot(_) = slot {
                        if slot.matches(letter) {
                            *slot = LetterKnowledge::Is(letter);
                            self.yellows.remove_letter(letter);
                        }
                    }
                }
            }
        }
    }

    pub fn matches(&self, possibility: &PossibleAnswer) -> bool {
        let slots_match = std::iter::zip(&self.slots, possibility.word).all(|(s, l)| s.matches(l));
        let needs_match = possibility.histogram.contains_other(self.histogram);
        slots_match && needs_match
    }

    pub fn matches_word(&self, word: Word) -> bool {
        self.matches(&PossibleAnswer::from_word(word))
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

    pub fn format_possibility(&self, possibility: &PossibleAnswer) -> String {
        std::iter::zip(possibility.word, &self.slots)
            .map(|(letter, slot)| match slot {
                LetterKnowledge::Is(known_letter) if letter == *known_letter => {
                    letter_with_fg(letter, Color::Green)
                }
                _ if self.yellows.contains(letter) => letter_with_fg(letter, Color::Yellow),
                _ => letter_to_string(letter),
            })
            .join("")
    }
}
