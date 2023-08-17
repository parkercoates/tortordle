use colored::{Color, Colorize};
use itertools::Itertools;
use std::{
    cmp::max,
    collections::{BTreeMap, BTreeSet},
    io::Write,
    iter::zip,
    process::ExitCode,
};

// Stolen from unstable
#[inline]
pub const fn first_chunk<const N: usize>(slice: &[u8]) -> Option<&[u8; N]> {
    if slice.len() < N {
        None
    } else {
        // SAFETY: We explicitly check for the correct number of elements,
        //   and do not let the reference outlive the slice.
        Some(unsafe { &*(slice.as_ptr() as *const [u8; N]) })
    }
}

type Letter = u8;
const WORD_LENGTH: usize = 5;
type Word = [Letter; WORD_LENGTH];

static ANSWERS: [Letter; 13890] = *std::include_bytes!("WORDLE-ANSWERS.txt");

fn make_word(input: &str) -> Option<Word> {
    let upper = input.to_ascii_uppercase();
    let bytes = upper.as_bytes();
    let valid = bytes.len() == 5 && bytes.iter().all(Letter::is_ascii_uppercase);
    if valid {
        bytes.try_into().ok()
    } else {
        None
    }
}

fn letter_to_string(letter: Letter) -> String {
    (letter as char).to_string()
}

fn letters_to_string<'a, I>(letters: I) -> String
where
    I: IntoIterator<Item = &'a u8>,
{
    letters.into_iter().map(|l| *l as char).collect()
}

fn letters_with_fg<'a, I>(letters: I, color: Color) -> String
where
    I: IntoIterator<Item = &'a u8>,
{
    letters_to_string(letters).color(color).to_string()
}

fn letters_with_bg<'a, I>(letters: I, color: Color) -> String
where
    I: IntoIterator<Item = &'a u8>,
{
    letters_to_string(letters).on_color(color).to_string()
}

fn letter_with_fg(letter: Letter, color: Color) -> String {
    letters_with_fg(std::slice::from_ref(&letter), color)
}

fn letter_with_bg(letter: Letter, color: Color) -> String {
    letters_with_bg(std::slice::from_ref(&letter), color)
}

fn str_with_fg(text: &str, color: Color) -> String {
    String::from(text).color(color).to_string()
}

#[derive(Clone, Copy)]
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
}

fn color_guess(guess: Word, answer: Word) -> ColoredWord {
    fn letter_count(word: &[Letter], letter: Letter) -> usize {
        word.iter().filter(|&c| *c == letter).count()
    }

    let mut slots = [(b' ', LetterState::Black); WORD_LENGTH];
    for (i, letter, answer_letter, slot) in itertools::izip!(0.., guess, answer, &mut slots) {
        slot.0 = letter;
        if letter == answer_letter {
            slot.1 = LetterState::Green;
        } else {
            let count_in_answer = letter_count(&answer, letter);
            let count_so_far_in_guess = letter_count(&guess[..i], letter);
            if count_so_far_in_guess < count_in_answer {
                slot.1 = LetterState::Yellow;
            }
        }
    }
    ColoredWord { slots }
}

#[derive(Clone, Debug)]
enum LetterKnowledge {
    Is(Letter),
    IsNot(BTreeSet<Letter>),
}

impl LetterKnowledge {
    fn formatted(&self, yellows: &LetterHistogram) -> String {
        match self {
            Self::Is(letter) => letter_with_fg(*letter, Color::Green),
            Self::IsNot(set) => {
                letters_with_fg(
                    yellows.map.keys().filter(|l| !set.contains(l)),
                    Color::Yellow,
                ) + &letters_with_fg(set, Color::Red)
            }
        }
    }

    fn matches(&self, letter: Letter) -> bool {
        match self {
            Self::Is(known_letter) => letter == *known_letter,
            Self::IsNot(set) => !set.contains(&letter),
        }
    }
}

#[derive(Clone)]
struct LetterHistogram {
    map: BTreeMap<Letter, usize>,
}

impl LetterHistogram {
    const fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    fn from_word(word: &[Letter]) -> Self {
        let mut histogram = Self::new();
        for &letter in word {
            histogram.add_letter(letter);
        }
        histogram
    }

    fn add_letter(&mut self, letter: Letter) {
        *self.map.entry(letter).or_default() += 1;
    }

    fn remove_letter(&mut self, letter: Letter) {
        if let Some(count) = self.map.get_mut(&letter) {
            *count -= 1;
            if *count == 0 {
                self.map.remove(&letter);
            }
        }
    }

    fn merge_via_max(&mut self, other: &Self) {
        for (letter, count) in &other.map {
            self.map
                .entry(*letter)
                .and_modify(|x| *x = max(*x, *count))
                .or_insert(*count);
        }
    }

    fn contains(&self, letter: Letter) -> bool {
        self.map.contains_key(&letter)
    }

    fn contains_other(&self, subset: &Self) -> bool {
        subset.map.iter().all(|(letter, needed_count)| {
            let count = self.map.get(letter).unwrap_or(&0);
            needed_count <= count
        })
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
            slots: [
                LetterKnowledge::IsNot(BTreeSet::new()),
                LetterKnowledge::IsNot(BTreeSet::new()),
                LetterKnowledge::IsNot(BTreeSet::new()),
                LetterKnowledge::IsNot(BTreeSet::new()),
                LetterKnowledge::IsNot(BTreeSet::new()),
            ],
            histogram: LetterHistogram::new(),
            yellows: LetterHistogram::new(),
        }
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

        self.histogram.merge_via_max(&new_histogram);

        self.yellows = self.histogram.clone();
        for slot in &mut self.slots {
            if let LetterKnowledge::Is(letter) = slot {
                self.yellows.remove_letter(*letter);
            }
        }

        for (letter, count) in self.yellows.map.clone() {
            let match_indexes: Vec<usize> = self
                .slots
                .iter()
                .enumerate()
                .filter_map(|(i, slot)| slot.matches(letter).then_some(i))
                .collect();
            if match_indexes.len() == count {
                for i in match_indexes {
                    self.slots[i] = LetterKnowledge::Is(letter);
                }
                self.yellows.remove_letter(letter);
            }
        }
    }

    fn matches(&self, word: Word) -> bool {
        let slots_match = zip(&self.slots, &word).all(|(s, l)| s.matches(*l));
        let needs_match = LetterHistogram::from_word(&word).contains_other(&self.histogram);
        slots_match && needs_match
    }

    fn formatted(&self) -> String {
        format!(
            "[{}]",
            self.slots
                .iter()
                .map(|slot| slot.formatted(&self.yellows))
                .join("|")
        )
    }

    fn format_remaining(&self, word: Word) -> String {
        std::iter::zip(word, &self.slots)
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

    let mut possibilities: Vec<Word> = ANSWERS
        .chunks_exact(6)
        .map(|w| *first_chunk::<WORD_LENGTH>(w).unwrap())
        .collect();

    println!("\nGuess Analysis:");
    let mut knowledge = WordKnowledge::new();
    for word in &words {
        let was_mistake = !knowledge.matches(*word);

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
        } else if !possibilities.contains(word) {
            println!(
                "           {}",
                str_with_fg(
                    "NOTE! This guess was not in the list of remaining possibilities!",
                    Color::Magenta
                )
            );
        }

        possibilities.retain(|w| knowledge.matches(*w));
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
                knowledge.format_remaining(possibilities[0])
            ),
            _ => {
                let words_string = &possibilities
                    .iter()
                    .map(|w| knowledge.format_remaining(*w))
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

        println!();
    }
    println!("   {}   Solution", letters_with_bg(&answer, Color::Green));
    ExitCode::SUCCESS
}
