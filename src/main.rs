mod colored_guess;
mod data_structures;
mod guess_suggestions;
mod knowledge;
mod possibilities;
mod word;

use colored_guess::color_guess;
use guess_suggestions::{best_guesses, Hundredths, ScoredGuess};
use knowledge::WordKnowledge;
use possibilities::all_possible_answers;
use word::*;

use colored::Color;
use itertools::Itertools;
use std::{io::Write, process::ExitCode};

const LABEL_WIDTH: usize = 22;
const COLUMN_WIDTH: usize = 5;

fn print_indent() {
    print!("{:>LABEL_WIDTH$}  ", "");
}

fn print_label(text: &str) {
    print!("{text:>LABEL_WIDTH$}: ",);
}

fn println_label_value(text: &str, value: &str) {
    println!("{text:>LABEL_WIDTH$}: {value}",);
}

fn println_note(text: &str) {
    print_indent();
    println!("{}", &str_with_fg(text, Color::Magenta),);
}

fn print_number_row(label: &str, guesses: &[ScoredGuess], getter: fn(&ScoredGuess) -> Hundredths) {
    print_label(label);
    for scored in guesses {
        print!("{:>COLUMN_WIDTH$.2} ", getter(scored));
    }
    println!();
}

fn prompt_for_word(prompt: &str) -> Option<Word> {
    let mut input = String::new();
    loop {
        input.clear();

        print_label(prompt);
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

fn main() -> ExitCode {
    const PROMPTS: [&str; 7] = [
        "First guess",
        "Second guess",
        "Third guess",
        "Fourth guess",
        "Fifth guess",
        "Sixth guess",
        "Answer",
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

    let mut possibilities = all_possible_answers();

    println!("\n================= Guess Analysis =================\n");
    let mut knowledge = WordKnowledge::new();
    for word in &words {
        let guess = color_guess(*word, answer);
        println_label_value("Guess", &guess.formatted());

        if !knowledge.matches_word(*word) {
            println_note("This guess conflicted with previously collected information!");
        } else if !possibilities.iter().any(|a| a.word == *word) {
            println_note("This guess was not in the list of remaining possibilities!");
        }

        knowledge.add_guess(&guess);
        println_label_value("Solve State", &knowledge.formatted());

        possibilities.retain(|p| knowledge.matches(p));
        match possibilities.len() {
            0 => println_note("There are no words remaining! Something went wrong!"),
            1 => println_label_value(
                "1 word remains",
                &knowledge.format_word(possibilities[0].word),
            ),
            _ => {
                let words_string = &possibilities
                    .iter()
                    .map(|a| knowledge.format_word(a.word))
                    .join(" ");

                let label_text = format!("{} words remain", possibilities.len());
                let label = format!("{:>LABEL_WIDTH$}: ", label_text);
                let indent = " ".repeat(label.len());
                let options = textwrap::Options::with_termwidth()
                    .initial_indent(&label)
                    .subsequent_indent(&indent);
                for line in textwrap::wrap(words_string, options) {
                    println!("{line}");
                }
            }
        }

        // If there are only two possibilities, there is no sense if ranking
        // them as they are both equally likely.
        if 2 < possibilities.len() {
            let suggestions_to_show =
                (textwrap::termwidth() - LABEL_WIDTH - 2) / (COLUMN_WIDTH + 1);
            let best_guesses = best_guesses(&possibilities, suggestions_to_show);

            println!();

            print_indent();
            for scored in &best_guesses {
                print!("{:^COLUMN_WIDTH$} ", scored.rank);
            }
            println!();

            print_label("Suggested Guesses");
            for scored in &best_guesses {
                // We can't use COLUMN_WIDTH here because the formatting codes
                // throw off the justification counts.
                print!("{} ", knowledge.format_word(scored.word));
            }
            println!();

            print_number_row("Avg Remaining Words", &best_guesses, |s| {
                s.score.remaining_words
            });

            print_number_row("Avg Green/Yellow Count", &best_guesses, |s| {
                s.score.green_yellow_count
            });

            print_number_row("Avg Green Count", &best_guesses, |s| s.score.green_count);
        }

        println!();
    }

    println_label_value("Solution", &letters_with_bg(&answer, Color::Green));

    ExitCode::SUCCESS
}
