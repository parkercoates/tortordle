use colored::{Color, Colorize};
use itertools::Itertools;
use std::{cmp::Ordering, io::Write, iter::zip, process::ExitCode};

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
struct LetterSet {
    bits: u32,
}

impl LetterSet {
    const fn new() -> Self {
        Self { bits: 0 }
    }

    fn insert(&mut self, letter: Letter) {
        self.bits |= 1 << (letter - b'A');
    }

    const fn contains(&self, letter: Letter) -> bool {
        self.bits & (1 << (letter - b'A')) != 0
    }

    fn letters(&self) -> Vec<Letter> {
        (b'A'..=b'Z')
            .into_iter()
            .filter(|&l| self.contains(l))
            .collect()
    }
}

#[derive(Clone, Copy)]
enum LetterKnowledge {
    Is(Letter),
    IsNot(LetterSet),
}

impl LetterKnowledge {
    fn formatted(&self, yellows: &LetterHistogram) -> String {
        match self {
            Self::Is(letter) => letter_with_fg(*letter, Color::Green),
            Self::IsNot(set) => {
                letters_with_fg(
                    yellows.letters().dedup().filter(|&l| !set.contains(*l)),
                    Color::Yellow,
                ) + &letters_with_fg(&set.letters(), Color::Red)
            }
        }
    }

    fn matches(&self, letter: Letter) -> bool {
        match self {
            Self::Is(known_letter) => letter == *known_letter,
            Self::IsNot(set) => !set.contains(letter),
        }
    }
}

#[derive(Clone)]
struct LetterHistogram {
    slots: [Letter; WORD_LENGTH],
}

impl LetterHistogram {
    // This value is specifically chosen to be larger than b'Z'.
    const NO_DATA: Letter = b'_';

    const fn new() -> Self {
        Self {
            slots: [Self::NO_DATA; WORD_LENGTH],
        }
    }

    fn from_word(word: Word) -> Self {
        let mut histogram = Self::new();
        for letter in word {
            histogram.add_letter(letter);
        }
        histogram
    }

    fn add_letter(&mut self, mut letter: Letter) {
        for i in 0..WORD_LENGTH {
            if letter < self.slots[i] {
                std::mem::swap(&mut self.slots[i], &mut letter);
            }
        }
    }

    fn remove_letter(&mut self, letter: Letter) {
        for i in 0..WORD_LENGTH {
            if self.slots[i] == letter {
                self.slots[i] = Self::NO_DATA;
                self.slots.sort_unstable();
                break;
            }
        }
    }

    fn letters(&self) -> impl Iterator<Item = &Letter> {
        self.slots.iter().take_while(|&l| *l != Self::NO_DATA)
    }

    fn contains(&self, letter: Letter) -> bool {
        self.slots.contains(&letter)
    }

    fn contains_other(&self, subset: &Self) -> bool {
        let mut i: usize = 0;
        let mut j: usize = 0;
        loop {
            match self.slots[i].cmp(&subset.slots[j]) {
                Ordering::Equal => {
                    i += 1;
                    j += 1;
                }
                Ordering::Greater => {
                    return false;
                }
                Ordering::Less => {
                    i += 1;
                }
            }

            if j == WORD_LENGTH || subset.slots[j] == Self::NO_DATA {
                return true;
            } else if i == WORD_LENGTH || self.slots[i] == Self::NO_DATA {
                return false;
            }
        }
    }

    fn merge_via_max(&mut self, other: &Self) {
        let mut new = [b'_'; WORD_LENGTH];
        let mut i = 0;
        let mut j = 0;
        let mut k = 0;
        while k < WORD_LENGTH {
            match self.slots[i].cmp(&other.slots[j]) {
                Ordering::Equal => {
                    new[k] = self.slots[i];
                    i += 1;
                    j += 1;
                    k += 1;
                }
                Ordering::Greater => {
                    new[k] = other.slots[j];
                    j += 1;
                    k += 1;
                }
                Ordering::Less => {
                    new[k] = self.slots[i];
                    i += 1;
                    k += 1;
                }
            }

            if i == WORD_LENGTH {
                while j < WORD_LENGTH && k < WORD_LENGTH {
                    new[k] = other.slots[j];
                    j += 1;
                    k += 1;
                }
                break;
            } else if j == WORD_LENGTH {
                while i < WORD_LENGTH && k < WORD_LENGTH {
                    new[k] = self.slots[i];
                    i += 1;
                    k += 1;
                }
                break;
            }
        }
        self.slots = new;
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
        let needs_match = possibility.histogram.contains_other(&self.histogram);
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
                .map(|slot| slot.formatted(&self.yellows))
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

    let mut possibilities: Vec<PossibleAnswer> = ANSWERS
        .chunks_exact(6)
        .map(|w| {
            let word = first_chunk::<WORD_LENGTH>(w).unwrap();
            PossibleAnswer::from_word(*word)
        })
        .collect();

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

        println!();
    }
    println!("   {}   Solution", letters_with_bg(&answer, Color::Green));
    ExitCode::SUCCESS
}
