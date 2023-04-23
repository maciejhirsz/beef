//! Namespace containing the 2-word `Cow` implementation.

use crate::traits::InternalCapacity;

/// Faster, 2-word `Cow`.
///
/// This version puts both capacity and length together in a fat pointer.
/// Both length and capacity are stored evenly through out the target pointer's width.
///
/// For example, on a machine with a pointer width of 64. both length and capacity would occupy 32 bits.
///
/// # Panics
///
/// [`Cow::owned`](../generic/struct.Cow.html#method.owned) will panic if capacity is larger than half the pointer width.
/// Use the top level `beef::Cow` if you wish to avoid this problem.
pub type Cow<'a, T> = crate::generic::Cow<'a, T, Lean>;

pub(crate) mod internal {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Lean;
}
use internal::Lean;

const POINTER_SIZE: usize = core::mem::size_of::<usize>() * 8;
const MASK_LO: usize = usize::MAX >> (POINTER_SIZE / 2);
const MASK_HI: usize = !MASK_LO;

impl Lean {
    #[inline]
    pub const fn mask_len(len: usize) -> usize {
        len & MASK_LO
    }
}

impl InternalCapacity for Lean {
    type Field = Lean;
    type NonZero = Lean;

    #[inline]
    fn len(fat: usize) -> usize {
        fat & MASK_LO
    }

    #[inline]
    fn empty(len: usize) -> (usize, Lean) {
        (len & MASK_LO, Lean)
    }

    #[inline]
    fn store(len: usize, capacity: usize) -> (usize, Lean) {
        if capacity & MASK_HI != 0 {
            panic!("beef::lean::Cow: Capacity out of bounds");
        }

        let fat = ((capacity & MASK_LO) << 32) | (len & MASK_LO);

        (fat, Lean)
    }

    #[inline]
    fn unpack(fat: usize, _: Lean) -> (usize, usize) {
        (fat & MASK_LO, (fat & MASK_HI) >> 32)
    }

    #[inline]
    fn maybe(fat: usize, _: Lean) -> Option<Lean> {
        if fat & MASK_HI != 0 {
            Some(Lean)
        } else {
            None
        }
    }
}
