use crate::alphagram::Alphagram;
use crate::letter::{letter_with_bg, Letter};
use crate::word::Word;

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

    // The color to be used when drawing a visible block
    pub const fn block_color(self) -> Color {
        match self {
            Black => Color::White,
            Yellow => Color::Yellow,
            Green => Color::Green,
        }
    }
}

pub struct ColoredGuess {
    slots: [(Letter, GuessColor); Word::LENGTH],
}

impl ColoredGuess {
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
            yellows.insert(answer_letter);
        }
    }
    for (letter, state) in &mut slots {
        if *state == Black && yellows.remove(*letter) {
            *state = Yellow;
        }
    }
    ColoredGuess { slots }
}
