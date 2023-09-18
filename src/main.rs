mod colored_guess;
mod data_structures;
mod guess_suggestions;
mod knowledge;
mod possibilities;
mod word;

use colored_guess::color_guess;
use guess_suggestions::best_guesses;
use knowledge::WordKnowledge;
use possibilities::all_possible_answers;
use word::*;

use colored::Color;
use itertools::Itertools;
use std::{io::Write, process::ExitCode};

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

    let mut possibilities = all_possible_answers();

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
                knowledge.format_word(possibilities[0].word)
            ),
            _ => {
                let words_string = &possibilities
                    .iter()
                    .map(|a| knowledge.format_word(a.word))
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

        // If there are only two possibilities, there is no sense if ranking
        // them as they are both equally likely.
        if 2 < possibilities.len() {
            const LABEL_WIDTH: usize = 22;
            const COLUMN_WIDTH: usize = 6;
            let suggestions_to_show = (textwrap::termwidth() - LABEL_WIDTH) / COLUMN_WIDTH;
            let best_guesses = best_guesses(&possibilities, suggestions_to_show);

            print!("\n{:>LABEL_WIDTH$}: ", "Suggested Guesses");
            for scored in &best_guesses {
                print!("{} ", knowledge.format_word(scored.word));
            }
            print!("\n   Avg Remaining Words:");
            for scored in &best_guesses {
                print!("{:>COLUMN_WIDTH$.2}", scored.score.remaining_words);
            }
            print!("\nAvg Green/Yellow Count:");
            for scored in &best_guesses {
                print!("{:>COLUMN_WIDTH$.2}", scored.score.green_yellow_count);
            }
            print!("\n       Avg Green Count:");
            for scored in &best_guesses {
                print!("{:>COLUMN_WIDTH$.2}", scored.score.green_count);
            }
            println!();
        }

        println!();
    }
    println!("   {}   Solution", letters_with_bg(&answer, Color::Green));
    ExitCode::SUCCESS
}
