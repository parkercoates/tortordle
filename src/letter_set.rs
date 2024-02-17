use crate::word::Letter;

#[derive(Clone, Copy)]
pub struct LetterSet {
    bits: u32,
}

impl LetterSet {
    pub const fn new() -> Self {
        Self { bits: 0 }
    }

    pub fn insert(&mut self, letter: Letter) {
        self.bits |= 1 << letter.index();
    }

    pub const fn contains(self, letter: Letter) -> bool {
        self.bits & (1 << letter.index()) != 0
    }

    pub fn letters(self) -> impl Iterator<Item = Letter> {
        (b'A'..=b'Z')
            .map(Letter::from_ascii)
            .filter(move |&l| self.contains(l))
    }
}
