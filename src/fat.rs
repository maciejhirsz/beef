use core::num::NonZeroUsize;
use core::ptr::slice_from_raw_parts_mut;
use crate::traits::Capacity;

/// Compact three word `Cow` that puts the ownership tag in capacity.
/// This is a type alias, for documentation see [`beef::generic::Cow`](./generic/struct.Cow.html).
pub type Cow<'a, T> = crate::generic::Cow<'a, T, Wide>;

mod internal {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Wide;
}
use internal::Wide;

impl Capacity for Wide {
    type Field = Option<NonZeroUsize>;
    type NonZero = NonZeroUsize;

    #[inline]
    fn as_ref<T>(ptr: *const [T]) -> *const [T] {
        ptr
    }

    #[inline]
    fn empty<T>(ptr: *mut T, len: usize) -> (*mut [T], Self::Field) {
        (slice_from_raw_parts_mut(ptr, len), None)
    }

    #[inline]
    fn store<T>(ptr: *mut T, len: usize, capacity: usize) -> (*mut [T], Self::Field) {
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

impl<'a, T> Cow<'a, T>
where
    T: crate::traits::internal::Beef + ?Sized
{
    // This requires nightly:
    // https://github.com/rust-lang/rust/issues/57563
    /// Borrowed data.
    ///
    /// Requires nightly. Currently not available for `beef::lean::Cow`.
    #[cfg(feature = "const_fn")]
    pub const fn const_borrow(val: &'a T) -> Self {
        Cow {
            // We are casting *const T to *mut T, however for all borrowed values
            // this raw pointer is only ever dereferenced back to &T.
            inner: unsafe { NonNull::new_unchecked(val as *const T as *mut T) },
            capacity: None,
            marker: PhantomData,
        }
    }
}