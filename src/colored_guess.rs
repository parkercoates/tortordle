use crate::letter::{letter_with_bg, Letter};
use crate::word::Word;

use colored::Color;
use itertools::izip;

// The discriminant of the GuessColor enum stores the weighted value we use to rank the strength of
// different guesses.
//
// Obviously greens are better than yellows, exactly how to weight them with respect to each other
// gets nuanced. Are three yellows better than two greens? After polling a few players, we found
// that weighting a yellow at 74% the value of a green seemed to roughly match the perceived value
// of different combinations. (74% was chosen to avoid a tie between 4 yellows and 3 greens.)

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GuessColor {
    Black = 0,
    Yellow = 74,
    Green = 100,
}

use GuessColor::*;

impl GuessColor {
    pub const fn color(self) -> Color {
        match self {
            Black => Color::Black,
            Yellow => Color::Yellow,
            Green => Color::Green,
        }
    }

    // The color to be used when drawing a visible block
    pub const fn block_color(self) -> Color {
        match self {
            Black => Color::White,
            Yellow => Color::Yellow,
            Green => Color::Green,
        }
    }

    pub const fn weight(self) -> usize {
        self as usize
    }
}

#[derive(Default)]
pub struct ColorCounts {
    greens: usize,
    yellows: usize,
}

fn find_then_remove(letters: &mut [Letter; Word::LENGTH], letter: Letter) -> bool {
    letter.is_valid()
        && letters
            .iter_mut()
            .find(|l| **l == letter)
            .map(|l| *l = Letter::NO_LETTER)
            .is_some()
}

impl ColorCounts {
    pub fn from_guess(guess: Word, answer: Word) -> Self {
        // We are going to mutate the contents of our arguments in ways that break Word invariants,
        // so let's just steal their guts and mutate those arrays directly instead.
        let mut guess = guess.letters;
        let mut answer = answer.letters;

        // On the first pass we count up green matches. When a green match is found, we remove those
        // letters from both guess and answer to get them out of the way when we later count the
        // yellows.
        let mut greens = 0;
        for (answer_letter, guess_letter) in izip!(&mut answer, &mut guess) {
            if *guess_letter == *answer_letter {
                greens += 1;
                *guess_letter = Letter::NO_LETTER;
                *answer_letter = Letter::NO_LETTER;
            }
        }

        // We now loop through the unmatched letters in answer and search for them in the unmatched
        // letters still in guess. Note that if found, we have to remove the letters from guess to
        // properly handle repeated letters.
        let yellows = answer
            .into_iter()
            .filter(|l| find_then_remove(&mut guess, *l))
            .count();

        Self { greens, yellows }
    }

    pub fn green_count(&self) -> usize {
        self.greens
    }
    pub fn yellow_count(&self) -> usize {
        self.yellows
    }
    pub fn green_yellow_count(&self) -> usize {
        self.greens + self.yellows
    }
    pub fn weighted_count(&self) -> usize {
        self.greens * Green.weight() + self.yellows * Yellow.weight()
    }
}

impl std::ops::AddAssign for ColorCounts {
    fn add_assign(&mut self, rhs: Self) {
        self.greens += rhs.greens;
        self.yellows += rhs.yellows;
    }
}

pub struct ColoredGuess {
    slots: [(Letter, GuessColor); Word::LENGTH],
}

impl ColoredGuess {
    pub fn new(guess: Word, answer: Word) -> ColoredGuess {
        // We are going to mutate the contents of answer in ways that break Word invariants, so
        // let's just steal its guts and mutate the array directly instead.
        let mut answer = answer.letters;

        // Guess colouring is a two pass operation. On the first pass we identify green matches.
        // When a green match is found, we remove those letters from both answer to get them out of
        // the way when we later count the yellows.
        let mut slots = [(Letter::NO_LETTER, Black); Word::LENGTH];
        for (guess_letter, answer_letter, (letter, state)) in izip!(guess, &mut answer, &mut slots)
        {
            *letter = guess_letter;
            if guess_letter == *answer_letter {
                *state = Green;
                *answer_letter = Letter::NO_LETTER;
            }
        }

        // We now loop through the slots that are still black and search for those letters in the
        // unmatched letters still in answer. When we find a letter, we have to remove it from
        // answer to properly handle repeated letters.
        for (letter, state) in &mut slots {
            if *state == Black && find_then_remove(&mut answer, *letter) {
                *state = Yellow;
            }
        }
        ColoredGuess { slots }
    }

    pub fn iter(&self) -> impl Iterator<Item = &(Letter, GuessColor)> {
        self.into_iter()
    }

    pub fn formatted(&self) -> String {
        self.iter()
            .map(|(letter, state)| match state {
                Black => letter.to_string(),
                _ => letter_with_bg(*letter, state.color()),
            })
            .collect()
    }
}

impl IntoIterator for ColoredGuess {
    type Item = (Letter, GuessColor);
    type IntoIter = std::array::IntoIter<(Letter, GuessColor), 5>;

    fn into_iter(self) -> Self::IntoIter {
        self.slots.into_iter()
    }
}

impl<'a> IntoIterator for &'a ColoredGuess {
    type Item = &'a (Letter, GuessColor);
    type IntoIter = std::slice::Iter<'a, (Letter, GuessColor)>;

    fn into_iter(self) -> Self::IntoIter {
        self.slots.iter()
    }
}
