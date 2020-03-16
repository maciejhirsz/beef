use core::num::NonZeroUsize;
use core::ptr::slice_from_raw_parts_mut;
use crate::traits::Capacity;

pub type Cow<'a, T> = crate::generic::Cow<'a, T, Option<NonZeroUsize>>;

impl Capacity for Option<NonZeroUsize> {
    type NonZero = NonZeroUsize;

    #[inline]
    fn empty<T>(ptr: *mut T, len: usize) -> (*mut [T], Self) {
        (slice_from_raw_parts_mut(ptr, len), None)
    }

    #[inline]
    fn store<T>(ptr: *mut T, len: usize, capacity: usize) -> (*mut [T], Self) {
        (slice_from_raw_parts_mut(ptr, len), NonZeroUsize::new(capacity))
    }

    #[inline]
    fn unpack(len: usize, capacity: NonZeroUsize) -> (usize, usize) {
        (len, capacity.get())
    }

    #[inline]
    fn maybe(_: usize, capacity: Option<NonZeroUsize>) -> Option<NonZeroUsize> {
        capacity
    }
}