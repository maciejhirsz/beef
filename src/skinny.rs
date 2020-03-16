use core::num::NonZeroU32;
use crate::traits::Word;

pub type Cow<'a, T> = crate::generic::Cow<'a, T, u32>;

impl Word for u32 {
    type NonZero = NonZeroU32;

    #[inline]
    fn from(word: usize) -> u32 {
        word as u32
    }

    #[inline]
    fn nonzero_from(word: usize) -> Option<NonZeroU32> {
    	if word > u32::max_value() as usize {
    		panic!("Capacity out of bounds");
    	}

        NonZeroU32::new(word as u32)
    }

    #[inline]
    fn into(self) -> usize {
    	self as usize
    }

    #[inline]
    fn nonzero_into(word: NonZeroU32) -> usize {
    	word.get() as usize
    }
}