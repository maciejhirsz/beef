//! Namespace containing the 2-word `Cow` implementation.

use core::ptr::slice_from_raw_parts;
use crate::traits::Capacity;

/// Faster, 2-word `Cow`. This version is available only on 64-bit architecture,
/// and it puts both capacity and length together in a fat pointer. Both length and capacity
/// is limited to 32 bits.
///
/// # Panics
///
/// [`Cow::owned`](../generic/struct.Cow.html#method.owned) will panic if capacity is larger than `u32::max_size()`. Use the
/// top level `beef::Cow` if you wish to avoid this problem.
pub type Cow<'a, T> = crate::generic::Cow<'a, T, Lean>;

mod internal {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Lean;
}
use internal::Lean;

const MASK_LO: usize = u32::max_value() as usize;
const MASK_HI: usize = !(u32::max_value() as usize);

impl Capacity for Lean {
    type Field = Lean;
    type NonZero = Lean;

    #[inline]
    fn as_ref<T>(ptr: *const T, len: usize) -> *const [T] {
        slice_from_raw_parts(ptr, len & MASK_LO)
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

        (
            (len & MASK_LO) | ((capacity & MASK_LO) << 32),
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

#[cfg(test)]
mod tests {
    use super::Cow;

    // Lean `Cow` creates and internally stores fat pointers with invalid len,
    // this makes Miri complain about memory access.
    #[test]
    fn stress_test_owned() {
        let mut expected = String::from("Hello... ");
        let mut cow: Cow<str> = Cow::borrowed("Hello... ");

        for i in 0..1024 {
            if i % 3 == 0 {
                let old = cow;
                cow = old.clone();

                std::mem::drop(old);
            }

            let mut owned = cow.into_owned();

            expected.push_str("Hello?.. ");
            owned.push_str("Hello?.. ");

            cow = owned.into();
        }

        assert_eq!(expected, cow.into_owned());
    }

    #[test]
    fn borrowed_str() {
        let s = "Hello World";
        let c = Cow::borrowed(s);

        assert_eq!(s, c);
        assert_eq!(s, c.as_ref());
        assert_eq!(s, &*c);
    }

    #[test]
    fn into_owned() {
        let hello = "Hello World";
        let borrowed = Cow::borrowed(hello);

        assert_eq!(borrowed.into_owned(), hello);
    }

    #[test]
    fn borrowed_slice() {
        let s: &[_] = &[1, 2, 42];
        let c = Cow::borrowed(s);

        assert_eq!(s, c);
        assert_eq!(s, c.as_ref());
        assert_eq!(s, &*c);
    }

}