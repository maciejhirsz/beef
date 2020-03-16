//! # beef
//!
//! Alternative implementation of `Cow` that's more compact in memory.
//!
//! **[Changelog](https://github.com/maciejhirsz/beef/releases) -**
//! **[Cargo](https://crates.io/crates/beef) -**
//! **[Repository](https://github.com/maciejhirsz/beef)**
//!
//! ```rust
//! # fn main() {
//! use beef::Cow;
//!
//! let borrowed = Cow::borrowed("Hello");
//! let owned = Cow::from(String::from("World"));
//!
//! assert_eq!(
//!     format!("{} {}!", borrowed, owned),
//!     "Hello World!",
//! );
//!
//! // beef::Cow is 3 word sized, while std::borrow::Cow is 4 word sized
//! assert!(std::mem::size_of::<Cow<str>>() < std::mem::size_of::<std::borrow::Cow<str>>());
//! # }
//! ```
#![cfg_attr(feature = "const_fn", feature(const_fn))]
#![warn(missing_docs)]
#![cfg_attr(not(test), no_std)]
extern crate alloc;

use alloc::borrow::{Borrow, Cow as StdCow, ToOwned};
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::num::NonZeroUsize;
use core::ptr::{slice_from_raw_parts, NonNull};

/// A clone-on-write smart pointer, mostly compatible with [`std::borrow::Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html).
#[derive(Eq)]
pub struct Cow<'a, T: Beef + ?Sized + 'a> {
    ptr: NonNull<T::PointerT>,
    len: usize,
    capacity: Option<NonZeroUsize>,
    marker: PhantomData<&'a T>,
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

    fn len(&self) -> usize;

    fn ref_from_parts(ptr: NonNull<Self::PointerT>, len: usize) -> *const Self;

    /// Convert `T::Owned` to `NonNull<T>` and capacity.
    /// Return `None` for `0` capacity.
    fn owned_into_parts(owned: Self::Owned) -> (NonNull<Self::PointerT>, usize, Option<NonZeroUsize>);

    /// Rebuild `T::Owned` from `NonNull<T>` and `capacity`. This can be done by the likes
    /// of [`Vec::from_raw_parts`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.from_raw_parts).
    unsafe fn owned_from_parts(ptr: NonNull<Self::PointerT>, len: usize, capacity: NonZeroUsize) -> Self::Owned;
}

unsafe impl Beef for str {
    type PointerT = u8;

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn ref_from_parts(ptr: NonNull<u8>, len: usize) -> *const str {
        slice_from_raw_parts(ptr.as_ptr(), len) as *const str
    }

    #[inline]
    fn owned_into_parts(owned: String) -> (NonNull<u8>, usize, Option<NonZeroUsize>) {
        // Convert to `String::into_raw_parts` once stabilized
        let mut owned = ManuallyDrop::new(owned);

        (
            unsafe { NonNull::new_unchecked(owned.as_mut_ptr()) },
            owned.len(),
            NonZeroUsize::new(owned.capacity()),
        )
    }

    #[inline]
    unsafe fn owned_from_parts(ptr: NonNull<u8>, len: usize, capacity: NonZeroUsize) -> String {
        String::from_utf8_unchecked(Vec::from_raw_parts(
            ptr.as_ptr(),
            len,
            capacity.get(),
        ))
    }
}

unsafe impl<T: Clone> Beef for [T] {
    type PointerT = T;

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn ref_from_parts(ptr: NonNull<T>, len: usize) -> *const [T] {
        slice_from_raw_parts(ptr.as_ptr(), len)
    }

    #[inline]
    fn owned_into_parts(owned: Vec<T>) -> (NonNull<T>, usize, Option<NonZeroUsize>) {
        // Convert to `Vec::into_raw_parts` once stabilized
        let mut owned = ManuallyDrop::new(owned);
        (
            unsafe { NonNull::new_unchecked(owned.as_mut_ptr()) },
            owned.len(),
            NonZeroUsize::new(owned.capacity()),
        )
    }

    #[inline]
    unsafe fn owned_from_parts(ptr: NonNull<T>, len: usize, capacity: NonZeroUsize) -> Vec<T> {
        Vec::from_raw_parts(
            ptr.as_ptr(),
            len,
            capacity.get(),
        )
    }
}

impl<B> Cow<'_, B>
where
    B: Beef + ?Sized,
{
    /// Owned data.
    #[inline]
    pub fn owned(val: B::Owned) -> Self {
        let (ptr, len, capacity) = B::owned_into_parts(val);

        Cow {
            ptr,
            len,
            capacity,
            marker: PhantomData,
        }
    }
}

impl<'a, T> Cow<'a, T>
where
    T: Beef + ?Sized,
{
    // // This requires nightly:
    // // https://github.com/rust-lang/rust/issues/57563
    // /// Owned data.
    // #[cfg(feature = "const_fn")]
    // #[inline]
    // pub const fn borrowed(val: &'a T) -> Self {
    //     Cow {
    //         // A note on soundness:
    //         //
    //         // We are casting *const T to *mut T, however for all borrowed values
    //         // this raw pointer is only ever dereferenced back to &T.
    //         ptr: unsafe { NonNull::new_unchecked(val as *const T as *mut T) },
    //         capacity: None,
    //         marker: PhantomData,
    //     }
    // }

    #[cfg(not(feature = "const_fn"))]
    #[inline]
    pub fn borrowed(val: &'a T) -> Self {
        Cow {
            // A note on soundness:
            //
            // We are casting *const T to *mut T, however for all borrowed values
            // this raw pointer is only ever dereferenced back to &T.
            ptr: unsafe { NonNull::new_unchecked(val as *const T as *mut T).cast() },
            len: val.len(),
            capacity: None,
            marker: PhantomData,
        }
    }

    /// Extracts the owned data.
    ///
    /// Clones the data if it is not already owned.
    #[inline]
    pub fn into_owned(self) -> T::Owned {
        let cow = ManuallyDrop::new(self);

        match cow.capacity {
            Some(capacity) => unsafe { T::owned_from_parts(cow.ptr, cow.len, capacity) },
            None => unsafe { &*T::ref_from_parts(cow.ptr, cow.len) }.to_owned(),
        }
    }

    /// Internal convenience method for casting `ptr` into a `&T`
    #[inline]
    fn borrow(&self) -> &T {
        unsafe { &*T::ref_from_parts(self.ptr, self.len) }
    }
}

impl<T> Hash for Cow<'_, T>
where
    T: Hash + Beef + ?Sized,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.borrow().hash(state)
    }
}

impl<'a, T> From<&'a T> for Cow<'a, T>
where
    T: Beef + ?Sized,
{
    #[inline]
    fn from(val: &'a T) -> Self {
        Cow::borrowed(val)
    }
}

impl From<String> for Cow<'_, str> {
    #[inline]
    fn from(s: String) -> Self {
        Cow::owned(s)
    }
}

impl<T> From<Vec<T>> for Cow<'_, [T]>
where
    T: Clone,
{
    #[inline]
    fn from(v: Vec<T>) -> Self {
        Cow::owned(v)
    }
}

impl<T> Drop for Cow<'_, T>
where
    T: Beef + ?Sized,
{
    #[inline]
    fn drop(&mut self) {
        if let Some(capacity) = self.capacity {
            unsafe { T::owned_from_parts(self.ptr, self.len, capacity) };
        }
    }
}

impl<'a, T> Clone for Cow<'a, T>
where
    T: Beef + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        match self.capacity {
            Some(_) => Cow::owned(self.borrow().to_owned()),
            None => Cow { ..*self },
        }
    }
}

impl<T> core::ops::Deref for Cow<'_, T>
where
    T: Beef + ?Sized,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.borrow()
    }
}

impl<T> AsRef<T> for Cow<'_, T>
where
    T: Beef + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &T {
        self.borrow()
    }
}

impl<T> Borrow<T> for Cow<'_, T>
where
    T: Beef + ?Sized,
{
    #[inline]
    fn borrow(&self) -> &T {
        self.borrow()
    }
}

impl<'a, T> From<StdCow<'a, T>> for Cow<'a, T>
where
    T: Beef + ?Sized,
{
    #[inline]
    fn from(stdcow: StdCow<'a, T>) -> Self {
        match stdcow {
            StdCow::Borrowed(v) => Self::borrowed(v),
            StdCow::Owned(v) => Self::owned(v),
        }
    }
}

impl<'a, T> From<Cow<'a, T>> for StdCow<'a, T>
where
    T: Beef + ?Sized,
{
    #[inline]
    fn from(cow: Cow<'a, T>) -> Self {
        let cow = ManuallyDrop::new(cow);

        match cow.capacity {
            Some(capacity) => StdCow::Owned(unsafe { T::owned_from_parts(cow.ptr, cow.len, capacity) }),
            None => StdCow::Borrowed(unsafe { &*T::ref_from_parts(cow.ptr, cow.len) }),
        }
    }
}

impl<T, U> PartialEq<U> for Cow<'_, T>
where
    T: Beef + PartialEq + ?Sized,
    U: AsRef<T> + ?Sized,
{
    #[inline]
    fn eq(&self, other: &U) -> bool {
        self.borrow() == other.as_ref()
    }
}

impl PartialEq<Cow<'_, str>> for str {
    #[inline]
    fn eq(&self, other: &Cow<str>) -> bool {
        self == other.borrow()
    }
}

impl PartialEq<Cow<'_, str>> for &str {
    #[inline]
    fn eq(&self, other: &Cow<str>) -> bool {
        *self == other.borrow()
    }
}

impl PartialEq<Cow<'_, str>> for String {
    #[inline]
    fn eq(&self, other: &Cow<str>) -> bool {
        self == other.borrow()
    }
}

impl<T> PartialEq<Cow<'_, [T]>> for [T]
where
    T: Clone + PartialEq,
    [T]: Beef,
{
    #[inline]
    fn eq(&self, other: &Cow<[T]>) -> bool {
        self == other.borrow()
    }
}

impl<T> PartialEq<Cow<'_, [T]>> for &[T]
where
    T: Clone + PartialEq,
    [T]: Beef,
{
    #[inline]
    fn eq(&self, other: &Cow<[T]>) -> bool {
        *self == other.borrow()
    }
}

impl<T> PartialEq<Cow<'_, [T]>> for Vec<T>
where
    T: Clone + PartialEq,
    [T]: Beef,
{
    #[inline]
    fn eq(&self, other: &Cow<[T]>) -> bool {
        &self[..] == other.borrow()
    }
}

impl<T: Beef + fmt::Debug + ?Sized> fmt::Debug for Cow<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.borrow().fmt(f)
    }
}

impl<T: Beef + fmt::Display + ?Sized> fmt::Display for Cow<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.borrow().fmt(f)
    }
}

unsafe impl<T: Beef + Sync + ?Sized> Sync for Cow<'_, T> {}
unsafe impl<T: Beef + Send + ?Sized> Send for Cow<'_, T> {}

#[cfg(test)]
mod tests {
    use super::Cow;

    #[test]
    fn borrowed_str() {
        let s = "Hello World";
        let c = Cow::borrowed(s);

        assert_eq!(s, c);
        assert_eq!(s, c.as_ref());
        assert_eq!(s, &*c);
    }

    #[test]
    fn owned_string() {
        let s = String::from("Hello World");
        let c: Cow<str> = Cow::owned(s.clone());

        assert_eq!(s, c);
    }

    #[test]
    fn into_owned() {
        let hello = "Hello World";
        let borrowed = Cow::borrowed(hello);
        let owned: Cow<str> = Cow::owned(String::from(hello));

        assert_eq!(borrowed.into_owned(), hello);
        assert_eq!(owned.into_owned(), hello);
    }

    #[test]
    fn borrowed_slice() {
        let s: &[_] = &[1, 2, 42];
        let c = Cow::borrowed(s);

        assert_eq!(s, c);
        assert_eq!(s, c.as_ref());
        assert_eq!(s, &*c);
    }

    #[test]
    fn owned_slice() {
        let s = vec![1, 2, 42];
        let c: Cow<[_]> = Cow::owned(s.clone());

        assert_eq!(s, c);
    }

    #[test]
    fn into_owned_vec() {
        let hello: &[u8] = b"Hello World";
        let borrowed = Cow::borrowed(hello);
        let owned: Cow<[u8]> = Cow::owned(hello.to_vec());

        assert_eq!(borrowed.into_owned(), hello);
        assert_eq!(owned.into_owned(), hello);
    }

    #[test]
    fn hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let slice = "Hello World!";
        let borrowed = Cow::borrowed(slice);
        let owned: Cow<str> = Cow::owned(slice.to_owned());

        let hash1 = {
            let mut hasher = DefaultHasher::default();

            slice.hash(&mut hasher);

            hasher.finish()
        };

        let hash2 = {
            let mut hasher = DefaultHasher::default();

            borrowed.hash(&mut hasher);

            hasher.finish()
        };

        let hash3 = {
            let mut hasher = DefaultHasher::default();

            owned.hash(&mut hasher);

            hasher.finish()
        };

        assert_eq!(hash1, hash2);
        assert_eq!(hash1, hash3);
        assert_eq!(hash2, hash3);
    }

    #[test]
    fn from_std_cow() {
        let std = std::borrow::Cow::Borrowed("Hello World");
        let beef = Cow::from(std.clone());

        assert_eq!(&*std, &*beef);
    }
}
