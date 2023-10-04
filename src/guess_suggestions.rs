use partial_sort::PartialSort;
use rayon::prelude::*;
use std::cmp::{min, Ordering};
use std::fmt;

use crate::colored_guess::color_guess;
use crate::knowledge::WordKnowledge;
use crate::possibilities::PossibleAnswer;
use crate::word::Word;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hundredths(i32);

impl Hundredths {
    fn zero() -> Self {
        Self(0)
    }
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
pub struct Score {
    pub remaining_words: Hundredths,
    pub green_yellow_count: Hundredths,
    pub green_count: Hundredths,
}

impl Ord for Score {
    fn cmp(&self, o: &Self) -> Ordering {
        self.remaining_words
            .cmp(&o.remaining_words)
            .then(self.green_yellow_count.cmp(&o.green_yellow_count).reverse())
            .then(self.green_count.cmp(&o.green_count).reverse())
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct ScoredGuess {
    pub score: Score,
    pub rank: usize,
    pub word: Word,
}

impl ScoredGuess {
    fn init_with_word(word: Word) -> Self {
        Self {
            word,
            rank: 0,
            score: Score {
                remaining_words: Hundredths::zero(),
                green_yellow_count: Hundredths::zero(),
                green_count: Hundredths::zero(),
            },
        }
    }
}

fn compute_avg_color_counts(scored_guess: &mut ScoredGuess, possibilities: &[PossibleAnswer]) {
    let mut green_count: usize = 0;
    let mut green_yellow_count: usize = 0;
    for answer in possibilities {
        let colored_guess = color_guess(scored_guess.word, answer.word);
        green_count += colored_guess.green_count();
        green_yellow_count += colored_guess.green_yellow_count();
    }
    scored_guess.score.green_count = Hundredths::from_div(green_count, possibilities.len());
    scored_guess.score.green_yellow_count =
        Hundredths::from_div(green_yellow_count, possibilities.len());
}

fn compute_avg_remaining_words(scored_guess: &mut ScoredGuess, possibilities: &[PossibleAnswer]) {
    let mut remaining_count: usize = 0;
    for answer in possibilities {
        if answer.word != scored_guess.word {
            let colored_guess = color_guess(scored_guess.word, answer.word);
            let knowledge = WordKnowledge::from_guess(&colored_guess);
            remaining_count += possibilities
                .iter()
                .filter(|pos| knowledge.matches(pos))
                .count();
        }
    }
    scored_guess.score.remaining_words = Hundredths::from_div(remaining_count, possibilities.len());
}

fn compute_ranks(guesses: &mut [ScoredGuess]) {
    let mut it = guesses.iter_mut();
    if let Some(first) = it.next() {
        let mut count = 1usize;
        first.rank = count;
        let mut previous = first;

        for v in it {
            count += 1;
            if v.score == previous.score {
                v.rank = previous.rank;
            } else {
                v.rank = count;
                previous = v;
            }
        }
    }
}

fn keep_top<T, F>(v: &mut Vec<T>, count: usize, cmp: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    let count = min(count, v.len());
    v.partial_sort(count, cmp);
    v.truncate(count);
}

fn keep_top_scores(scores: &mut Vec<ScoredGuess>, count: usize) {
    keep_top(scores, count, ScoredGuess::cmp);
}

pub fn best_guesses(possibilities: &[PossibleAnswer], count: usize) -> Vec<ScoredGuess> {
    const TO_KEEP_FROM_COLOR_COUNT: usize = 100;

    // Turn all possibilities into ScoredGuesses, but don't compute any scores
    // yet.
    let mut scores: Vec<ScoredGuess> = possibilities
        .iter()
        .map(|p| ScoredGuess::init_with_word(p.word))
        .collect();

    // Compute the average green and yellow counts first as this can be done
    // very quickly and is a good rough indicator of the quality of a guess.
    scores
        .par_iter_mut()
        .for_each(|scored| compute_avg_color_counts(scored, possibilities));

    // Keep just the top `TO_KEEP_FROM_COLOR_COUNT` guesses.
    keep_top_scores(&mut scores, TO_KEEP_FROM_COLOR_COUNT);

    // Calculate the average remaining word count for the remaining guesses.
    // This is a considerably more expensive calculation.
    scores
        .par_iter_mut()
        .for_each(|scored| compute_avg_remaining_words(scored, possibilities));

    // Keep just the top `count` guesses.
    keep_top_scores(&mut scores, count);

    // Assign ranks to the guesses, detecting any ties.
    compute_ranks(&mut scores);

    scores
}

fn top_guesses(possibilities: &[PossibleAnswer]) -> Vec<ScoredGuess> {
    const WIDEST_TIE_TO_WORRY_ABOUT: usize = 10;
    let mut guesses = best_guesses(possibilities, WIDEST_TIE_TO_WORRY_ABOUT);
    guesses.retain(|g| g.rank == 1);
    guesses
}

pub fn rand_top_guess(possibilities: &[PossibleAnswer]) -> Option<Word> {
    fastrand::choice(top_guesses(possibilities)).map(|g| g.word)
}
