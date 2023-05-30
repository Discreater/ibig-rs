use core::ops::Range;

use crate::{
    arch::word::{Word, WORD_BITS as ARCH_WORD_BITS},
    buffer::Buffer,
    ubig::Repr,
    UBig,
};

pub const WORD_BITS: usize = ARCH_WORD_BITS;

use alloc::borrow::Cow;

pub struct Ap<'a> {
    inner: Cow<'a, Buffer>,
    range: Range<usize>,
}

impl Ap<'_> {
    fn up(&self) -> usize {
        self.range.end - 1
    }
    fn down(&self) -> usize {
        self.range.start
    }
}
impl<'a> Ap<'a> {
    fn range(&self, up: usize, down: usize) -> Self {
        assert!(up <= self.up());
        assert!(down <= self.down());
        assert!(up + 1 >= down);
        let down = down + self.down();
        let up = up + self.down();
        Ap {
            inner: Cow::clone(&self.inner),
            range: down..up + 1,
        }
    }
}

impl<'a> From<&'a Buffer> for Ap<'a> {
    fn from(buffer: &'a Buffer) -> Self {
        Self {
            inner: Cow::Borrowed(buffer),
            range: 0..buffer.len(),
        }
    }
}

impl From<Buffer> for Ap<'_> {
    fn from(buffer: Buffer) -> Self {
        Self {
            range: 0..buffer.len(),
            inner: Cow::Owned(buffer),
        }
    }
}

pub trait ApBig {
    fn range(&self, up: usize, down: usize) -> Self;
}

impl ApBig for UBig {
    fn range(&self, up: usize, down: usize) -> Self {
        assert!(down <= up + 1);
        let shiftted = self >> down;
        let len = up - down + 1;
        match shiftted.into_repr() {
            Repr::Small(i) => UBig::from(i & ((1 << len) - 1)),
            Repr::Large(mut large) => {
                truncate_in_place(&mut large, len);
                UBig::from(large)
            }
        }
    }
}

impl ApBig for Word {
    fn range(&self, up: usize, down: usize) -> Self {
        assert!(down <= up + 1);
        let len = up - down + 1;
        self >> down & ((1 << len) - 1)
    }
    // fn word(&self, index: usize) -> Word {
    //     assert_eq!(index, 0);
    //     *self
    // }
}

pub(crate) fn truncate_in_place(words: &mut Buffer, len: usize) {
    let len_words = (len + WORD_BITS - 1) / WORD_BITS;
    let mask = (1 << (len % WORD_BITS)) - 1;
    if words.len() >= len_words {
        words[len_words - 1] &= mask;
    }
    words.truncate(len_words);
}

#[cfg(test)]
mod tests {
    use crate::{bit::ApBig, UBig};

    #[test]
    fn small_range() {
        let a = super::UBig::from_word(0b1100111110);
        let b = a.range(6, 2);
        assert_eq!(b, super::UBig::from_word(0b01111));
    }

    #[test]
    fn big_range() {
        let cases = [
            ("0x1234aebc032148970911", 18, 12, "0b1110000"),
            ("0x1234aebc032148970911", 79, 12, "0x1234aebc032148970"),
        ];
        for (a, up, down, re) in cases {
            let a = UBig::from_str_with_radix_prefix(a).unwrap();
            let b = a.range(up, down);
            assert_eq!(b, UBig::from_str_with_radix_prefix(re).unwrap());
        }
    }
}
