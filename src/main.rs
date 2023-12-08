mod colored_guess;
mod data_structures;
mod guess_suggestions;
mod knowledge;
mod possibilities;
mod slice_subset;
mod word;

use colored_guess::{color_guess, ColoredGuess};
use guess_suggestions::{best_guesses, rand_top_guess, Points, ScoredGuess};
use itertools::Itertools;
use knowledge::WordKnowledge;
use possibilities::{all_possible_answers, PossibleAnswer};
use terminal_size::terminal_size;
use word::*;

use colored::{Color, Colorize};
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
    println!("{}", &str_with_fg(text, Color::Magenta),);
}

fn println_indented_note(text: &str) {
    print_indent();
    println_note(text);
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
                "--color" => colored::control::set_override(true),
                "--no-color" => colored::control::set_override(false),
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
        0 => println_indented_note("There are no words remaining! Something went wrong!"),
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

fn print_suggestions(suggestions: &[ScoredGuess], knowledge: &WordKnowledge, actual_guess: Word) {
    // If there are two suggestions or less, they are all equally good. If the
    // last suggestion's rank is 1, they are all tied and therefore equally
    // good. Regardless, let's not bother showing any suggested guesses.
    let all_equally_good = suggestions.len() <= 2 || suggestions.last().unwrap().rank == 1;
    if !all_equally_good {
        print_indent();
        for suggestion in suggestions {
            let mut item = format!("{:^COLUMN_WIDTH$}", suggestion.rank);
            if suggestion.word == actual_guess {
                item = item.on_color(Color::Blue).to_string();
            }
            print!("{} ", item);
        }
        println!();

        print_label("Suggested Guesses");
        for suggestion in suggestions {
            if suggestion.word == actual_guess {
                print!(
                    "{:^COLUMN_WIDTH$} ",
                    letters_to_string(&suggestion.word).on_color(Color::Blue)
                );
            } else {
                // We can't use COLUMN_WIDTH here because the formatting codes
                // throw off the justification counts.
                print!("{} ", knowledge.format_word(suggestion.word));
            }
        }
        println!();

        let print_numbers = |label, higher_is_better, getter: fn(&ScoredGuess) -> Points| {
            let best_suggestion = if higher_is_better {
                suggestions.iter().max_by(|a, b| getter(a).cmp(&getter(b)))
            } else {
                suggestions.iter().min_by(|a, b| getter(a).cmp(&getter(b)))
            };
            let best_value = getter(best_suggestion.unwrap());
            print_label(label);
            for suggestion in suggestions {
                let value = getter(suggestion);
                let mut item = format!("{:^COLUMN_WIDTH$.*}", Points::DECIMAL_PLACES, value,);
                if value == best_value {
                    item = item.color(Color::Green).to_string();
                }
                if suggestion.word == actual_guess {
                    item = item.on_color(Color::Blue).to_string();
                }
                print!("{} ", item);
            }
            println!();
        };

        // Due to its high cost, avg_score is only calculated when
        // the possibility space gets relatively small, so let's only print it
        // if it was computed.
        if suggestions.first().unwrap().score.avg_score != Points::zero() {
            print_numbers("Average Score", false, |s| s.score.avg_score);
        }
        print_numbers("Avg Remaining Words", false, |s| s.score.remaining_words);
        print_numbers("Avg Green/Yellow Count", true, |s| {
            s.score.green_yellow_count
        });
        print_numbers("Avg Green Count", true, |s| s.score.green_count);
        println!();
    }
}

pub fn attempt_optimal_solve(answer: Word) -> Option<Vec<ColoredGuess>> {
    const STARTING_GUESS: Word = [b'R', b'A', b'I', b'S', b'E'];
    let mut guess = STARTING_GUESS;
    let mut knowledge = WordKnowledge::new();
    let mut possibilities = all_possible_answers();
    let mut result = Vec::<ColoredGuess>::new();
    loop {
        let colored = color_guess(guess, answer);
        knowledge.add_guess(&colored);
        result.push(colored);
        if guess == answer {
            break;
        }
        possibilities.retain(|p| knowledge.matches(p));
        // In the case of ties, we take a random option from the tie, since that
        // feels nicer than always just taking the first alphabetically. This
        // means the output isn't always deterministic.
        guess = rand_top_guess(&possibilities)?;
    }
    Some(result)
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
    let known_answer = possibilities.iter().filter(|a| a.word == answer).count() == 1;
    if !known_answer {
        println!();
        println_note(&format!(
            "{} is not in the list of known potential answers.",
            letters_to_string(&answer)
        ));
        println_note("That shouldn't happen. Analysis is unlikely to go well.");
    }

    let mut knowledge = WordKnowledge::new();

    let term_width = terminal_size().map_or(80, |(w, _h)| w.0 as usize);
    let column_count = (term_width - LABEL_WIDTH - 2) / (COLUMN_WIDTH + 1);

    println!("\n================= Guess Analysis =================\n");
    for (guess_index, guess) in guesses.into_iter().enumerate() {
        if cmd_args.suggest_first_guess || guess_index != 0 {
            let suggestions = best_guesses(&possibilities, column_count, guess_index);
            print_suggestions(&suggestions, &knowledge, guess);
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
            println_indented_note("This guess conflicts with previously collected information:");
            for conflict in conflicts {
                println_indented_note(&format!("    {}", &conflict.as_text()));
            }
        } else if !possibilities.iter().any(|a| a.word == guess) {
            println_indented_note("This guess was not in the list of remaining possibilities!");
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

    // Let the algorithm attempt solve it itself.
    println!();
    print_label("My Best Attempt");
    if let Some(guesses) = attempt_optimal_solve(answer) {
        println!(
            "{}",
            &guesses.iter().map(ColoredGuess::formatted).join(" -> "),
        );
        if MAX_GUESSES < guesses.len() {
            println_indented_note("I failed.")
        }
    } else {
        println_note(
            "Something went wrong while attempting to find an optimal solution to the puzzle.",
        );
    }

    ExitCode::SUCCESS
}
