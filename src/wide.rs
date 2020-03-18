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

#[cfg(test)]
mod tests {
    use super::Cow;

    #[test]
    fn stress_test_owned() {
        let mut expected = String::from("Hello... ");
        let mut cow: Cow<str> = Cow::borrowed("Hello... ");

        for i in 0..1024 {
            if i % 3 == 0 {
                cow = cow.clone();
            }

            let mut owned = cow.into_owned();

            expected.push_str("Hello?.. ");
            owned.push_str("Hello?.. ");

            cow = owned.into();
        }

        assert_eq!(expected, cow.into_owned());
    }
}