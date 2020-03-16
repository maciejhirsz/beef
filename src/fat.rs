use core::num::NonZeroUsize;
use crate::traits::Word;

pub type Cow<'a, T> = crate::generic::Cow<'a, T, usize>;

impl Word for usize {
    type NonZero = NonZeroUsize;

    #[inline]
    fn from(word: usize) -> usize {
        word
    }

    #[inline]
    fn nonzero_from(word: usize) -> Option<NonZeroUsize> {
        NonZeroUsize::new(word)
    }
}