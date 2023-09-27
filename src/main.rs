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
use terminal_size::terminal_size;
use word::*;

use colored::Color;
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

struct CmdArgs {
    words: Vec<Word>,
}

fn process_args(args: std::env::Args) -> Result<CmdArgs, String> {
    let mut words = Vec::with_capacity(7);
    for arg in args.skip(1) {
        if let Some(word) = make_word(&arg) {
            words.push(word);
        } else {
            return Err(format!("{arg} is not a single word of five A-Z letters!"));
        }
    }
    if 7 < words.len() {
        return Err(String::from(
            "More than 7 words provided on the command line!",
        ));
    }
    Ok(CmdArgs { words })
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

fn prompt_for_words() -> Vec<Word> {
    const PROMPTS: [&str; 7] = [
        "First guess",
        "Second guess",
        "Third guess",
        "Fourth guess",
        "Fifth guess",
        "Sixth guess",
        "Answer",
    ];

    PROMPTS.into_iter().map_while(prompt_for_word).collect()
}

fn main() -> ExitCode {
    let cmd_args = match process_args(std::env::args()) {
        Err(msg) => {
            println!("{msg}");
            return ExitCode::from(1);
        }
        Ok(result) => result,
    };

    let mut words = cmd_args.words;
    if words.is_empty() {
        words = prompt_for_words();
        if words.is_empty() {
            return ExitCode::SUCCESS;
        }
    }

    let answer = *words.last().unwrap();

    let failed = 6 < words.len();
    if failed {
        words.pop();
    }

    let mut possibilities = all_possible_answers();
    let mut knowledge = WordKnowledge::new();

    let term_width = terminal_size().map_or(80, |(w, _h)| w.0 as usize);
    let column_count = (term_width - LABEL_WIDTH - 2) / (COLUMN_WIDTH + 1);

    println!("\n================= Guess Analysis =================\n");
    for (guess_index, word) in words.iter().enumerate() {
        let guess = color_guess(*word, answer);
        println_label_value(&format!("Guess #{}", guess_index + 1), &guess.formatted());

        if *word == answer {
            println_label_value("Solve State", "Solved");
            break;
        }

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
                for (i, posibility) in possibilities.iter().enumerate() {
                    if i == 0 {
                        print_label(&format!("{} words remain", possibilities.len()));
                    } else if i % column_count == 0 {
                        println!();
                        print_indent();
                    }
                    print!("{} ", knowledge.format_word(posibility.word));
                }
                println!();
            }
        }

        // If there are only two possibilities, there is no sense if ranking
        // them as they are both equally likely.
        if 2 < possibilities.len() {
            let best_guesses = best_guesses(&possibilities, column_count);

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

    if failed {
        println_label_value("Solution", &letters_with_bg(&answer, Color::Green));
    }

    ExitCode::SUCCESS
}
