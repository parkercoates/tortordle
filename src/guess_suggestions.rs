use partial_sort::PartialSort;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::fmt;

use crate::colored_guess::color_guess;
use crate::knowledge::WordKnowledge;
use crate::possibilities::PossibleAnswer;
use crate::word::Word;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hundredths(i32);

impl Hundredths {
    fn from_div(numerator: usize, denominator: usize) -> Self {
        Self(((100.0 * numerator as f64) / denominator as f64).round() as i32)
    }
}

impl fmt::Display for Hundredths {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self.0 as f64 / 100.0).fmt(f)
    }
}

#[derive(PartialEq, Eq)]
pub struct ScoredGuess {
    pub word: Word,
    pub green_count: Hundredths,
    pub green_yellow_count: Hundredths,
    pub remaining_words: Hundredths,
}

impl Ord for ScoredGuess {
    fn cmp(&self, o: &Self) -> Ordering {
        self.remaining_words
            .cmp(&o.remaining_words)
            .then(self.green_yellow_count.cmp(&o.green_yellow_count).reverse())
            .then(self.green_count.cmp(&o.green_count).reverse())
            .then(self.word.cmp(&o.word))
    }
}

impl PartialOrd for ScoredGuess {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn score_guess(guess: Word, possibilities: &[PossibleAnswer]) -> ScoredGuess {
    let mut green_count: usize = 0;
    let mut green_yellow_count: usize = 0;
    let mut remaining_count: usize = 0;
    for answer in possibilities {
        let colored_guess = color_guess(guess, answer.word);
        let knowledge = WordKnowledge::from_guess(&colored_guess);

        green_count += colored_guess.green_count();
        green_yellow_count += colored_guess.green_yellow_count();
        if answer.word != guess {
            remaining_count += possibilities
                .iter()
                .filter(|pos| knowledge.matches(pos))
                .count();
        }
    }
    ScoredGuess {
        word: guess,
        green_count: Hundredths::from_div(green_count, possibilities.len()),
        green_yellow_count: Hundredths::from_div(green_yellow_count, possibilities.len()),
        remaining_words: Hundredths::from_div(remaining_count, possibilities.len()),
    }
}

pub fn best_guesses(possibilities: &[PossibleAnswer], count: usize) -> Vec<ScoredGuess> {
    let count = std::cmp::min(count, possibilities.len());
    let mut rankings = Vec::<ScoredGuess>::with_capacity(possibilities.len());

    possibilities
        .par_iter()
        .map(|guess| score_guess(guess.word, possibilities))
        .collect_into_vec(&mut rankings);

    rankings.partial_sort(count, ScoredGuess::cmp);
    rankings.truncate(count);
    rankings
}
