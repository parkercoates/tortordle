use crate::alphagram::Alphagram;
use crate::word::{Letter, Word};

pub const fn l(c: char) -> Letter {
    let l = Letter::from_char(c);
    assert!(l.is_valid());
    l
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

pub fn a(s: &str) -> Alphagram {
    let mut result = Alphagram::new();
    for c in s.chars() {
        result.insert(l(c));
    }
    result
}

pub fn dbg<T>(v: T) -> String
where
    T: std::fmt::Debug,
{
    format!("{v:?}")
}
