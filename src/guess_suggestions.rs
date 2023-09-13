use partial_sort::PartialSort;
use rayon::prelude::*;
use std::cmp::Ordering;

use crate::colored_guess::color_guess;
use crate::knowledge::WordKnowledge;
use crate::possibilities::PossibleAnswer;
use crate::word::Word;

#[derive(PartialEq)]
pub struct ScoredGuess {
    pub word: Word,
    pub green_count: f32,
    pub green_yellow_count: f32,
    pub remaining_words: f32,
}

impl ScoredGuess {
    fn cmp(lhs: &Self, rhs: &Self) -> Ordering {
        f32::total_cmp(&lhs.remaining_words, &rhs.remaining_words)
            .then(f32::total_cmp(&lhs.green_yellow_count, &rhs.green_yellow_count).reverse())
            .then(f32::total_cmp(&lhs.green_count, &rhs.green_count).reverse())
            .then(lhs.word.cmp(&rhs.word))
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
        green_count: green_count as f32 / possibilities.len() as f32,
        green_yellow_count: green_yellow_count as f32 / possibilities.len() as f32,
        remaining_words: remaining_count as f32 / possibilities.len() as f32,
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
