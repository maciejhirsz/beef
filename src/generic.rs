//! This module contains the actual, albeit generic, implementaiton of the `Cow`,
//! and the traits that are available to it.

use alloc::borrow::{Borrow, Cow as StdCow, ToOwned};
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
#[cfg(feature = "const_deref")]
use core::marker::Destruct;
use core::marker::PhantomData;
#[cfg(not(feature = "const_deref"))]
use core::marker::Sized as Destruct;
use core::mem::ManuallyDrop;
use core::ptr::NonNull;

#[cfg(target_pointer_width = "64")]
use crate::lean::internal::Lean;
use crate::traits::{Beef, Capacity, Steak};
use crate::wide::internal::Wide;

/// A clone-on-write smart pointer, mostly compatible with [`std::borrow::Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html).
///
/// This type is using a generic `U: Capacity`. Use either [`beef::Cow`](../type.Cow.html) or [`beef::lean::Cow`](../lean/type.Cow.html) in your code.
pub struct Cow<'a, T: Beef + ?Sized + 'a, U: Capacity> {
    /// Pointer to data
    ptr: NonNull<T::PointerT>,

    /// This usize contains length, but it may contain other
    /// information pending on impl of `Capacity`, and must therefore
    /// always go through `U::len` or `U::unpack`
    fat: usize,

    /// Capacity field. For `beef::lean::Cow` this is 0-sized!
    cap: U::Field,

    /// Lifetime marker
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
        let (ptr, fat, cap) = T::owned_into_parts::<U>(val);

        Cow {
            ptr,
            fat,
            cap,
            marker: PhantomData,
        }
    }
}

impl<'a, T, U> Cow<'a, T, U>
where
    T: Beef + ?Sized,
    U: Capacity,
{
    // fixme: it should be unsafe :)
    const fn from_parts(ptr: NonNull<T::PointerT>, fat: usize, cap: U::Field) -> Self {
        Cow {
            ptr,
            fat,
            cap,
            marker: PhantomData,
        }
    }

    const fn to_parts(&self) -> (NonNull<T::PointerT>, usize, U::Field) {
        let Cow { ptr, fat, cap, .. } = self;
        (*ptr, *fat, *cap)
    }

    // fixme: required `borrowed_from_raw_parts` - it has bug with the processing of structs
    //  for example with: `Cow { ptr, fat, cap, marker }`
    crate::cfg_const_deref! {
        /// Borrowed data.
        ///
        /// # Example
        ///
        /// ```rust
        /// use beef::Cow;
        ///
        /// let borrowed: Cow<str> = Cow::borrowed("I'm just a borrow");
        /// ```
        ///
        #[inline]
        pub const fn borrowed(val: &'a T) -> Self
        where
            T: ~const Steak
        {
            let (ptr, fat, cap) = T::ref_into_parts::<U>(val);
            Self::from_parts(ptr, fat, cap)
        }
    }

    /// Extracts the owned data.
    ///
    /// Clones the data if it is not already owned.
    #[inline]
    pub fn into_owned(self) -> T::Owned {
        let cow = ManuallyDrop::new(self);

        match cow.capacity() {
            Some(capacity) => unsafe { T::owned_from_parts::<U>(cow.ptr, cow.fat, capacity) },
            None => unsafe { &*T::ref_from_parts::<U>(cow.ptr, cow.fat) }.to_owned(),
        }
    }

    crate::cfg_const_deref! {
        /// Extracts borrowed data if it is borrowed
        #[inline]
        pub const fn as_borrowed(&self) -> Option<&'_ T>
        where
            T: ~const Steak,
            U: ~const Capacity,
        {
            if self.is_borrowed() {
                Some(self.borrow())
            } else {
                None
            }
        }
    }

    crate::cfg_const_deref! {
        //crate::cfg_const_deref! {
        /// Extracts borrowed data.
        ///
        /// Panics: If the data is owned.
        #[inline]
        pub const fn unwrap_borrowed(self) -> &'a T
        where
            T: ~const Steak,
            U: ~const Capacity + 'a,
            Self: ~const Destruct,
        {
            if self.is_borrowed() {
                unsafe { &*T::ref_from_parts::<U>(self.ptr, self.fat) }
            } else {
                panic!("Can not turn owned beef::Cow into a borrowed value")
            }
        }
    }

    crate::cfg_const_deref! {
        /// Returns `true` if data is borrowed or had no capacity.
        ///
        /// # Example
        ///
        /// ```rust
        /// use beef::Cow;
        ///
        /// let borrowed: Cow<str> = Cow::borrowed("Borrowed");
        /// let no_capacity: Cow<str> = Cow::owned(String::new());
        /// let owned: Cow<str> = Cow::owned(String::from("Owned"));
        ///
        /// assert_eq!(borrowed.is_borrowed(), true);
        /// assert_eq!(no_capacity.is_borrowed(), true);
        /// assert_eq!(owned.is_borrowed(), false);
        /// ```
        #[inline]
        pub const fn is_borrowed(&self) -> bool
        where
            U: ~const Capacity,
        {
            self.capacity().is_none()
        }
    }

    crate::cfg_const_deref! {
        /// Returns `true` if data is owned and has non-0 capacity.
        ///
        /// # Example
        ///
        /// ```rust
        /// use beef::Cow;
        ///
        /// let borrowed: Cow<str> = Cow::borrowed("Borrowed");
        /// let no_capacity: Cow<str> = Cow::owned(String::new());
        /// let owned: Cow<str> = Cow::owned(String::from("Owned"));
        ///
        /// assert_eq!(borrowed.is_owned(), false);
        /// assert_eq!(no_capacity.is_owned(), false);
        /// assert_eq!(owned.is_owned(), true);
        /// ```
        #[inline]
        pub const fn is_owned(&self) -> bool
        where
            U: ~const Capacity,
        {
            self.capacity().is_some()
        }
    }

    crate::cfg_const_deref! {
        /// Internal convenience method for casting `ptr` into a `&T`
        #[inline]
        const fn borrow(&self) -> &T
        where
            T: ~const Steak
        {
            unsafe { &*T::ref_from_parts::<U>(self.ptr, self.fat) }
        }
    }

    crate::cfg_const_deref! {
        #[inline]
        const fn capacity(&self) -> Option<U::NonZero>
        where
            U: ~const Capacity,
        {
            U::maybe(self.fat, self.cap)
        }
    }

    #[allow(dead_code)]
    fn std_into_parts<S, C>(cow: StdCow<S>) -> (NonNull<S::PointerT>, usize, C::Field)
    where
        S: Beef + ?Sized,
        C: Capacity,
    {
        match cow {
            StdCow::Borrowed(borrowed) => S::ref_into_parts::<C>(borrowed),
            StdCow::Owned(owned) => S::owned_into_parts::<C>(owned),
        }
    }

    crate::cfg_const_deref! {
        #[allow(dead_code)]
        const fn std_into_parts_const<'std, S, C>(
            cow: StdCow<'std, S>,
        ) -> (NonNull<S::PointerT>, usize, C::Field)
        where
            S: ~const Steak + Beef + ?Sized,
            C: ~const Capacity,
            StdCow<'std, S>: ~const Destruct,
        {
            if let StdCow::Borrowed(borrowed) = cow {
                S::ref_into_parts::<C>(borrowed)
            } else {
                panic!("This message is useless: you can't create const `::owned`")
            }
        }
    }

    crate::cfg_const_deref! {
        #[allow(dead_code)]
        const fn ct_from_cow<'beef, S, C>(
            ptr: NonNull<S::PointerT>,
            fat: usize,
            cap: C::Field,
        ) -> Cow<'beef, S, C>
        where
            S: ~const Steak + Beef + ?Sized,
            C: ~const Capacity,
        {
            if C::maybe(fat, cap).is_none() {
                // SAFETY: later infer lifetime from `'a` but in this place
                // generic params can't use
                Cow::from_parts(ptr, fat, cap)
            } else {
                panic!("This message is useless: you can't create const `::owned`")
            }
        }
    }

    #[allow(dead_code)]
    fn rt_from_cow<'beef, S, C>(
        ptr: NonNull<S::PointerT>,
        fat: usize,
        cap: C::Field,
    ) -> Cow<'beef, S, C>
    where
        S: Steak + Beef + ?Sized,
        C: Capacity,
    {
        Cow::from_parts(ptr, fat, cap)
    }

    crate::cfg_const_deref! {
        #[inline]
        const fn from_std_cow(cow: StdCow<'a, T>) -> Cow<'a, T, U>
        where
            T: ~const Steak,
            U: ~const Capacity,
            StdCow<'a, T>: ~const Destruct,
        {
            #[cfg(not(feature = "const_deref"))]
            {
                // feature :(
                // rt_from_cow.call_once(std_into_parts(cow))

                let (ptr, fat, cap) = Self::std_into_parts::<T, U>(cow);
                Self::rt_from_cow(ptr, fat, cap)
            }

            // SAFETY: `ct` and `rt` must behave observably equivalent
            //      rt      |      rt
            // `::borrowed` | `::borrowed`
            //  `::owned`   | compile error
            // I think it is equivalent
            #[cfg(feature = "const_deref")]
            unsafe {
                core::intrinsics::const_eval_select(
                    Self::std_into_parts_const::<T, U>(cow),
                    Self::ct_from_cow::<T, U>,
                    Self::rt_from_cow::<T, U>,
                )
            }
        }
    }

    crate::cfg_const_deref! {
        #[inline]
        const fn into_std_cow(self) -> StdCow<'a, T>
        where
            T: ~const Steak,
            U: ~const Capacity,
        {
            #[allow(dead_code)]
            const fn ct_into_cow<'cow, S: Beef + ?Sized, C: Capacity>(
                ptr: NonNull<S::PointerT>,
                fat: usize,
                cap: C::Field,
            ) -> StdCow<'cow, S>
            where
                S: ~const Steak,
                C: ~const Capacity,
            {
                // fixme: at `match` `cfg_const_deref` is not work
                if C::maybe(fat, cap).is_none() {
                    StdCow::Borrowed(unsafe { &*S::ref_from_parts::<C>(ptr, fat) })
                } else {
                    panic!("This message is useless: you can't create const `::owned`")
                }
            }

            fn rt_into_cow<'cow, S: Beef + ?Sized, C: Capacity>(
                ptr: NonNull<S::PointerT>,
                fat: usize,
                cap: C::Field,
            ) -> StdCow<'cow, S> {
                if let Some(capacity) = C::maybe(fat, cap) {
                    StdCow::Owned(unsafe { S::owned_from_parts::<C>(ptr, fat, capacity) })
                } else {
                    StdCow::Borrowed(unsafe { &*S::ref_from_parts::<C>(ptr, fat) })
                }
            }

            #[cfg(not(feature = "const_deref"))]
            {
                let (ptr, fat, cap) = self.to_parts();
                rt_into_cow::<T, U>(ptr, fat, cap)
            }

            // SAFETY: `ct` and `rt` must behave observably equivalent
            //      rt      |      rt
            // `::borrowed` | `::borrowed`
            //  `::owned`   | compile error
            // I think it is equivalent
            #[cfg(feature = "const_deref")]
            unsafe {
                core::intrinsics::const_eval_select(
                    ManuallyDrop::new(self).to_parts(),
                    ct_into_cow::<T, U>,
                    rt_into_cow::<T, U>,
                )
            }
        }
    }
}

impl<'a> Cow<'a, str, Wide> {
    /// Borrowed data.
    ///
    /// This is functionally identical to [`borrow`](./generic/struct.Cow.html#method.borrow).
    /// We use impl specialization to allow this function to be `const`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use beef::Cow;
    ///
    /// const HELLO: Cow<str> = Cow::const_str("Hello");
    /// ```
    #[cfg_attr(
        feature = "const_deref",
        deprecated(note = "use Cow::borrowed() instead")
    )]
    pub const fn const_str(val: &'a str) -> Self {
        Cow {
            // We are casting *const T to *mut T, however for all borrowed values
            // this raw pointer is only ever dereferenced back to &T.
            ptr: unsafe { NonNull::new_unchecked(val.as_ptr() as *mut u8) },
            fat: val.len(),
            cap: None,
            marker: PhantomData,
        }
    }
}

#[cfg(target_pointer_width = "64")]
impl<'a> Cow<'a, str, Lean> {
    /// Borrowed data.
    ///
    /// This is functionally identical to [`borrow`](./generic/struct.Cow.html#method.borrow).
    /// We use impl specialization to allow this function to be `const`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use beef::lean::Cow;
    ///
    /// const HELLO: Cow<str> = Cow::const_str("Hello");
    /// ```
    #[cfg_attr(
        feature = "const_deref",
        deprecated(note = "use Cow::borrowed() instead")
    )]
    pub const fn const_str(val: &'a str) -> Self {
        Cow {
            // We are casting *const T to *mut T, however for all borrowed values
            // this raw pointer is only ever dereferenced back to &T.
            ptr: unsafe { NonNull::new_unchecked(val.as_ptr() as *mut u8) },
            fat: Lean::mask_len(val.len()),
            cap: Lean,
            marker: PhantomData,
        }
    }
}

// This requires nightly:
// https://github.com/rust-lang/rust/issues/57563
#[cfg(feature = "const_fn")]
impl<'a, T> Cow<'a, [T], Wide>
where
    T: Clone,
{
    /// Borrowed data.
    ///
    /// This is functionally identical to [`borrow`](./generic/struct.Cow.html#method.borrow).
    /// We use impl specialization to allow this function to be `const`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use beef::Cow;
    ///
    /// const HELLO: Cow<[u8]> = Cow::const_slice(&[1, 2, 3]);
    /// ```
    #[cfg_attr(
        feature = "const_deref",
        deprecated(note = "use Cow::borrowed() instead")
    )]
    pub const fn const_slice(val: &'a [T]) -> Self {
        Cow {
            // We are casting *const T to *mut T, however for all borrowed values
            // this raw pointer is only ever dereferenced back to &T.
            ptr: unsafe { NonNull::new_unchecked(val.as_ptr() as *mut T) },
            fat: val.len(),
            cap: None,
            marker: PhantomData,
        }
    }
}

// This requires nightly:
// https://github.com/rust-lang/rust/issues/57563
#[cfg(all(feature = "const_fn", target_pointer_width = "64"))]
impl<'a, T> Cow<'a, [T], Lean>
where
    T: Clone,
{
    /// Borrowed data.
    ///
    /// This i functionally identical to [`borrow`](./generic/struct.Cow.html#method.borrow).
    /// We use impl specialization to allow this function to be `const`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use beef::lean::Cow;
    ///
    /// const HELLO: Cow<[u8]> = Cow::const_slice(&[1, 2, 3]);
    /// ```
    #[cfg_attr(
        feature = "const_deref",
        deprecated(note = "use Cow::borrowed() instead")
    )]
    pub const fn const_slice(val: &'a [T]) -> Self {
        Cow {
            // We are casting *const T to *mut T, however for all borrowed values
            // this raw pointer is only ever dereferenced back to &T.
            ptr: unsafe { NonNull::new_unchecked(val.as_ptr() as *mut T) },
            fat: Lean::mask_len(val.len()),
            cap: Lean,
            marker: PhantomData,
        }
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

crate::cfg_const_deref! {
    impl<'a, T, U> const Default for Cow<'a, T, U>
    where
        T: Beef + ?Sized + ~const Steak,
        U: Capacity,
        &'a T: ~const Default,
    {
        #[inline]
        fn default() -> Self {
            Cow::borrowed(Default::default())
        }
    }
}

crate::cfg_const_deref! {
    impl<T, U> const Eq for Cow<'_, T, U>
    where
        T: Eq + Beef + ?Sized,
        U: Capacity,
    {
    }
}

crate::cfg_const_deref! {
    impl<A, B, U, V> const PartialOrd<Cow<'_, B, V>> for Cow<'_, A, U>
    where
        A: Beef + ?Sized + ~const Steak + ~const PartialOrd<B>,
        B: Beef + ?Sized + ~const Steak,
        U: Capacity,
        V: Capacity,
    {
        #[inline]
        fn partial_cmp(&self, other: &Cow<'_, B, V>) -> Option<Ordering> {
            PartialOrd::partial_cmp(self.borrow(), other.borrow())
        }
    }
}

crate::cfg_const_deref! {
    impl<T, U> const Ord for Cow<'_, T, U>
    where
        T: ~const Ord + Beef + ?Sized + ~const Steak,
        U: Capacity,
    {
        #[inline]
        fn cmp(&self, other: &Self) -> Ordering {
            Ord::cmp(self.borrow(), other.borrow())
        }
    }
}

crate::cfg_const_deref! {
    impl<'a, T, U> const From<&'a T> for Cow<'a, T, U>
    where
        T: Beef + ?Sized + ~const Steak,
        U: Capacity,
    {
        #[inline]
        fn from(val: &'a T) -> Self {
            Cow::borrowed(val)
        }
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
            unsafe { T::owned_from_parts::<U>(self.ptr, self.fat, capacity) };
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

crate::cfg_const_deref! {
    impl<T, U> const core::ops::Deref for Cow<'_, T, U>
    where
        T: Beef + ?Sized + ~const Steak,
        U: Capacity,
    {
        type Target = T;

        #[inline]
        fn deref(&self) -> &T {
            self.borrow()
        }
    }
}

crate::cfg_const_deref! {
    impl<T, U> const AsRef<T> for Cow<'_, T, U>
    where
        T: Beef + ?Sized + ~const Steak,
        U: Capacity,
    {
        #[inline]
        fn as_ref(&self) -> &T {
            self.borrow()
        }
    }
}

crate::cfg_const_deref! {
    impl<T, U> const Borrow<T> for Cow<'_, T, U>
    where
        T: Beef + ?Sized + ~const Steak,
        U: Capacity,
    {
        #[inline]
        fn borrow(&self) -> &T {
            self.borrow()
        }
    }
}

crate::cfg_const_deref! {
    impl<A, B, U, V> const PartialEq<Cow<'_, B, V>> for Cow<'_, A, U>
    where
        A: Beef + ?Sized + ~const Steak,
        B: Beef + ?Sized + ~const Steak,
        U: Capacity,
        V: Capacity,
        A: ~const PartialEq<B>,
    {
        fn eq(&self, other: &Cow<B, V>) -> bool {
            // fixme: compile error within `a == b` context :(
            self.borrow().eq(other.borrow())
        }
    }
}

crate::cfg_const_deref! {
    impl<'a, T, U> const From<StdCow<'a, T>> for Cow<'a, T, U>
    where
        T: Beef + ?Sized,
        U: Capacity,
    {
        #[inline]
        fn from(stdcow: StdCow<'a, T>) -> Self
        where
            T: ~const Steak,
            U: ~const Capacity,
            <T as ToOwned>::Owned: ~const Destruct
        {
            Cow::from_std_cow(stdcow)
        }
    }
}

crate::cfg_const_deref! {
    impl<'a, T, U> const From<Cow<'a, T, U>> for StdCow<'a, T>
    where
        T: Beef + ?Sized,
        U: Capacity,
    {
        #[inline]
        fn from(cow: Cow<'a, T, U>) -> Self
        where
            T: ~const Steak,
            U: ~const Capacity,
        {
            cow.into_std_cow()
        }
    }
}

macro_rules! impl_eq {
    ($($(@for< $bounds:tt >)? $ptr:ty => $([$($deref:tt)+])? <$with:ty>,)*) => {$(
        impl<U $(, $bounds)*> PartialEq<$with> for Cow<'_, $ptr, U>
        where
            U: Capacity,
            $( $bounds: Clone + PartialEq, )*
        {
            #[inline]
            fn eq(&self, other: &$with) -> bool {
                self.borrow() == $($($deref)*)* other
            }
        }

        impl<U $(, $bounds)*> PartialEq<Cow<'_, $ptr, U>> for $with
        where
            U: Capacity,
            $( $bounds: Clone + PartialEq, )*
        {
            #[inline]
            fn eq(&self, other: &Cow<$ptr, U>) -> bool {
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

// Safety: Same bounds as `std::borrow::Cow`.
unsafe impl<T, U> Sync for Cow<'_, T, U>
where
    U: Capacity,
    T: Beef + Sync + ?Sized,
    T::Owned: Sync,
{
}

unsafe impl<T, U> Send for Cow<'_, T, U>
where
    U: Capacity,
    T: Beef + Sync + ?Sized,
    T::Owned: Send,
{
}

impl<T, U> Unpin for Cow<'_, T, U>
where
    U: Capacity,
    T: Beef + ?Sized,
    T::Owned: Unpin,
{
}
