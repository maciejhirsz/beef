use core::num::NonZeroUsize;
use core::ptr::slice_from_raw_parts_mut;
use crate::traits::Capacity;

/// Compact three word `Cow` that puts the ownership tag in capacity.
/// This is a type alias, for documentation see [`beef::generic::Cow`](./generic/struct.Cow.html).
pub type Cow<'a, T> = crate::generic::Cow<'a, T, Wide>;

pub(crate) mod internal {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Wide;
}
use internal::Wide;

impl Capacity for Wide {
    type Field = Option<NonZeroUsize>;
    type NonZero = NonZeroUsize;

    #[inline]
    fn make_valid<T>(ptr: *const [()]) -> *const [T] {
        ptr as *const [T]
    }

    #[inline]
    fn empty<T>(ptr: *const [T]) -> (*mut [()], Self::Field) {
        (ptr as *const [()] as *mut [()], None)
    }

    #[inline]
    fn store<T>(ptr: *mut T, len: usize, capacity: usize) -> (*mut [()], Self::Field) {
        (slice_from_raw_parts_mut(ptr as *mut (), len), NonZeroUsize::new(capacity))
    }

    #[inline]
    fn unpack(fat: usize, capacity: NonZeroUsize) -> (usize, usize) {
        (fat, capacity.get())
    }

    #[inline]
    fn maybe(_: usize, capacity: Option<NonZeroUsize>) -> Option<NonZeroUsize> {
        capacity
    }
}
