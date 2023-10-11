use crate::data_structures::Alphagram;
use crate::word::{Letter, Word, WORD_LENGTH};

#[derive(Clone, Copy)]
pub struct PossibleAnswer {
    pub word: Word,
    pub alphagram: Alphagram,
}

impl PossibleAnswer {
    pub fn from_word(word: Word) -> Self {
        Self {
            word,
            alphagram: Alphagram::from_word(word),
        }
    }
}

static ANSWERS: [Letter; 13890] = *std::include_bytes!("WORDLE-ANSWERS.txt");

const fn slice_as_array_ref<T, const N: usize>(slice: &[T]) -> &[T; N] {
    assert!(N <= slice.len());
    unsafe { &*slice.as_ptr().cast::<[T; N]>() }
}

pub fn all_possible_answers() -> Vec<PossibleAnswer> {
    ANSWERS
        .chunks_exact(WORD_LENGTH + 1) // Five letters then a newline
        .map(|chunk| {
            let word: Word = *slice_as_array_ref(chunk);
            assert!(word.iter().all(u8::is_ascii_uppercase));
            PossibleAnswer::from_word(word)
        })
        .collect()
}
