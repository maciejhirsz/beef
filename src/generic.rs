use alloc::borrow::{Borrow, Cow as StdCow};
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use crate::traits::{Beef, Capacity};

/// A clone-on-write smart pointer, mostly compatible with [`std::borrow::Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html).
// #[derive(Eq)]
pub struct Cow<'a, T: Beef + ?Sized + 'a, U: Capacity> {
    inner: NonNull<T>,
    capacity: U,
    marker: PhantomData<&'a T>,
}

impl<T, U> Cow<'_, T, U>
where
    T: Beef + ?Sized,
    U: Capacity,
{
    /// Owned data.
    #[inline]
    pub fn owned(val: T::Owned) -> Self {
        let (inner, capacity) = T::owned_into_parts(val);

        Cow {
            inner,
            capacity,
            marker: PhantomData,
        }
    }
}

impl<'a, T, U> Cow<'a, T, U>
where
    T: Beef + ?Sized,
    U: Capacity,
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
    //         inner: unsafe { NonNull::new_unchecked(val as *const T as *mut T) },
    //         capacity: None,
    //         marker: PhantomData,
    //     }
    // }

    #[cfg(not(feature = "const_fn"))]
    #[inline]
    pub fn borrowed(val: &'a T) -> Self {
    	let (inner, capacity) = T::ref_into_parts(val);

        Cow {
            // A note on soundness:
            //
            // We are casting *const T to *mut T, however for all borrowed values
            // this raw pointer is only ever dereferenced back to &T.
            inner,
            capacity,
            marker: PhantomData,
        }
    }

    /// Extracts the owned data.
    ///
    /// Clones the data if it is not already owned.
    #[inline]
    pub fn into_owned(self) -> T::Owned {
        let cow = ManuallyDrop::new(self);

        match cow.capacity() {
            Some(capacity) => unsafe { T::owned_from_parts::<U>(cow.inner, capacity) },
            None => unsafe { &*cow.inner.as_ptr() }.to_owned(),
        }
    }

    /// Internal convenience method for casting `inner` into a `&T`
    #[inline]
    fn borrow(&self) -> &T {
        unsafe { &*self.inner.as_ptr() }
    }

    fn capacity(&self) -> Option<U::NonZero> {
    	U::maybe(T::len(self.inner.as_ptr()), self.capacity)
    }
}

impl<T, U> Hash for Cow<'_, T, U>
where
    T: Hash + Beef + ?Sized,
    U: Capacity,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.borrow().hash(state)
    }
}

impl<'a, T, U> From<&'a T> for Cow<'a, T, U>
where
    T: Beef + ?Sized,
    U: Capacity,
{
    #[inline]
    fn from(val: &'a T) -> Self {
        Cow::borrowed(val)
    }
}

impl<U> From<String> for Cow<'_, str, U>
where
	U: Capacity,
{
    #[inline]
    fn from(s: String) -> Self {
        Cow::owned(s)
    }
}

impl<T, U> From<Vec<T>> for Cow<'_, [T], U>
where
    T: Clone,
    U: Capacity,
{
    #[inline]
    fn from(v: Vec<T>) -> Self {
        Cow::owned(v)
    }
}

impl<T, U> Drop for Cow<'_, T, U>
where
    T: Beef + ?Sized,
    U: Capacity,
{
    #[inline]
    fn drop(&mut self) {
        if let Some(capacity) = self.capacity() {
            unsafe { T::owned_from_parts::<U>(self.inner, capacity) };
        }
    }
}

impl<'a, T, U> Clone for Cow<'a, T, U>
where
    T: Beef + ?Sized,
    U: Capacity,
{
    #[inline]
    fn clone(&self) -> Self {
        match self.capacity() {
            Some(_) => Cow::owned(self.borrow().to_owned()),
            None => Cow { ..*self },
        }
    }
}

impl<T, U> core::ops::Deref for Cow<'_, T, U>
where
    T: Beef + ?Sized,
    U: Capacity,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.borrow()
    }
}

impl<T, U> AsRef<T> for Cow<'_, T, U>
where
    T: Beef + ?Sized,
    U: Capacity,
{
    #[inline]
    fn as_ref(&self) -> &T {
        self.borrow()
    }
}

impl<T, U> Borrow<T> for Cow<'_, T, U>
where
    T: Beef + ?Sized,
    U: Capacity,
{
    #[inline]
    fn borrow(&self) -> &T {
        self.borrow()
    }
}

impl<'a, T, U> From<StdCow<'a, T>> for Cow<'a, T, U>
where
    T: Beef + ?Sized,
    U: Capacity,
{
    #[inline]
    fn from(stdcow: StdCow<'a, T>) -> Self {
        match stdcow {
            StdCow::Borrowed(v) => Self::borrowed(v),
            StdCow::Owned(v) => Self::owned(v),
        }
    }
}

impl<'a, T, U> From<Cow<'a, T, U>> for StdCow<'a, T>
where
    T: Beef + ?Sized,
    U: Capacity,
{
    #[inline]
    fn from(cow: Cow<'a, T, U>) -> Self {
        let cow = ManuallyDrop::new(cow);

        match cow.capacity() {
            Some(capacity) => StdCow::Owned(unsafe { T::owned_from_parts::<U>(cow.inner, capacity) }),
            None => StdCow::Borrowed(unsafe { &*cow.inner.as_ptr() }),
        }
    }
}

impl<T, U, V> PartialEq<V> for Cow<'_, T, U>
where
    T: Beef + PartialEq + ?Sized,
    U: Capacity,
    V: AsRef<T> + ?Sized,
{
    #[inline]
    fn eq(&self, other: &V) -> bool {
        self.borrow() == other.as_ref()
    }
}

impl<U> PartialEq<Cow<'_, str, U>> for str
where
    U: Capacity,
{
    #[inline]
    fn eq(&self, other: &Cow<str, U>) -> bool {
        self == other.borrow()
    }
}

impl<U> PartialEq<Cow<'_, str, U>> for &str
where
    U: Capacity,
{
    #[inline]
    fn eq(&self, other: &Cow<str, U>) -> bool {
        *self == other.borrow()
    }
}

impl<U> PartialEq<Cow<'_, str, U>> for String
where
    U: Capacity,
{
    #[inline]
    fn eq(&self, other: &Cow<str, U>) -> bool {
        self == other.borrow()
    }
}

impl<T, U> PartialEq<Cow<'_, [T], U>> for [T]
where
    T: Clone + PartialEq,
    [T]: Beef,
    U: Capacity,
{
    #[inline]
    fn eq(&self, other: &Cow<[T], U>) -> bool {
        self == other.borrow()
    }
}

impl<T, U> PartialEq<Cow<'_, [T], U>> for &[T]
where
    T: Clone + PartialEq,
    [T]: Beef,
    U: Capacity,
{
    #[inline]
    fn eq(&self, other: &Cow<[T], U>) -> bool {
        *self == other.borrow()
    }
}

impl<T, U> PartialEq<Cow<'_, [T], U>> for Vec<T>
where
    T: Clone + PartialEq,
    [T]: Beef,
    U: Capacity,
{
    #[inline]
    fn eq(&self, other: &Cow<[T], U>) -> bool {
        &self[..] == other.borrow()
    }
}

impl<T, U> fmt::Debug for Cow<'_, T, U>
where
    T: Beef + fmt::Debug + ?Sized,
    U: Capacity,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.borrow().fmt(f)
    }
}

impl<T, U> fmt::Display for Cow<'_, T, U>
where
    T: Beef + fmt::Display + ?Sized,
    U: Capacity,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.borrow().fmt(f)
    }
}

unsafe impl<T: Beef + Sync + ?Sized, U: Capacity> Sync for Cow<'_, T, U> {}
unsafe impl<T: Beef + Send + ?Sized, U: Capacity> Send for Cow<'_, T, U> {}