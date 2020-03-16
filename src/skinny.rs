//! Namespace containing the 2-word `Cow` implementation.

use core::ptr::slice_from_raw_parts_mut;
use crate::traits::Capacity;

/// Faster, 2-word `Cow`. This version is available only on 64-bit architecture,
/// and it puts both capacity and length together in a fat pointer. Both length and capacity
/// is limited to 32 bits.
///
/// # Panics
///
/// `Cow::owned` will panic if capacity is larger than overflows `u32::max_size()`. Use the
/// top level `beef::Cow` if you wish to avoid this problem.
pub type Cow<'a, T> = crate::generic::Cow<'a, T, Cursed>;

mod internal {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Cursed;
}
use internal::Cursed;

const MASK_LO: usize = u32::max_value() as usize;
const MASK_HI: usize = !u32::max_value() as usize;

impl Capacity for Cursed {
    type NonZero = Cursed;

    #[inline]
    fn empty<T>(ptr: *mut T, len: usize) -> (*mut [T], Cursed) {
        (slice_from_raw_parts_mut(ptr, len & MASK_LO), Cursed)
    }

    #[inline]
    fn store<T>(ptr: *mut T, len: usize, capacity: usize) -> (*mut [T], Cursed) {
        if capacity > MASK_LO {
            panic!("beef::skinny::Cow: Capacity out of bounds");
        }

        (
            slice_from_raw_parts_mut(
                ptr,
                (len & MASK_LO) | ((capacity & MASK_HI) << 32),
            ),
            Cursed,
        )
    }

    #[inline]
    fn unpack(len: usize, _: Cursed) -> (usize, usize) {
        (len & MASK_LO, (len & MASK_HI) >> 32)
    }

    #[inline]
    fn maybe(len: usize, _: Cursed) -> Option<Cursed> {
        if len & MASK_HI != 0 {
            Some(Cursed)
        } else {
            None
        }
    }
}