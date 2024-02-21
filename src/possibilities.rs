use crate::alphagram::Alphagram;
use crate::word::Word;

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

#[allow(clippy::forget_non_drop)]
pub const POSSIBLE_ANSWERS: &[PossibleAnswer] = &konst::iter::collect_const!(PossibleAnswer =>
    konst::string::split(konst::string::trim(include_str!("WORDLE-ANSWERS.txt")), '\n'),
    map(|line| PossibleAnswer::from_word(Word::expect_from_str(line))),
);
