use crate::alphagram::Alphagram;
use crate::colored_guess::{color_guess, ColoredGuess};
use crate::letter::Letter;
use crate::letter_set::LetterSet;
use crate::possibilities::PossibleAnswer;
use crate::word::Word;

fn is_sorted_and_unique(s: &str) -> bool {
    let mut previous = '\0';
    for c in s.chars() {
        if c <= previous {
            return false;
        }
        previous = c;
    }
    true
}

pub const fn l(c: char) -> Letter {
    assert!('A' <= c && c <= 'Z');
    Letter::from_char(c)
}

pub fn ls(s: &str) -> StrLetters {
    StrLetters(s.chars())
}

pub struct StrLetters<'a>(std::str::Chars<'a>);

impl<'a> Iterator for StrLetters<'a> {
    type Item = Letter;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(l)
    }
}

pub const fn w(s: &str) -> Word {
    Word::expect_from_str(s)
}

pub fn ar(s: &str) -> [Letter; Word::LENGTH] {
    let mut ar = [Letter::NO_LETTER; Word::LENGTH];
    let bytes = s.bytes();
    assert!(bytes.len() <= Word::LENGTH);
    for (i, b) in bytes.enumerate() {
        ar[i] = l(b as char);
    }
    ar
}

pub fn lset(s: &str) -> LetterSet {
    assert!(is_sorted_and_unique(s));
    let mut result = LetterSet::new();
    for c in s.chars() {
        result.insert(l(c));
    }
    result
}

pub fn a(s: &str) -> Alphagram {
    let mut result = Alphagram::new();
    for c in s.chars() {
        result.insert(l(c));
    }
    result
}

pub fn g(word: &str, answer: &str) -> ColoredGuess {
    color_guess(w(word), w(answer))
}

pub const fn p(s: &str) -> PossibleAnswer {
    PossibleAnswer::from_word(w(s))
}

pub fn dbg<T>(v: T) -> String
where
    T: std::fmt::Debug,
{
    format!("{v:?}")
}
