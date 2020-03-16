//! This module contains the actual, albeit generic, implementaiton of the `Cow`,
//! and the traits that are available to it.

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
///
/// This type is using a generic `U: Capacity`. Use either `beef::Cow` or `beef::skinny::Cow` in your code.
#[derive(Eq)]
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use beef::Cow;
    ///
    /// let owned: Cow<str> = Cow::owned("I own my content".to_string());
    /// ```
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
    /// Borrowed data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use beef::Cow;
    ///
    /// let borrowed: Cow<str> = Cow::borrowed("I'm just a borrow");
    /// ```
    #[inline]
    pub fn borrowed(val: &'a T) -> Self {
        let (inner, capacity) = T::ref_into_parts(val);

        Cow {
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

impl<A, B, U, V> PartialEq<Cow<'_, B, V>> for Cow<'_, A, U>
where
    A: Beef + ?Sized,
    B: Beef + ?Sized,
    U: Capacity,
    V: Capacity,
    A: PartialEq<B>,
{
    fn eq(&self, other: &Cow<B, V>) -> bool {
        self.borrow() == other.borrow()
    }
}

macro_rules! impl_eq {
    ($($(@for< $bounds:tt >)? $inner:ty => $([$($deref:tt)+])? <$with:ty>,)*) => {$(
        impl<U $(, $bounds)*> PartialEq<$with> for Cow<'_, $inner, U>
        where
            U: Capacity,
            $( $bounds: Clone + PartialEq, )*
        {
            #[inline]
            fn eq(&self, other: &$with) -> bool {
                self.borrow() == $($($deref)*)* other
            }
        }

        impl<U $(, $bounds)*> PartialEq<Cow<'_, $inner, U>> for $with
        where
            U: Capacity,
            $( $bounds: Clone + PartialEq, )*
        {
            #[inline]
            fn eq(&self, other: &Cow<$inner, U>) -> bool {
                $($($deref)*)* self == other.borrow()
            }
        }
    )*};
}

impl_eq! {
    str => <str>,
    str => [*]<&str>,
    str => <String>,
    @for<T> [T] => <[T]>,
    @for<T> [T] => [*]<&[T]>,
    @for<T> [T] => [&**]<Vec<T>>,
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