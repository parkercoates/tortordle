mod data_structures;
mod word;

use data_structures::{LetterHistogram, LetterSet};
use word::*;

use colored::Color;
use itertools::Itertools;
use partial_sort::PartialSort;
use rayon::prelude::*;
use std::{cmp::Ordering, io::Write, iter::zip, process::ExitCode};

const fn slice_as_array_ref<T, const N: usize>(slice: &[T]) -> &[T; N] {
    assert!(N <= slice.len());
    unsafe { &*slice.as_ptr().cast::<[T; N]>() }
}

#[derive(Clone, Copy, PartialEq)]
enum LetterState {
    Black,
    Yellow,
    Green,
}

impl LetterState {
    const fn color(self) -> Color {
        match self {
            Self::Black => Color::Black,
            Self::Yellow => Color::Yellow,
            Self::Green => Color::Green,
        }
    }
}

struct ColoredWord {
    slots: [(Letter, LetterState); WORD_LENGTH],
}

impl ColoredWord {
    fn iter(&self) -> std::slice::Iter<(Letter, LetterState)> {
        self.slots.iter()
    }

    // fn word(&self) -> Word {
    //     [
    //         self.slots[0].0,
    //         self.slots[1].0,
    //         self.slots[2].0,
    //         self.slots[3].0,
    //         self.slots[4].0,
    //     ]
    // }

    fn formatted(&self) -> String {
        self.iter()
            .map(|(letter, state)| letter_with_bg(*letter, state.color()))
            .join("")
    }

    fn green_count(&self) -> usize {
        self.iter()
            .filter(|(_, state)| *state == LetterState::Green)
            .count()
    }

    fn weighted_green_yellow_count(&self) -> f32 {
        self.iter()
            .map(|(_, state)| match state {
                // This dumb weighting does not attempt to assign relative
                // values to greens and yellows. It just ensures that if the
                // number of greens+yellows is the same for two guesses, the
                // guess with more greens will score higher.
                LetterState::Green => 1.10,
                LetterState::Yellow => 0.90,
                LetterState::Black => 0.0,
            })
            .sum()
    }
}

fn color_guess(guess: Word, answer: Word) -> ColoredWord {
    let mut slots = [(b' ', LetterState::Black); WORD_LENGTH];
    let mut yellows = LetterHistogram::new();
    for (guess_letter, answer_letter, (letter, state)) in
        itertools::izip!(guess, answer, &mut slots)
    {
        *letter = guess_letter;
        if guess_letter == answer_letter {
            *state = LetterState::Green;
        } else {
            yellows.add_letter(answer_letter);
        }
    }
    for (letter, state) in &mut slots {
        if *state == LetterState::Black && yellows.contains(*letter) {
            *state = LetterState::Yellow;
            yellows.remove_letter(*letter);
        }
    }
    ColoredWord { slots }
}

#[derive(Clone, Copy)]
enum LetterKnowledge {
    Is(Letter),
    IsNot(LetterSet),
}

impl LetterKnowledge {
    fn formatted(self, yellows: LetterHistogram) -> String {
        match self {
            Self::Is(letter) => letter_with_fg(letter, Color::Green),
            Self::IsNot(set) => {
                letters_with_fg(
                    yellows.letters().dedup().filter(|&l| !set.contains(*l)),
                    Color::Yellow,
                ) + &letters_with_fg(&set.letters(), Color::Red)
            }
        }
    }

    const fn matches(self, letter: Letter) -> bool {
        match self {
            Self::Is(known_letter) => letter == known_letter,
            Self::IsNot(set) => !set.contains(letter),
        }
    }
}

struct WordKnowledge {
    slots: [LetterKnowledge; WORD_LENGTH],
    histogram: LetterHistogram,
    yellows: LetterHistogram,
}

impl WordKnowledge {
    const fn new() -> Self {
        Self {
            slots: [LetterKnowledge::IsNot(LetterSet::new()); 5],
            histogram: LetterHistogram::new(),
            yellows: LetterHistogram::new(),
        }
    }

    fn from_guess(guess: &ColoredWord) -> Self {
        let mut result = Self::new();
        result.add_guess(guess);
        //
        // for (i, (letter, color)) in guess.iter().enumerate() {
        //     match color {
        //         LetterState::Black => {
        //             // If we've already seen a particular letter in yellow in this
        //             // guess, seeing it in black only tells us that that this
        //             // specific slot can't be that letter.
        //             if result.yellows.contains(*letter) {
        //                 if let LetterKnowledge::IsNot(set) = &mut result.slots[i] {
        //                     set.insert(*letter);
        //                 }
        //             // Otherwise, we know that letter occurs in no slot.
        //             } else {
        //                 for slot in &mut result.slots {
        //                     if let LetterKnowledge::IsNot(set) = slot {
        //                         set.insert(*letter);
        //                     }
        //                 }
        //             }
        //         }
        //         LetterState::Yellow => {
        //             result.histogram.add_letter(*letter);
        //             result.yellows.add_letter(*letter);
        //             if let LetterKnowledge::IsNot(set) = &mut result.slots[i] {
        //                 set.insert(*letter);
        //             }
        //         }
        //         LetterState::Green => {
        //             result.histogram.add_letter(*letter);
        //             result.slots[i] = LetterKnowledge::Is(*letter);
        //         }
        //     }
        // }

        // for (count, &letter) in result.yellows.clone().letters().dedup_with_count() {
        //     let matches = result
        //         .slots
        //         .iter()
        //         .filter(|slot| slot.matches(letter))
        //         .count();
        //     if matches == count {
        //         for slot in &mut result.slots {
        //             if let LetterKnowledge::IsNot(_) = slot {
        //                 if slot.matches(letter) {
        //                     *slot = LetterKnowledge::Is(letter);
        //                     result.yellows.remove_letter(letter);
        //                 }
        //             }
        //         }
        //     }
        // }
        //
        result
    }

    fn add_guess(&mut self, guess: &ColoredWord) {
        let mut new_histogram = LetterHistogram::new();
        let mut new_yellows = LetterHistogram::new();
        for (i, (letter, color)) in guess.iter().enumerate() {
            match color {
                LetterState::Black => {
                    // If we've already seen a particular letter in yellow in this
                    // guess, seeing it in black only tells us that that this
                    // specific slot can't be that letter.
                    if new_yellows.contains(*letter) {
                        if let LetterKnowledge::IsNot(set) = &mut self.slots[i] {
                            set.insert(*letter);
                        }
                    // Otherwise, we know that letter occurs in no slot.
                    } else {
                        for slot in &mut self.slots {
                            if let LetterKnowledge::IsNot(set) = slot {
                                set.insert(*letter);
                            }
                        }
                    }
                }
                LetterState::Yellow => {
                    new_histogram.add_letter(*letter);
                    new_yellows.add_letter(*letter);
                    if let LetterKnowledge::IsNot(set) = &mut self.slots[i] {
                        set.insert(*letter);
                    }
                }
                LetterState::Green => {
                    new_histogram.add_letter(*letter);
                    self.slots[i] = LetterKnowledge::Is(*letter);
                }
            }
        }

        self.histogram.merge_via_max(new_histogram);

        self.yellows = self.histogram;
        for slot in &mut self.slots {
            if let LetterKnowledge::Is(letter) = slot {
                self.yellows.remove_letter(*letter);
            }
        }

        for (count, &letter) in self.yellows.clone().letters().dedup_with_count() {
            let matches = self
                .slots
                .iter()
                .filter(|slot| slot.matches(letter))
                .count();
            if matches == count {
                for slot in &mut self.slots {
                    if let LetterKnowledge::IsNot(_) = slot {
                        if slot.matches(letter) {
                            *slot = LetterKnowledge::Is(letter);
                            self.yellows.remove_letter(letter);
                        }
                    }
                }
            }
        }
    }

    fn matches(&self, possibility: &PossibleAnswer) -> bool {
        let slots_match = zip(&self.slots, possibility.word).all(|(s, l)| s.matches(l));
        let needs_match = possibility.histogram.contains_other(self.histogram);
        slots_match && needs_match
    }

    fn matches_word(&self, word: Word) -> bool {
        self.matches(&PossibleAnswer::from_word(word))
    }

    fn formatted(&self) -> String {
        format!(
            "[{}]",
            self.slots
                .iter()
                .map(|slot| slot.formatted(self.yellows))
                .join("|")
        )
    }

    fn format_possibility(&self, possibility: &PossibleAnswer) -> String {
        std::iter::zip(possibility.word, &self.slots)
            .map(|(letter, slot)| match slot {
                LetterKnowledge::Is(known_letter) if letter == *known_letter => {
                    letter_with_fg(letter, Color::Green)
                }
                _ if self.yellows.contains(letter) => letter_with_fg(letter, Color::Yellow),
                _ => letter_to_string(letter),
            })
            .join("")
    }
}

struct PossibleAnswer {
    word: Word,
    histogram: LetterHistogram,
}

impl PossibleAnswer {
    fn from_word(word: Word) -> Self {
        Self {
            word,
            histogram: LetterHistogram::from_word(word),
        }
    }
}

#[derive(PartialEq, PartialOrd)]
struct ScoredWord {
    word: Word,
    score: f32,
}

impl ScoredWord {
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

fn best_guesses_by_remaining_possibilities(
    possibilities: &[PossibleAnswer],
    count: usize,
) -> Vec<ScoredWord> {
    let count = std::cmp::min(count, possibilities.len());
    let mut rankings = Vec::<ScoredWord>::with_capacity(possibilities.len());

    possibilities
        .par_iter()
        .map(|guess| ScoredWord {
            word: guess.word,
            score: average_remaining_possibilites(guess.word, possibilities),
        })
        .collect_into_vec(&mut rankings);

    rankings.partial_sort(count, ScoredWord::cmp_ascending_score);
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

fn best_guesses_by_green_count(possibilities: &[PossibleAnswer], count: usize) -> Vec<ScoredWord> {
    let count = std::cmp::min(count, possibilities.len());
    let mut rankings = Vec::<ScoredWord>::with_capacity(possibilities.len());

    possibilities
        .par_iter()
        .map(|guess| ScoredWord {
            word: guess.word,
            score: average_green_count(guess.word, possibilities),
        })
        .collect_into_vec(&mut rankings);

    rankings.partial_sort(count, ScoredWord::cmp_descending_score);
    rankings.truncate(count);
    rankings
}

fn average_weighted_green_yellow_count(guess: Word, possibilities: &[PossibleAnswer]) -> f32 {
    let count: f32 = possibilities
        .iter()
        .map(|possibility| color_guess(guess, possibility.word).weighted_green_yellow_count())
        .sum();
    count / possibilities.len() as f32
}

fn best_guesses_by_weighted_green_yellow_count(
    possibilities: &[PossibleAnswer],
    count: usize,
) -> Vec<ScoredWord> {
    let count = std::cmp::min(count, possibilities.len());
    let mut rankings = Vec::<ScoredWord>::with_capacity(possibilities.len());

    possibilities
        .par_iter()
        .map(|guess| ScoredWord {
            word: guess.word,
            score: average_weighted_green_yellow_count(guess.word, possibilities),
        })
        .collect_into_vec(&mut rankings);

    rankings.partial_sort(count, ScoredWord::cmp_descending_score);
    rankings.truncate(count);
    rankings
}

fn prompt_for_word(prompt: &str) -> Option<Word> {
    let mut input = String::new();
    loop {
        input.clear();

        print!("{prompt}");
        std::io::stdout().flush().unwrap();

        if std::io::stdin().read_line(&mut input).is_err() {
            println!("Failed to read line!");
            continue;
        }

        let input = input.trim();

        if input.is_empty() {
            return None;
        }

        let word = make_word(input);
        if word.is_none() {
            println!("{input} is not a single word of five A-Z letters!");
            continue;
        }

        return word;
    }
}

static ANSWERS: [Letter; 13890] = *std::include_bytes!("WORDLE-ANSWERS.txt");

fn main() -> ExitCode {
    const PROMPTS: [&str; 7] = [
        "  First guess: ",
        " Second guess: ",
        "  Third guess: ",
        " Fourth guess: ",
        "  Fifth guess: ",
        "  Sixth guess: ",
        "       Answer: ",
    ];

    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut words: Vec<Word> = Vec::new();
    if args.is_empty() {
        for prompt in PROMPTS {
            if let Some(word) = prompt_for_word(prompt) {
                words.push(word);
            } else {
                break;
            }
        }
    } else {
        for arg in args {
            if let Some(word) = make_word(&arg) {
                words.push(word);
            } else {
                println!("{arg} is not a single word of five A-Z letters!");
                return ExitCode::from(1);
            }
        }
    }
    if words.is_empty() {
        return ExitCode::SUCCESS;
    }

    let answer = words.pop().unwrap();

    let mut possibilities: Vec<PossibleAnswer> = ANSWERS
        .chunks_exact(WORD_LENGTH + 1) // Five letters then a newline
        .map(|chunk| {
            let word: Word = *slice_as_array_ref(chunk);
            assert!(word.iter().all(u8::is_ascii_uppercase));
            PossibleAnswer::from_word(word)
        })
        .collect();

    // println!("Starting words that lead to the fewest remaining words on average:");
    // for (count, word) in find_best_of_possibilities(&possibilities, 20) {
    //     println!(
    //         "{}: {:.2}",
    //         letters_to_string(&word),
    //         count as f32 / possibilities.len() as f32
    //     );
    // }
    // return ExitCode::SUCCESS;

    println!("\nGuess Analysis:");
    let mut knowledge = WordKnowledge::new();
    for word in &words {
        let was_mistake = !knowledge.matches_word(*word);

        let guess = color_guess(*word, answer);
        knowledge.add_guess(&guess);
        println!(
            "   {}   Solve state: {}",
            guess.formatted(),
            knowledge.formatted()
        );

        if was_mistake {
            println!(
                "           {}",
                str_with_fg(
                    "NOTE! This guess conflicted with previously collected information!",
                    Color::Magenta
                )
            );
        } else if !possibilities.iter().any(|a| a.word == *word) {
            println!(
                "           {}",
                str_with_fg(
                    "NOTE! This guess was not in the list of remaining possibilities!",
                    Color::Magenta
                )
            );
        }

        possibilities.retain(|p| knowledge.matches(p));
        match possibilities.len() {
            0 => println!(
                "           {}",
                str_with_fg(
                    "NOTE! There are no words remaining! Something went wrong!",
                    Color::Magenta
                )
            ),
            1 => println!(
                "           1 word remains: {}",
                knowledge.format_possibility(&possibilities[0])
            ),
            _ => {
                let words_string = &possibilities
                    .iter()
                    .map(|a| knowledge.format_possibility(a))
                    .join(" ");

                let label = format!("           {} words remain: ", possibilities.len());
                let indent = " ".repeat(label.len());
                let options = textwrap::Options::with_termwidth()
                    .initial_indent(&label)
                    .subsequent_indent(&indent);
                for line in textwrap::wrap(words_string, options) {
                    println!("{line}");
                }
            }
        }

        if possibilities.len() > 2 {
            const SUGGESTIONS_TO_SHOW: usize = 4;
            const COLON_COLUMN: usize = 40;
            println!("\n           Suggested Guesses:");

            print!("{:>COLON_COLUMN$}: ", "Highest green count");
            for ScoredWord { score, word } in
                best_guesses_by_green_count(&possibilities, SUGGESTIONS_TO_SHOW)
            {
                print!("{}={score:.2}  ", letters_to_string(&word));
            }
            println!();

            print!("{:>COLON_COLUMN$}: ", "Highest green/yellow count");
            for ScoredWord { score, word } in
                best_guesses_by_weighted_green_yellow_count(&possibilities, SUGGESTIONS_TO_SHOW)
            {
                print!("{}={score:.2}  ", letters_to_string(&word));
            }
            println!();

            print!("{:>COLON_COLUMN$}: ", "Fewest remaining words");
            for ScoredWord { score, word } in
                best_guesses_by_remaining_possibilities(&possibilities, SUGGESTIONS_TO_SHOW)
            {
                print!("{}={score:.2}  ", letters_to_string(&word));
            }
            println!();
        }

        println!();
    }
    println!("   {}   Solution", letters_with_bg(&answer, Color::Green));
    ExitCode::SUCCESS
}
