use crate::alphagram::Alphagram;
use crate::letter::fmt_letters;
use crate::word::Word;

use std::fmt::Debug;

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

impl Debug for PossibleAnswer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_letters(self.word.letters, f)
    }
}

#[expect(
    clippy::forget_non_drop,
    reason = "This warning is internal to konst and presumably safe for us to ignore."
)]
pub const POSSIBLE_ANSWERS: &[PossibleAnswer] = &konst::iter::collect_const!(PossibleAnswer =>
    konst::string::split(konst::string::trim(include_str!("WORDLE-ANSWERS.txt")), '\n'),
    map(|line| PossibleAnswer::from_word(Word::expect_from_str(line))),
);
