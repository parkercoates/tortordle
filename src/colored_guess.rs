use crate::alphagram::Alphagram;
use crate::word::{letter_with_bg, Letter, Word};

use colored::Color;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GuessColor {
    Black,
    Yellow,
    Green,
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
}

pub struct ColoredGuess {
    slots: [(Letter, GuessColor); Word::LENGTH],
}

impl ColoredGuess {
    pub fn iter(&self) -> std::slice::Iter<(Letter, GuessColor)> {
        self.slots.iter()
    }

    pub fn formatted(&self) -> String {
        self.iter()
            .map(|(letter, state)| match state {
                Black => letter.to_string(),
                _ => letter_with_bg(*letter, state.color()),
            })
            .collect()
    }

    pub fn weighted_green_yellow_count(&self) -> usize {
        self.iter()
            .map(|(_, state)| match state {
                Black => 0,
                Yellow => 74,
                Green => 100,
            })
            .sum()
    }
}

pub fn color_guess(guess: Word, answer: Word) -> ColoredGuess {
    let mut slots = [(Letter::NO_LETTER, Black); Word::LENGTH];
    let mut yellows = Alphagram::new();
    for (guess_letter, answer_letter, (letter, state)) in
        itertools::izip!(guess, answer, &mut slots)
    {
        *letter = guess_letter;
        if guess_letter == answer_letter {
            *state = Green;
        } else {
            yellows.add_letter(answer_letter);
        }
    }
    for (letter, state) in &mut slots {
        if *state == Black && yellows.remove_letter(*letter) {
            *state = Yellow;
        }
    }
    ColoredGuess { slots }
}
