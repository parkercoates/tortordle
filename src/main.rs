mod alphagram;
mod colored_guess;
mod guess_suggestions;
mod knowledge;
mod letter_set;
mod possibilities;
mod slice_subset;
mod word;

use colored_guess::{color_guess, ColoredGuess};
use guess_suggestions::{best_guesses, top_guess, Points, ScoredGuess};
use itertools::Itertools;
use knowledge::WordKnowledge;
use possibilities::{PossibleAnswer, POSSIBLE_ANSWERS};
use terminal_size::terminal_size;
use word::*;

use clap::{value_parser, Parser};
use colored::{Color, Colorize};
use std::{io::Write, process::ExitCode};

const MAX_GUESSES: usize = 6;
const MAX_WORDS: usize = MAX_GUESSES + 1;

const LABEL_WIDTH: usize = 22;

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
            for (i, possibility) in possibilities.iter().enumerate() {
                if i == 0 {
                    print_label(&format!("{} words remain", possibilities.len()));
                } else if i % columns == 0 {
                    println!();
                    print_indent();
                }
                print!("{} ", knowledge.format_word(possibility.word));
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
            let mut item = format!("{:^WORD_LENGTH$}", suggestion.rank);
            if suggestion.word == actual_guess {
                item = item.on_color(Color::Blue).to_string();
            }
            print!("{item} ");
        }
        println!();

        print_label("Suggested Guesses");
        for suggestion in suggestions {
            if suggestion.word == actual_guess {
                print!(
                    "{:^WORD_LENGTH$} ",
                    letters_to_string(suggestion.word).on_color(Color::Blue)
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
                let mut item = format!("{:^WORD_LENGTH$.*}", Points::DECIMAL_PLACES, value,);
                if value == best_value {
                    item = item.color(Color::Green).to_string();
                }
                if suggestion.word == actual_guess {
                    item = item.on_color(Color::Blue).to_string();
                }
                print!("{item} ");
            }
            println!();
        };

        // Due to its high cost, avg_score is only calculated when
        // the possibility space gets relatively small, so let's only print it
        // if it was computed.
        if suggestions.first().unwrap().avg_score != Points::zero() {
            print_numbers("Average Score", false, |s| s.avg_score);
        }
        print_numbers("Avg Remaining Words", false, |s| s.remaining_words);
        print_numbers("Avg Green/Yellow Count", true, |s| s.green_yellow_count);
        print_numbers("Avg Green Count", true, |s| s.green_count);
        println!();
    }
}

pub fn attempt_optimal_solve(answer: Word) -> Option<Vec<ColoredGuess>> {
    const STARTING_GUESS: Word = [
        Letter::from_ascii(b'R'),
        Letter::from_ascii(b'A'),
        Letter::from_ascii(b'I'),
        Letter::from_ascii(b'S'),
        Letter::from_ascii(b'E'),
    ];
    let mut guess = STARTING_GUESS;
    let mut knowledge = WordKnowledge::new();
    let mut possibilities = Vec::from(POSSIBLE_ANSWERS);
    let mut result = Vec::<ColoredGuess>::new();
    loop {
        let colored = color_guess(guess, answer);
        knowledge.add_guess(&colored);
        result.push(colored);
        if guess == answer {
            break;
        }
        possibilities.retain(|p| knowledge.matches(p));
        guess = top_guess(&possibilities)?;
    }
    Some(result)
}

fn parse_word_from_arg(s: &str) -> Result<Word, String> {
    make_word(s).ok_or(format!("'{s}' is not a word of five A-Z letters!"))
}

#[derive(Parser)]
#[command(version, about = "A command line Wordle game analyser")]
pub struct CmdArgs {
    #[arg(
        long,
        num_args = 1..=MAX_WORDS,
        value_parser = parse_word_from_arg,
        value_name = "WORDS",
        help = "Pass the game as a space separated list instead of via prompts"
    )]
    pub words: Option<Vec<Word>>,

    #[arg(
        long,
        default_value_t = false,
        help = "Show suggestions for the very first guess. Note that this is very slow and the results will be the same every time."
    )]
    pub suggest_first_guess: bool,

    #[arg(
        long,
        value_name = "COLUMNS",
        value_parser = value_parser!(u8).range(1..=32),
        help = "Set the number of columns in the output. If not set, defaults to the number of columns that fit in the terminal width."
    )]
    pub columns: Option<u8>,

    #[arg(long, help = "Force colorised output on")]
    pub color: bool,

    #[arg(long, conflicts_with = "color", help = "Force colorised output off")]
    pub no_color: bool,
}

fn column_count_from_width() -> usize {
    let term_width = terminal_size().map(|(w, _h)| w.0).unwrap_or(80);
    // This calculation must be signed as it can go negative...
    let column_count = (i32::from(term_width) - LABEL_WIDTH as i32 - 2) / (WORD_LENGTH as i32 + 1);
    // ...but that isn't a big deal as we need to clamp the value at 1 anyway.
    column_count.clamp(1, 32) as usize
}

fn main() -> ExitCode {
    let cmd_args = CmdArgs::parse();

    if cmd_args.color {
        colored::control::set_override(true);
    } else if cmd_args.no_color {
        colored::control::set_override(false);
    }

    let column_count = cmd_args
        .columns
        .map(usize::from)
        .unwrap_or_else(column_count_from_width);

    let words = cmd_args.words.unwrap_or_else(prompt_for_words);
    if words.is_empty() {
        return ExitCode::FAILURE;
    }

    let failed = MAX_GUESSES < words.len();
    let (answer, guesses) = if failed {
        let mut words = words;
        (words.pop().unwrap(), words)
    } else {
        (*words.last().unwrap(), words)
    };

    let known_answer = POSSIBLE_ANSWERS.iter().any(|a| a.word == answer);
    if !known_answer {
        println!();
        println_note(&format!(
            "{} is not in the list of known potential answers.",
            letters_to_string(answer)
        ));
        println_note("That shouldn't happen. Analysis is unlikely to go well.");
    }

    let mut knowledge = WordKnowledge::new();
    let mut possibilities = Vec::from(POSSIBLE_ANSWERS);

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
        } else if !POSSIBLE_ANSWERS.iter().any(|a| a.word == guess) {
            println_indented_note("This guess is not one of the potential Wordle answers.");
        }

        knowledge.add_guess(&colored_guess);
        println_label_value("Solve State", &knowledge.formatted());

        possibilities.retain(|p| knowledge.matches(p));
        print_remaining_words(&possibilities, &knowledge, column_count);

        println!();
    }

    if failed {
        println_label_value("Solution", &letters_with_bg(answer, Color::Green));
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
            println_indented_note("I failed.");
        }
    } else {
        println_note(
            "Something went wrong while attempting to find an optimal solution to the puzzle.",
        );
    }

    ExitCode::SUCCESS
}
