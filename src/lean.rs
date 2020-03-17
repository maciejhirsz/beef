//! Namespace containing the 2-word `Cow` implementation.

use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};
use crate::traits::Capacity;

/// Faster, 2-word `Cow`. This version is available only on 64-bit architecture,
/// and it puts both capacity and length together in a fat pointer. Both length and capacity
/// is limited to 32 bits.
///
/// # Panics
///
/// [`Cow::owned`](../generic/struct.Cow.html#method.owned) will panic if capacity is larger than overflows `u32::max_size()`. Use the
/// top level `beef::Cow` if you wish to avoid this problem.
pub type Cow<'a, T> = crate::generic::Cow<'a, T, Lean>;

mod internal {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Lean;
}
use internal::Lean;

const MASK_LO: usize = u32::max_value() as usize;
const MASK_HI: usize = !u32::max_value() as usize;

impl Capacity for Lean {
    type NonZero = Lean;

    #[inline]
    fn as_ref<T>(ptr: *const [T]) -> *const [T] {
        let len = unsafe { (*ptr).len() };

        slice_from_raw_parts(ptr as *mut T, len & MASK_LO)
    }

    #[inline]
    fn empty<T>(ptr: *mut T, len: usize) -> (*mut [T], Lean) {
        (slice_from_raw_parts_mut(ptr, len & MASK_LO), Lean)
    }

    #[inline]
    fn store<T>(ptr: *mut T, len: usize, capacity: usize) -> (*mut [T], Lean) {
        if capacity > MASK_LO {
            panic!("beef::lean::Cow: Capacity out of bounds");
        }

        (
            slice_from_raw_parts_mut(
                ptr,
                (len & MASK_LO) | ((capacity & MASK_HI) << 32),
            ),
            Lean,
        )
    }

    #[inline]
    fn unpack(len: usize, _: Lean) -> (usize, usize) {
        (len & MASK_LO, (len & MASK_HI) >> 32)
    }

    #[inline]
    fn maybe(len: usize, _: Lean) -> Option<Lean> {
        if len & MASK_HI != 0 {
            Some(Lean)
        } else {
            None
        }
    }
}