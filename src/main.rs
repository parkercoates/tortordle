mod colored_guess;
mod data_structures;
mod guess_suggestions;
mod knowledge;
mod possibilities;
mod word;

use colored_guess::color_guess;
use guess_suggestions::{best_guesses, Hundredths, ScoredGuess};
use knowledge::WordKnowledge;
use possibilities::{all_possible_answers, PossibleAnswer};
use terminal_size::terminal_size;
use word::*;

use colored::Color;
use std::{io::Write, process::ExitCode};

const MAX_GUESSES: usize = 6;
const MAX_WORDS: usize = MAX_GUESSES + 1;

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

struct CmdArgs {
    suggest_first_guess: bool,
    words: Vec<Word>,
}

fn process_args(args: std::env::Args) -> Result<CmdArgs, String> {
    let mut suggest_first_guess = false;
    let mut words = Vec::with_capacity(MAX_WORDS);
    for arg in args.skip(1) {
        if arg.starts_with('-') {
            match arg.as_str() {
                "--suggest-first-guess" => suggest_first_guess = true,
                _ => return Err(format!("Unrecognized flag: {arg}")),
            }
        } else if let Some(word) = make_word(&arg) {
            words.push(word);
        } else {
            return Err(format!("{arg} is not a single word of five A-Z letters!"));
        }
    }
    if MAX_WORDS < words.len() {
        return Err(format!("More than {MAX_WORDS} word arguments provided!"));
    }
    Ok(CmdArgs {
        suggest_first_guess,
        words,
    })
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
    const PROMPTS: [&str; MAX_WORDS] = [
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

fn print_remaining_words(
    possibilities: &[PossibleAnswer],
    knowledge: &WordKnowledge,
    columns: usize,
) {
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
                } else if i % columns == 0 {
                    println!();
                    print_indent();
                }
                print!("{} ", knowledge.format_word(posibility.word));
            }
            println!();
        }
    }
}

fn print_suggestions(suggestions: &[ScoredGuess], knowledge: &WordKnowledge) {
    // If there are two suggestions or less, they are all equally good. If the
    // last suggestion's rank is 1, they are all tied and therefore equally
    // good. Regardless, let's not bother showing any suggested guesses.
    let all_equally_good = suggestions.len() <= 2 || suggestions.last().unwrap().rank == 1;
    if !all_equally_good {
        print_indent();
        for suggestion in suggestions {
            print!("{:^COLUMN_WIDTH$} ", suggestion.rank);
        }
        println!();

        print_label("Suggested Guesses");
        for suggestion in suggestions {
            // We can't use COLUMN_WIDTH here because the formatting codes
            // throw off the justification counts.
            print!("{} ", knowledge.format_word(suggestion.word));
        }
        println!();

        let print_numbers = |label, getter: fn(&ScoredGuess) -> Hundredths| {
            print_label(label);
            for suggestion in suggestions {
                print!("{:>COLUMN_WIDTH$.2} ", getter(suggestion));
            }
            println!();
        };

        print_numbers("Avg Remaining Words", |s| s.score.remaining_words);
        print_numbers("Avg Green/Yellow Count", |s| s.score.green_yellow_count);
        print_numbers("Avg Green Count", |s| s.score.green_count);
        println!();
    }
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

    let failed = MAX_GUESSES < words.len();
    if failed {
        words.pop();
    }
    let guesses = words;

    let mut possibilities = all_possible_answers();
    let mut knowledge = WordKnowledge::new();

    let term_width = terminal_size().map_or(80, |(w, _h)| w.0 as usize);
    let column_count = (term_width - LABEL_WIDTH - 2) / (COLUMN_WIDTH + 1);

    println!("\n================= Guess Analysis =================\n");
    for (guess_index, guess) in guesses.into_iter().enumerate() {
        if cmd_args.suggest_first_guess || guess_index != 0 {
            let suggestions = best_guesses(&possibilities, column_count);
            print_suggestions(&suggestions, &knowledge);
        }

        let colored_guess = color_guess(guess, answer);
        println_label_value(
            &format!("Guess #{}", guess_index + 1),
            &colored_guess.formatted(),
        );

        if guess == answer {
            println_label_value("Solve State", "Solved");
            break;
        }

        let conflicts = knowledge.check_for_conflicts(guess);
        if !conflicts.is_empty() {
            println_note("This guess conflicts with previously collected information:");
            for conflict in conflicts {
                println_note(&format!("    {}", &conflict.as_text()));
            }
        } else if !possibilities.iter().any(|a| a.word == guess) {
            println_note("This guess was not in the list of remaining possibilities!");
        }

        knowledge.add_guess(&colored_guess);
        println_label_value("Solve State", &knowledge.formatted());

        possibilities.retain(|p| knowledge.matches(p));
        print_remaining_words(&possibilities, &knowledge, column_count);

        println!();
    }

    if failed {
        println_label_value("Solution", &letters_with_bg(&answer, Color::Green));
    }

    ExitCode::SUCCESS
}
