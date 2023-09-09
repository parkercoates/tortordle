mod colored_guess;
mod data_structures;
mod knowledge;
mod word;

use colored_guess::*;
use data_structures::LetterHistogram;
use knowledge::WordKnowledge;
use word::*;

use colored::Color;
use itertools::Itertools;
use partial_sort::PartialSort;
use rayon::prelude::*;
use std::{cmp::Ordering, io::Write, process::ExitCode};

const fn slice_as_array_ref<T, const N: usize>(slice: &[T]) -> &[T; N] {
    assert!(N <= slice.len());
    unsafe { &*slice.as_ptr().cast::<[T; N]>() }
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
