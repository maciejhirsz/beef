use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use core::mem::ManuallyDrop;
use core::ptr::{slice_from_raw_parts, NonNull};

pub(crate) use internal::Word;

mod internal {
    use super::*;

    pub trait Word: Into<usize> + Copy {
        type NonZero: Into<usize> + Copy;

        fn from(word: usize) -> Self;

        fn nonzero_from(word: usize) -> Option<Self::NonZero>;
    }
}

/// Helper trait required by `Cow<T>` to extract capacity of owned
/// variant of `T`, and manage conversions.
///
/// This can be only implemented on types that match requirements:
///
/// + `T::Owned` has a `capacity`, which is an extra word that is absent in `T`.
/// + `T::Owned` with `capacity` of `0` does not allocate memory.
/// + `T::Owned` can be reconstructed from `*mut T` borrowed out of it, plus capacity.
pub unsafe trait Beef: ToOwned {
    type PointerT;

    fn len<U: Word>(&self) -> U;

    fn ref_from_parts<U>(ptr: NonNull<Self::PointerT>, len: U) -> *const Self
    where
        U: Word;

    /// Convert `T::Owned` to `NonNull<T>` and capacity.
    /// Return `None` for `0` capacity.
    fn owned_into_parts<U>(owned: Self::Owned) -> (NonNull<Self::PointerT>, U, Option<U::NonZero>)
    where
        U: Word;

    /// Rebuild `T::Owned` from `NonNull<T>` and `capacity`. This can be done by the likes
    /// of [`Vec::from_raw_parts`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.from_raw_parts).
    unsafe fn owned_from_parts<U>(ptr: NonNull<Self::PointerT>, len: U, capacity: U::NonZero) -> Self::Owned
    where
        U: Word;
}

unsafe impl Beef for str {
    type PointerT = u8;

    #[inline]
    fn len<U: Word>(&self) -> U {
        U::from(self.len())
    }

    #[inline]
    fn ref_from_parts<U>(ptr: NonNull<u8>, len: U) -> *const str
    where
        U: Word,
    {
        slice_from_raw_parts(ptr.as_ptr(), len.into()) as *const str
    }

    #[inline]
    fn owned_into_parts<U>(owned: String) -> (NonNull<u8>, U, Option<U::NonZero>)
    where
        U: Word,
    {
        // Convert to `String::into_raw_parts` once stabilized
        let mut owned = ManuallyDrop::new(owned);

        (
            unsafe { NonNull::new_unchecked(owned.as_mut_ptr()) },
            U::from(owned.len()),
            U::nonzero_from(owned.capacity()),
        )
    }

    #[inline]
    unsafe fn owned_from_parts<U>(ptr: NonNull<u8>, len: U, capacity: U::NonZero) -> String
    where
        U: Word,
    {
        String::from_utf8_unchecked(Vec::from_raw_parts(
            ptr.as_ptr(),
            len.into(),
            capacity.into(),
        ))
    }
}

unsafe impl<T: Clone> Beef for [T] {
    type PointerT = T;

    #[inline]
    fn len<U: Word>(&self) -> U {
        U::from(self.len())
    }

    #[inline]
    fn ref_from_parts<U>(ptr: NonNull<T>, len: U) -> *const [T]
    where
        U: Word,
    {
        slice_from_raw_parts(ptr.as_ptr(), len.into())
    }

    #[inline]
    fn owned_into_parts<U>(owned: Vec<T>) -> (NonNull<T>, U, Option<U::NonZero>)
    where
        U: Word,
    {
        // Convert to `Vec::into_raw_parts` once stabilized
        let mut owned = ManuallyDrop::new(owned);
        (
            unsafe { NonNull::new_unchecked(owned.as_mut_ptr()) },
            U::from(owned.len()),
            U::nonzero_from(owned.capacity()),
        )
    }

    #[inline]
    unsafe fn owned_from_parts<U>(ptr: NonNull<T>, len: U, capacity: U::NonZero) -> Vec<T>
    where
        U: Word,
    {
        Vec::from_raw_parts(
            ptr.as_ptr(),
            len.into(),
            capacity.into(),
        )
    }
}