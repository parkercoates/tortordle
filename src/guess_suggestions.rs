use partial_sort::PartialSort;
use rayon::prelude::*;
use std::cmp::Ordering;

use crate::colored_guess::color_guess;
use crate::knowledge::WordKnowledge;
use crate::possibilities::PossibleAnswer;
use crate::word::Word;

#[derive(PartialEq, PartialOrd)]
pub struct ScoredGuess {
    pub word: Word,
    pub score: f32,
}

impl ScoredGuess {
    fn cmp_ascending_score(lhs: &Self, rhs: &Self) -> Ordering {
        lhs.score
            .total_cmp(&rhs.score)
            .then(lhs.word.cmp(&rhs.word))
    }

    fn cmp_descending_score(lhs: &Self, rhs: &Self) -> Ordering {
        lhs.score
            .total_cmp(&rhs.score)
            .reverse()
            .then(lhs.word.cmp(&rhs.word))
    }
}

fn average_remaining_possibilites(guess: Word, possibilities: &[PossibleAnswer]) -> f32 {
    let mut count = 0usize;
    for answer in possibilities {
        if answer.word != guess {
            let knowledge = WordKnowledge::from_guess(&color_guess(guess, answer.word));
            for possibility in possibilities {
                if knowledge.matches(possibility) {
                    count += 1;
                }
            }
        }
    }
    count as f32 / possibilities.len() as f32
}

pub fn best_guesses_by_remaining_possibilities(
    possibilities: &[PossibleAnswer],
    count: usize,
) -> Vec<ScoredGuess> {
    let count = std::cmp::min(count, possibilities.len());
    let mut rankings = Vec::<ScoredGuess>::with_capacity(possibilities.len());

    possibilities
        .par_iter()
        .map(|guess| ScoredGuess {
            word: guess.word,
            score: average_remaining_possibilites(guess.word, possibilities),
        })
        .collect_into_vec(&mut rankings);

    rankings.partial_sort(count, ScoredGuess::cmp_ascending_score);
    rankings.truncate(count);
    rankings
}

fn average_green_count(guess: Word, possibilities: &[PossibleAnswer]) -> f32 {
    let count: usize = possibilities
        .iter()
        .map(|possibility| color_guess(guess, possibility.word).green_count())
        .sum();
    count as f32 / possibilities.len() as f32
}

pub fn best_guesses_by_green_count(
    possibilities: &[PossibleAnswer],
    count: usize,
) -> Vec<ScoredGuess> {
    let count = std::cmp::min(count, possibilities.len());
    let mut rankings = Vec::<ScoredGuess>::with_capacity(possibilities.len());

    possibilities
        .par_iter()
        .map(|guess| ScoredGuess {
            word: guess.word,
            score: average_green_count(guess.word, possibilities),
        })
        .collect_into_vec(&mut rankings);

    rankings.partial_sort(count, ScoredGuess::cmp_descending_score);
    rankings.truncate(count);
    rankings
}

pub fn average_weighted_green_yellow_count(guess: Word, possibilities: &[PossibleAnswer]) -> f32 {
    let count: f32 = possibilities
        .iter()
        .map(|possibility| color_guess(guess, possibility.word).weighted_green_yellow_count())
        .sum();
    count / possibilities.len() as f32
}

pub fn best_guesses_by_weighted_green_yellow_count(
    possibilities: &[PossibleAnswer],
    count: usize,
) -> Vec<ScoredGuess> {
    let count = std::cmp::min(count, possibilities.len());
    let mut rankings = Vec::<ScoredGuess>::with_capacity(possibilities.len());

    possibilities
        .par_iter()
        .map(|guess| ScoredGuess {
            word: guess.word,
            score: average_weighted_green_yellow_count(guess.word, possibilities),
        })
        .collect_into_vec(&mut rankings);

    rankings.partial_sort(count, ScoredGuess::cmp_descending_score);
    rankings.truncate(count);
    rankings
}
