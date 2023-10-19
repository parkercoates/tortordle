use arrayvec::ArrayVec;
use partial_sort::PartialSort;
use rayon::prelude::*;
use std::cmp::{min, Ordering};
use std::fmt;

use crate::colored_guess::color_guess;
use crate::knowledge::WordKnowledge;
use crate::possibilities::PossibleAnswer;
use crate::slice_subset::SliceSubset;
use crate::word::Word;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Points(i32);

impl Points {
    pub const DECIMAL_PLACES: usize = 1;
    const DENOMINATOR: f64 = 10u32.pow(Self::DECIMAL_PLACES as u32) as f64;
    pub fn zero() -> Self {
        Self(0)
    }
    fn from_f64(f: f64) -> Self {
        Self((Self::DENOMINATOR * f).round() as i32)
    }
    fn from_div(numerator: usize, denominator: usize) -> Self {
        Self::from_f64((numerator as f64) / denominator as f64)
    }
}

impl fmt::Display for Points {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self.0 as f64 / Self::DENOMINATOR).fmt(f)
    }
}

#[derive(PartialEq, Eq)]
pub struct Score {
    pub avg_remaining_guesses: Points,
    pub remaining_words: Points,
    pub weighted_green_yellow_count: Points,
    pub green_yellow_count: Points,
    pub green_count: Points,
}

impl Ord for Score {
    fn cmp(&self, o: &Self) -> Ordering {
        self.avg_remaining_guesses
            .cmp(&o.avg_remaining_guesses)
            .then(self.remaining_words.cmp(&o.remaining_words))
            .then(
                self.weighted_green_yellow_count
                    .cmp(&o.weighted_green_yellow_count)
                    .reverse(),
            )
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
                avg_remaining_guesses: Points::zero(),
                remaining_words: Points::zero(),
                weighted_green_yellow_count: Points::zero(),
                green_yellow_count: Points::zero(),
                green_count: Points::zero(),
            },
        }
    }
}

fn compute_avg_color_counts(scored_guess: &mut ScoredGuess, possibilities: &[PossibleAnswer]) {
    let mut green_count: usize = 0;
    let mut green_yellow_count: usize = 0;
    let mut weighted_green_yellow_count: usize = 0;
    for answer in possibilities {
        let colored_guess = color_guess(scored_guess.word, answer.word);
        green_count += colored_guess.green_count();
        green_yellow_count += colored_guess.green_yellow_count();
        weighted_green_yellow_count += colored_guess.weighted_green_yellow_count();
    }
    scored_guess.score.green_count = Points::from_div(green_count, possibilities.len());
    scored_guess.score.green_yellow_count =
        Points::from_div(green_yellow_count, possibilities.len());
    scored_guess.score.weighted_green_yellow_count =
        Points::from_div(weighted_green_yellow_count, possibilities.len() * 100);
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
    scored_guess.score.remaining_words = Points::from_div(remaining_count, possibilities.len());
}

fn compute_avg_remaining_guesses(
    scored_guess: &mut ScoredGuess,
    possibilities: &SliceSubset<PossibleAnswer>,
) {
    const MAX_GUESSES_TO_TRY: usize = 3;

    fn best_guesses_by_color_counts(
        possibilities: &SliceSubset<PossibleAnswer>,
    ) -> ArrayVec<(Word, usize), MAX_GUESSES_TO_TRY> {
        let score = |p: &PossibleAnswer| {
            possibilities
                .iter()
                .map(|answer| color_guess(p.word, answer.word).weighted_green_yellow_count())
                .sum()
        };

        // We use an ArrayVec to avoid the need to allocate at all
        let mut best_guesses = ArrayVec::<(Word, usize), MAX_GUESSES_TO_TRY>::new();
        let mut it = possibilities.iter();

        // First fill the ArrayVec until full, returning early if we don't have
        // enough guesses.
        for _ in [..MAX_GUESSES_TO_TRY] {
            if let Some(p) = it.next() {
                best_guesses.push((p.word, score(p)));
            } else {
                return best_guesses;
            }
        }

        // Then replace old values with better ones as we find them.
        //
        // Note that we are not sorting the output at all; all of the results
        // will be used and averaged, so we don't care about the order. Just
        // that we get the best MAX_GUESSES_TO_TRY.
        for p in it {
            let score = score(p);
            for best in &mut best_guesses {
                if best.1 < score {
                    *best = (p.word, score);
                    break;
                }
            }
        }
        best_guesses
    }

    fn avg_remaining_guesses(guess: Word, possibilities: &SliceSubset<PossibleAnswer>) -> f64 {
        let mut total_guesses = 0f64;
        for answer in possibilities {
            if answer.word != guess {
                let colored_guess = color_guess(guess, answer.word);
                let knowledge = WordKnowledge::from_guess(&colored_guess);
                let new_possibilities = possibilities.retained(|a| knowledge.matches(a));
                if new_possibilities.len() <= 1 {
                    total_guesses += 1.0;
                } else {
                    let next_guesses = best_guesses_by_color_counts(&new_possibilities);
                    let total_next_guesses: f64 = next_guesses
                        .iter()
                        .map(|(next_guess, _)| {
                            avg_remaining_guesses(*next_guess, &new_possibilities)
                        })
                        .sum();
                    total_guesses += total_next_guesses / next_guesses.len() as f64;
                }
            }
        }
        1.0 + (total_guesses / possibilities.len() as f64)
    }

    scored_guess.score.avg_remaining_guesses =
        Points::from_f64(avg_remaining_guesses(scored_guess.word, possibilities));
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
    const TO_KEEP_FROM_REMAINING_WORDS: usize = 32;

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

    // Keep just the top `TO_KEEP_FROM_REMAINING_WORDS` guesses.
    keep_top_scores(&mut scores, TO_KEEP_FROM_REMAINING_WORDS);

    // For performance reasons, only compute the avg_remaining_guesses if the
    // remaining possibilities fit into a slice subset.
    if let Some(possibilities_subset) = SliceSubset::from_slice(possibilities) {
        scores
            .par_iter_mut()
            .for_each(|scored| compute_avg_remaining_guesses(scored, &possibilities_subset));
    }

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
