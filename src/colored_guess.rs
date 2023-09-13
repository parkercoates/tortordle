use crate::data_structures::LetterHistogram;
use crate::word::{letter_with_bg, Letter, Word, WORD_LENGTH};

use colored::Color;
use itertools::Itertools;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GuessColor {
    Black,
    Yellow,
    Green,
}

impl GuessColor {
    pub const fn color(self) -> Color {
        match self {
            Self::Black => Color::Black,
            Self::Yellow => Color::Yellow,
            Self::Green => Color::Green,
        }
    }
}

pub struct ColoredGuess {
    slots: [(Letter, GuessColor); WORD_LENGTH],
}

impl ColoredGuess {
    pub fn iter(&self) -> std::slice::Iter<(Letter, GuessColor)> {
        self.slots.iter()
    }

    pub fn formatted(&self) -> String {
        self.iter()
            .map(|(letter, state)| letter_with_bg(*letter, state.color()))
            .join("")
    }

    pub fn green_count(&self) -> usize {
        self.iter()
            .filter(|(_, state)| *state == GuessColor::Green)
            .count()
    }

    pub fn green_yellow_count(&self) -> usize {
        self.iter()
            .filter(|(_, state)| *state != GuessColor::Black)
            .count()
    }
}

pub fn color_guess(guess: Word, answer: Word) -> ColoredGuess {
    let mut slots = [(b' ', GuessColor::Black); WORD_LENGTH];
    let mut yellows = LetterHistogram::new();
    for (guess_letter, answer_letter, (letter, state)) in
        itertools::izip!(guess, answer, &mut slots)
    {
        *letter = guess_letter;
        if guess_letter == answer_letter {
            *state = GuessColor::Green;
        } else {
            yellows.add_letter(answer_letter);
        }
    }
    for (letter, state) in &mut slots {
        if *state == GuessColor::Black && yellows.contains(*letter) {
            *state = GuessColor::Yellow;
            yellows.remove_letter(*letter);
        }
    }
    ColoredGuess { slots }
}
