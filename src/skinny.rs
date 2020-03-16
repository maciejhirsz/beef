use core::num::NonZeroU32;
use crate::traits::Word;

pub type Cow<'a, T> = crate::generic::Cow<'a, T, u32>;

impl Word for usize {
    type NonZero = NonZeroU32;

    #[inline]
    fn from(word: usize) -> u32 {
        word as u32
    }

    #[inline]
    fn nonzero_from(word: usize) -> Option<NonZeroU32> {
        NonZeroU32::new(word)
    }
}