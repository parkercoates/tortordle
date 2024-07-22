use crate::alphagram::Alphagram;
use crate::colored_guess::{ColoredGuess, GuessColor};
use crate::letter_set::LetterSet;
use crate::possibilities::PossibleAnswer;
use crate::word::*;
use arrayvec::ArrayVec;

use colored::Color;
use itertools::{izip, Itertools};

enum LetterKnowledge {
    Is(Letter),
    IsNot(LetterSet),
}
use LetterKnowledge::*;

impl LetterKnowledge {
    const fn new() -> Self {
        IsNot(LetterSet::new())
    }

    fn set_letter(&mut self, letter: Letter) {
        *self = Is(letter);
    }

    fn remove_letter(&mut self, letter: Letter) {
        match self {
            Is(_) => (),
            IsNot(set) => set.insert(letter),
        }
    }

    fn formatted(&self, yellows: Alphagram) -> String {
        match self {
            Is(letter) => letter_with_fg(*letter, Color::Green),
            IsNot(set) => {
                letters_with_fg(
                    yellows.into_iter().dedup().filter(|&l| !set.contains(l)),
                    Color::Yellow,
                ) + &letters_with_fg(set.letters(), Color::Red)
            }
        }
    }

    fn matches(&self, letter: Letter) -> bool {
        match self {
            Is(known_letter) => letter == *known_letter,
            IsNot(set) => !set.contains(letter),
        }
    }

    const fn could_still_be(&self, letter: Letter) -> bool {
        match self {
            Is(_) => false,
            IsNot(set) => !set.contains(letter),
        }
    }
}

pub struct WordKnowledge {
    slots: [LetterKnowledge; Word::LENGTH],
    all_letters: Alphagram,
    yellows: Alphagram,
}

impl WordKnowledge {
    pub const fn new() -> Self {
        Self {
            slots: [const { LetterKnowledge::new() }; Word::LENGTH],
            all_letters: Alphagram::new(),
            yellows: Alphagram::new(),
        }
    }

    // from_guess is an unholy, stripped down combination of color_guess and add_guess focused
    // purely on performance. Because it only creates new Knowledge, it doesn't have to deal with
    // the complications of existing knowledge and conflicting guesses, allowing it to cut a lot of
    // corners.
    pub fn from_guess(guess: Word, answer: Word) -> Self {
        let mut result = Self::new();
        let mut potential_yellows = Alphagram::new();
        for (guess_letter, answer_letter, slot) in izip!(guess, answer, &mut result.slots) {
            if guess_letter == answer_letter {
                slot.set_letter(guess_letter);
                result.all_letters.insert(guess_letter);
            } else {
                slot.remove_letter(guess_letter);
                potential_yellows.insert(answer_letter);
            }
        }

        for (i, &letter) in guess.iter().enumerate() {
            if let IsNot(_) = result.slots[i] {
                if potential_yellows.remove(letter) {
                    result.yellows.insert(letter);
                    result.all_letters.insert(letter);
                } else if !result.yellows.contains(letter) {
                    result
                        .slots
                        .iter_mut()
                        .for_each(|s| s.remove_letter(letter));
                }
            }
        }
        result
    }

    pub fn add_guess(&mut self, guess: &ColoredGuess) {
        // First, we add information from the guess into the existing slots and build up the
        // all-letter and yellow-letter alphagrams for _this_ guess.
        let mut new_letters = Alphagram::new();
        let mut new_yellows = Alphagram::new();
        for (i, (letter, color)) in guess.iter().enumerate() {
            match color {
                GuessColor::Green => {
                    new_letters.insert(*letter);
                    self.slots[i].set_letter(*letter);
                }
                GuessColor::Yellow => {
                    new_letters.insert(*letter);
                    new_yellows.insert(*letter);
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

        // Second, we update our all_letter alphagram by merging the all-letter alphagram for this
        // guess into it, taking the higher count of the two for each letter.
        //
        // Note that we can't just use `new_letters` because of non-hard mode players. `new_letters`
        // may be missing letters from previous guesses.
        self.all_letters.merge_via_max(new_letters);

        // Third, we compute the set of yellows by removing the greens from the all-letter
        // alphagram.
        //
        // Note that we can't just use `new_yellows` because of non-hard mode players. The new guess
        // could conflict with previous guesses, meaning `new_yellows` could contain letters that
        // were green in previous guesses and/or missing yellow letters from previous guesses.
        self.yellows = self.all_letters;
        for slot in &self.slots {
            if let Is(letter) = slot {
                self.yellows.remove(*letter);
            }
        }

        // Finally, we search for yellow letters whose count exactly equals the number of slots
        // where that letter could possibly be put, allowing us to place the letter in those slots
        // and remove it from self.yellows.
        //
        // Note that this has to support the extremely rare case of having two yellows of the same
        // letter and only two slots they could match. (Something I've never seen happen in a real
        // game.)
        const MAX_POTENTIAL_SLOTS: usize = Word::LENGTH - 1;
        for (count, letter) in self.yellows.into_iter().dedup_with_count() {
            let potential_slots = self
                .slots
                .iter_mut()
                .filter(|slot| slot.could_still_be(letter))
                .collect::<ArrayVec<&mut LetterKnowledge, MAX_POTENTIAL_SLOTS>>();
            if potential_slots.len() == count {
                for slot in potential_slots {
                    slot.set_letter(letter);
                    self.yellows.remove(letter);
                }
            }
        }
    }

    pub fn check_for_conflicts(&self, word: Word) -> Vec<Conflict> {
        let mut conflicts = Vec::new();
        for (i, letter, slot) in izip!(0..Word::LENGTH, word, &self.slots) {
            match slot {
                Is(known) => {
                    if *known != letter {
                        conflicts.push(Conflict::MustBe(i, *known));
                    }
                }
                IsNot(set) => {
                    if set.contains(letter) {
                        conflicts.push(Conflict::CannotBe(i, letter));
                    }
                }
            }
        }
        for (count_needed, letter) in self.yellows.into_iter().dedup_with_count() {
            let count_found = word.iter().filter(|&&l| l == letter).count();
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
            && possibility.alphagram.contains_other(self.all_letters)
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
                _ => letter.to_string(),
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
                    format!("The word must contain {needed} {}'s.", letter.char())
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
    let c = letter.char();
    match c {
        'A' | 'E' | 'F' | 'H' | 'I' | 'L' | 'M' | 'N' | 'O' | 'R' | 'S' | 'X' => format!("an {c}"),
        _ => format!("a {c}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color_guess;

    fn w(s: &str) -> Word {
        Word::expect_from_str(s)
    }

    fn g(word: &str, answer: &str) -> ColoredGuess {
        color_guess(w(word), w(answer))
    }

    fn p(s: &str) -> PossibleAnswer {
        PossibleAnswer::from_word(w(s))
    }

    #[test] // GH-1
    fn same_letter_green_and_yellow() {
        let mut k = WordKnowledge::new();
        k.add_guess(&g("CHEER", "EMBER"));

        assert!(k.matches(&p("METER")));
        assert!(!k.matches(&p("DRYER")));
    }

    #[test] // GH-1: This scenario is extremely unlikely, but bugs is bugs.
    fn same_letter_green_and_green_and_yellow() {
        let mut k = WordKnowledge::new();
        k.add_guess(&g("ERROR", "RARER"));

        assert!(k.matches(&p("RARER")));
        assert!(!k.matches(&p("PURER")));
    }
}
