use arrayvec::ArrayVec;
use itertools::Itertools;
use partial_sort::PartialSort;
use rayon::prelude::*;
use std::cmp::{Ordering, min};
use std::fmt;

use crate::guess_coloring::ColorCounts;
use crate::knowledge::WordKnowledge;
use crate::possibilities::PossibleAnswer;
use crate::slice_subset::SliceSubset;
use crate::word::Word;

fn color_counts<'a, I: IntoIterator<Item = &'a PossibleAnswer>>(
    word: Word,
    possibilities: I,
) -> ColorCounts {
    let mut totals = ColorCounts::default();
    for possibility in possibilities {
        totals += ColorCounts::from_guess(word, possibility.word);
    }
    totals
}

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
    #[must_use]
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }
}

impl fmt::Display for Points {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (f64::from(self.0) / Self::DENOMINATOR).fmt(f)
    }
}

impl std::ops::Neg for Points {
    type Output = Points;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

pub struct ScoredGuess {
    pub word: Word,

    pub green_count: Points,
    pub green_yellow_count: Points,
    pub remaining_words: Points,
    pub avg_score: Points,

    pub rank: usize,

    score: isize,
}

impl ScoredGuess {
    fn init_with_word(word: Word) -> Self {
        Self {
            word,
            green_count: Points::zero(),
            green_yellow_count: Points::zero(),
            remaining_words: Points::zero(),
            avg_score: Points::zero(),
            score: 100,
            rank: 0,
        }
    }

    fn compute_avg_color_counts(&mut self, possibilities: &[PossibleAnswer]) {
        let counts = color_counts(self.word, possibilities);
        self.green_count = Points::from_div(counts.green_count(), possibilities.len());
        self.green_yellow_count =
            Points::from_div(counts.green_yellow_count(), possibilities.len());
        self.score -=
            (counts.weighted_count() as f64 / (10.0 * possibilities.len() as f64)).round() as isize;
    }

    fn compute_avg_remaining_words(&mut self, possibilities: &[PossibleAnswer]) {
        let mut remaining_count: usize = 0;
        for answer in possibilities {
            if answer.word != self.word {
                let knowledge = WordKnowledge::from_guess(self.word, answer.word);
                remaining_count += possibilities
                    .iter()
                    .filter(|pos| knowledge.matches(pos))
                    .count();
            }
        }
        self.remaining_words = Points::from_div(remaining_count, possibilities.len());
        self.score += self.remaining_words.0 as isize * 1_000;
    }

    fn compute_avg_score(
        &mut self,
        possibilities: &SliceSubset<PossibleAnswer>,
        guesses_so_far: usize,
    ) {
        const MAX_GUESSES_TO_TRY: usize = 3;

        fn best_guesses_by_color_counts(
            possibilities: &SliceSubset<PossibleAnswer>,
        ) -> ArrayVec<Word, MAX_GUESSES_TO_TRY> {
            // Note that we are not sorting the output at all; all of the results will be used and
            // averaged together, so we don't care about the order.

            // We use an ArrayVec to avoid the need to allocate at all.
            let mut words = ArrayVec::<Word, MAX_GUESSES_TO_TRY>::new();

            let mut it = possibilities.iter().map(|p| p.word);

            // If we don't have an excess of words, we don't need to score and rank them at all. We
            // can just copy them all into the ArrayVec and return.
            if it.len() <= MAX_GUESSES_TO_TRY {
                words.extend(it);
                return words;
            }

            // Otherwise, fill the ArrayVec until full.
            for _ in 0..MAX_GUESSES_TO_TRY {
                if let Some(w) = it.next() {
                    words.push(w);
                }
            }

            // Build a parallel array of scores.
            let score = |w: Word| color_counts(w, possibilities).weighted_count();
            let mut scores = words
                .as_mut_array::<MAX_GUESSES_TO_TRY>()
                .expect("Words has to be MAX_GUESSES_TO_TRY in size.")
                .map(score);

            let find_worst_score = |scores: &[usize; MAX_GUESSES_TO_TRY]| {
                let index = scores.iter().position_min().unwrap();
                (index, scores[index])
            };

            // Loop through the remaining possibilities, scoring them. If a new score is better
            // than our worst, replace our worst with it and calculate the new worst.
            let (mut worst_index, mut worst_score) = find_worst_score(&scores);
            for w in it {
                let s = score(w);
                if s > worst_score {
                    words[worst_index] = w;
                    scores[worst_index] = s;
                    (worst_index, worst_score) = find_worst_score(&scores);
                }
            }

            words
        }

        fn avg_remaining_guesses(guess: Word, possibilities: &SliceSubset<PossibleAnswer>) -> f64 {
            let mut total_guesses = 0f64;
            for answer in possibilities {
                if answer.word != guess {
                    let knowledge = WordKnowledge::from_guess(guess, answer.word);
                    let new_possibilities = possibilities.retained(|a| knowledge.matches(a));
                    if new_possibilities.len() <= 1 {
                        total_guesses += 1.0;
                    } else {
                        let next_guesses = best_guesses_by_color_counts(&new_possibilities);
                        let total_next_guesses = next_guesses
                            .iter()
                            .map(|w| avg_remaining_guesses(*w, &new_possibilities))
                            .sum::<f64>();
                        total_guesses += total_next_guesses / next_guesses.len() as f64;
                    }
                }
            }
            1.0 + (total_guesses / possibilities.len() as f64)
        }

        let avg_guesses = avg_remaining_guesses(self.word, possibilities);
        self.avg_score = Points::from_f64(avg_guesses + guesses_so_far as f64);
        self.score += self.avg_score.0 as isize * 1_000_000;
    }
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
    keep_top(scores, count, |a, b| a.score.cmp(&b.score));
}

fn keep_top_scores_final(guesses: &mut Vec<ScoredGuess>, count: usize, user_guess: Option<Word>) {
    // In the case of an N-way tie on score, the order of the guesses is thus far non-deterministic,
    // but we'd prefer it be deterministic, so let's apply a super thorough sorting for the final
    // sort. This is always the smallest sort we do, so performance is not critical.
    //
    // If a user guess was provided, order it first in the event of a tie. Users like to see that
    // blue column as close to the left edge as possible, and in the event of a tie, I see no reason
    // not to indulge them.
    //
    // Otherwise, order by green/yellow count, green count and then alphabetically. Putting
    // green/yellow count before green count is entirely arbitrary, but matches the row order shown
    // to users.
    //
    // Note that these extra layers of sorting are only for ordering purposes. Suggestion ranking is
    // is still entirely based on score, so these do not reduce the number of ties.
    keep_top(guesses, count, |a, b| {
        a.score
            .cmp(&b.score)
            .then_with(|| match user_guess {
                Some(guess) if guess == a.word => Ordering::Less,
                Some(guess) if guess == b.word => Ordering::Greater,
                _ => Ordering::Equal,
            })
            .then_with(|| b.green_yellow_count.cmp(&a.green_yellow_count))
            .then_with(|| b.green_count.cmp(&a.green_count))
            .then_with(|| a.word.cmp(&b.word))
    });
}

pub fn best_guesses(
    possibilities: &[PossibleAnswer],
    count: usize,
    guesses_so_far: usize,
    user_guess: Option<Word>,
) -> Vec<ScoredGuess> {
    const TO_KEEP_FROM_COLOR_COUNT: usize = 100;
    const TO_KEEP_FROM_REMAINING_WORDS: usize = 32;

    // Turn all possibilities into ScoredGuesses, but only compute the average
    // green and yellow counts first as this can be done very quickly and is a
    // rough, but good indicator of the quality of a guess.
    let mut guesses: Vec<_> = possibilities
        .par_iter()
        .map(|p| {
            let mut g = ScoredGuess::init_with_word(p.word);
            g.compute_avg_color_counts(possibilities);
            g
        })
        .collect();

    // Keep just the top `TO_KEEP_FROM_COLOR_COUNT` guesses.
    keep_top_scores(&mut guesses, TO_KEEP_FROM_COLOR_COUNT);

    // Calculate the average remaining word count for the remaining guesses.
    // This is a considerably more expensive calculation.
    guesses
        .par_iter_mut()
        .for_each(|guess| guess.compute_avg_remaining_words(possibilities));

    // Keep just the top `TO_KEEP_FROM_REMAINING_WORDS` guesses.
    keep_top_scores(&mut guesses, TO_KEEP_FROM_REMAINING_WORDS);

    // For performance reasons, only compute the avg_score if the remaining possibilities fit into a
    // slice subset.
    if let Some(subset) = SliceSubset::from_slice(possibilities) {
        guesses
            .par_iter_mut()
            .for_each(|guess| guess.compute_avg_score(&subset, guesses_so_far));
    }

    // Keep just the top `count` guesses and make ordering deterministic.
    keep_top_scores_final(&mut guesses, count, user_guess);

    // Assign ranks to the guesses, detecting any ties.
    compute_ranks(&mut guesses);

    guesses
}

pub fn top_guess(possibilities: &[PossibleAnswer]) -> Option<Word> {
    best_guesses(possibilities, 1, 0, None)
        .first()
        .map(|guess| guess.word)
}
