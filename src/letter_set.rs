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
        self.bits |= 1 << (letter - b'A');
    }

    pub const fn contains(self, letter: Letter) -> bool {
        self.bits & (1 << (letter - b'A')) != 0
    }

    pub fn letters(self) -> Vec<Letter> {
        (b'A'..=b'Z').filter(|&l| self.contains(l)).collect()
    }
}
