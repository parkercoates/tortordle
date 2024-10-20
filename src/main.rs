use tortordle::colored_guess::{color_guess, ColoredGuess};
use tortordle::guess_suggestions::{best_guesses, top_guess, Points, ScoredGuess};
use tortordle::knowledge::WordKnowledge;
use tortordle::letter::letters_with_bg;
use tortordle::possibilities::{PossibleAnswer, POSSIBLE_ANSWERS};
use tortordle::word::Word;

use clap::{value_parser, Parser};
use colored::{Color, Colorize};
use itertools::Itertools;
use std::fmt::Display;
use std::{io::Write, process::ExitCode};
use terminal_size::terminal_size;

const MAX_GUESSES: usize = 6;
const MAX_WORDS: usize = MAX_GUESSES + 1;

const LABEL_WIDTH: usize = 22;

fn print_indent() {
    print!("{:>LABEL_WIDTH$}  ", "");
}

fn print_label(label: &str) {
    print!("{label:>LABEL_WIDTH$}: ",);
}

fn println_label_value<V: Display>(label: &str, value: V) {
    println!("{label:>LABEL_WIDTH$}: {value}",);
}

fn println_note(text: &str) {
    println!("{}", &text.color(Color::Magenta));
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
        let _ = std::io::stdout().flush();

        if std::io::stdin().read_line(&mut input).is_err() {
            println!("Failed to read line!");
            continue;
        }

        let input = input.trim();

        if input.is_empty() {
            return None;
        }

        let word = Word::from_str(input);
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
        0 => println_indented_note("I am not aware of any remaining possibilities."),
        1 => println_label_value(
            "1 word remains",
            knowledge.format_word(possibilities[0].word),
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
    const COLUMN_WIDTH: usize = Word::LENGTH;

    // If there are two suggestions or less, they are all equally good. If the
    // last suggestion's rank is 1, they are all tied and therefore equally
    // good. Regardless, let's not bother showing any suggested guesses.
    let [first, _, .., last] = suggestions else {
        return;
    };
    if last.rank == 1 {
        return;
    }

    print_indent();
    for suggestion in suggestions {
        let mut item = format!("{:^COLUMN_WIDTH$}", suggestion.rank).normal();
        if suggestion.word == actual_guess {
            item = item.on_color(Color::Blue);
        }
        print!("{item} ");
    }
    println!();

    print_label("Suggested Guesses");
    for suggestion in suggestions {
        if suggestion.word == actual_guess {
            print!(
                "{:^COLUMN_WIDTH$} ",
                suggestion.word.to_string().on_color(Color::Blue)
            );
        } else {
            // We can't use COLUMN_WIDTH here because the formatting codes
            // throw off the justification counts.
            print!("{} ", knowledge.format_word(suggestion.word));
        }
    }
    println!();

    let print_numbers = |label, getter: fn(&ScoredGuess) -> Points| {
        let best_value = suggestions
            .iter()
            .map(getter)
            .max()
            .expect("suggestions not empty");
        print_label(label);
        for suggestion in suggestions {
            let value = getter(suggestion).abs();
            let mut item = format!("{:^COLUMN_WIDTH$.*}", Points::DECIMAL_PLACES, value).normal();
            if value == best_value {
                item = item.color(Color::Green);
            }
            if suggestion.word == actual_guess {
                item = item.on_color(Color::Blue);
            }
            print!("{item} ");
        }
        println!();
    };

    // Due to its high cost, avg_score is only calculated when
    // the possibility space gets relatively small, so let's only print it
    // if it was computed.
    if first.avg_score != Points::zero() {
        print_numbers("Average Score", |s| -s.avg_score);
    }
    print_numbers("Avg Remaining Words", |s| -s.remaining_words);
    print_numbers("Avg Green/Yellow Count", |s| s.green_yellow_count);
    print_numbers("Avg Green Count", |s| s.green_count);
    println!();
}

fn attempt_optimal_solve(starting_guess: Word, answer: Word) -> Option<Vec<ColoredGuess>> {
    let mut guess = starting_guess;
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

fn attempt_to_solve_all(starting_guess: Word) {
    let attempt_and_print = |word: &Word| {
        print_label(&word.to_string());
        if let Some(guesses) = attempt_optimal_solve(starting_guess, *word) {
            println!(
                "{} {}",
                guesses.len(),
                guesses.iter().map(ColoredGuess::formatted).join(" -> ")
            );
            Some(guesses.len())
        } else {
            println_note("Error");
            None
        }
    };

    const TOTAL_COUNT: usize = POSSIBLE_ANSWERS.len();
    println!("\n===== Attempting All Possible Games =====\n");
    let mut failures = Vec::new();
    let mut total_guesses = 0usize;
    for PossibleAnswer { word, .. } in POSSIBLE_ANSWERS {
        let score = attempt_and_print(word);
        total_guesses += score.unwrap_or(0);
        let won = score.is_some_and(|x| x <= MAX_GUESSES);
        if !won {
            failures.push(*word);
        }
    }
    let success_count = TOTAL_COUNT - failures.len();
    let success_rate = 100.0 * success_count as f64 / TOTAL_COUNT as f64;
    let avg_guesses = total_guesses as f64 / TOTAL_COUNT as f64;
    println!();
    println_label_value(
        "Success Rate",
        format_args!("{success_count}/{TOTAL_COUNT} = {success_rate:.3}%"),
    );
    println_label_value("Average Guesses", format_args!("{avg_guesses:.3}"));
    println_label_value("Failures", failures.len());
    for failure in &failures {
        attempt_and_print(failure);
    }
}

fn parse_word_from_arg(s: &str) -> Result<Word, String> {
    Word::from_str(s).ok_or(format!("'{s}' is not a word of five A-Z letters!"))
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

    #[arg(
        long,
        value_name = "STARTING_WORD",
        value_parser = parse_word_from_arg,
        help = "Attempt to solve every possible Wordle game from the given starting word.",
        conflicts_with_all = ["words", "columns", "suggest_first_guess"]
    )]
    pub solve_all: Option<Word>,

    #[arg(long, help = "Force colorised output on")]
    pub color: bool,

    #[arg(long, conflicts_with = "color", help = "Force colorised output off")]
    pub no_color: bool,
}

fn column_count_from_width() -> usize {
    let term_width = terminal_size().map(|(w, _h)| w.0).unwrap_or(80);
    // This calculation must be signed as it can go negative...
    let column_count = (i32::from(term_width) - LABEL_WIDTH as i32 - 2) / (Word::LENGTH as i32 + 1);
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

    // --solve-all is a separate mode, independent of the game analyser.
    if let Some(starting_guess) = cmd_args.solve_all {
        attempt_to_solve_all(starting_guess);
        return ExitCode::SUCCESS;
    }

    let words = cmd_args.words.unwrap_or_else(prompt_for_words);
    if words.is_empty() {
        return ExitCode::FAILURE;
    }

    let failed = MAX_GUESSES < words.len();
    let answer = *words.last().expect("words not empty");

    let guesses = if failed {
        let mut guesses = words;
        guesses.pop();
        guesses
    } else {
        words
    };
    let starting_guess = *guesses.first().expect("guesses guesses empty");

    let known_answer = POSSIBLE_ANSWERS.iter().any(|a| a.word == answer);
    if !known_answer {
        println_note(&format!(
            "
{answer} is not in my list of known potential answers.

The Wordle answer list is no longer public and new words are occasionally added
to it. If {answer} was the answer to a Wordle puzzle, please let Parker know
and he will add it to my list."
        ));
    }

    let mut knowledge = WordKnowledge::new();
    let mut possibilities = Vec::from(POSSIBLE_ANSWERS);

    println!("\n================= Guess Analysis =================\n");
    for (guess_index, guess) in guesses.into_iter().enumerate() {
        if cmd_args.suggest_first_guess || guess_index != 0 {
            let suggestions = best_guesses(&possibilities, column_count, guess_index, Some(guess));
            print_suggestions(&suggestions, &knowledge, guess);
        }

        let colored_guess = color_guess(guess, answer);
        println_label_value(
            &format!("Guess #{}", guess_index + 1),
            colored_guess.formatted(),
        );

        if guess == answer {
            println_label_value("Solve State", "Solved");
            break;
        }

        if !POSSIBLE_ANSWERS.iter().any(|a| a.word == guess) {
            println_indented_note("This guess is not in my list of the potential Wordle answers.");
        }

        let conflicts = knowledge.check_for_conflicts(guess);
        if !conflicts.is_empty() {
            println_indented_note("This guess conflicts with previously collected information:");
            for conflict in conflicts {
                println_indented_note(&format!("    {}", &conflict.as_text()));
            }
        }

        knowledge.add_guess(&colored_guess);
        println_label_value("Solve State", knowledge.formatted());

        possibilities.retain(|p| knowledge.matches(p));
        print_remaining_words(&possibilities, &knowledge, column_count);

        println!();
    }

    if failed {
        println_label_value("Solution", letters_with_bg(answer, Color::Green));
    }

    // Let the algorithm attempt solve it itself.
    if known_answer {
        println!();
        print_label("My Best Attempt");
        if let Some(guesses) = attempt_optimal_solve(starting_guess, answer) {
            println!(
                "{}",
                &guesses.iter().map(ColoredGuess::formatted).join(" -> "),
            );
            if MAX_GUESSES < guesses.len() {
                println_indented_note("I failed.");
            }
        } else {
            println_note("Something went wrong while attempting to find an optimal solution.");
        }
    }

    ExitCode::SUCCESS
}
