use crate::alphagram::Alphagram;
use crate::word::{Word, WORD_LENGTH};

#[derive(Clone)]
pub struct PossibleAnswer {
    pub word: Word,
    pub alphagram: Alphagram,
}

impl PossibleAnswer {
    pub const fn from_word(word: Word) -> Self {
        Self {
            word,
            alphagram: Alphagram::from_word(word),
        }
    }
}

const fn possible_answer_from_line(line: &str) -> PossibleAnswer {
    let bytes = line.as_bytes();
    assert!(bytes.len() == WORD_LENGTH);
    let mut word = [b' '; WORD_LENGTH];
    let mut i = 0;
    while i < WORD_LENGTH {
        assert!(bytes[i].is_ascii_uppercase());
        word[i] = bytes[i];
        i += 1;
    }
    PossibleAnswer::from_word(word)
}

#[allow(clippy::forget_non_drop)]
pub const POSSIBLE_ANSWERS: &[PossibleAnswer] = &konst::iter::collect_const!(PossibleAnswer =>
    konst::string::split(konst::string::trim(include_str!("WORDLE-ANSWERS.txt")), '\n'),
    map(possible_answer_from_line),
);
