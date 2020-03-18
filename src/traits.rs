pub(crate) use internal::Capacity;
pub(crate) use internal::Beef;
pub(crate) use internal::Inner;

pub(crate) mod internal {
    use alloc::borrow::ToOwned;
    use alloc::string::String;
    use alloc::vec::Vec;
    use core::mem::ManuallyDrop;
    use core::ptr::NonNull;

    /// Type alias to a fat pointer slice. This is using empty slice `()`
    /// to ensure that we can read length from it, even if said length is
    /// an arbitrary integer.
    pub type Inner = NonNull<[()]>;

    pub trait Capacity {
        type Field: Copy;
        type NonZero: Copy;

        fn make_valid<T>(ptr: *const [()]) -> *const [T];

        fn empty<T>(ptr: *const [T]) -> (*mut [()], Self::Field);

        fn store<T>(ptr: *mut T, len: usize, capacity: usize) -> (*mut [()], Self::Field);

        fn unpack(len: usize, capacity: Self::NonZero) -> (usize, usize);

        fn maybe(len: usize, capacity: Self::Field) -> Option<Self::NonZero>;
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
        fn ref_into_parts<U>(&self) -> (Inner, U::Field)
        where
            U: Capacity;

        unsafe fn ref_from_parts<U>(inner: Inner) -> *const Self
        where
            U: Capacity;

        /// Convert `T::Owned` to `NonNull<T>` and capacity.
        /// Return `None` for `0` capacity.
        fn owned_into_parts<U>(owned: Self::Owned) -> (Inner, U::Field)
        where
            U: Capacity;

        /// Rebuild `T::Owned` from `NonNull<T>` and `capacity`. This can be done by the likes
        /// of [`Vec::from_raw_parts`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.from_raw_parts).
        unsafe fn owned_from_parts<U>(inner: Inner, capacity: U::NonZero) -> Self::Owned
        where
            U: Capacity;
    }

    unsafe impl Beef for str {
        #[inline]
        fn ref_into_parts<U>(&self) -> (Inner, U::Field)
        where
            U: Capacity
        {
            let (inner, cap) = U::empty(self as *const str as *const [u8]);

            (unsafe { NonNull::new_unchecked(inner) }, cap)
        }

        #[inline]
        unsafe fn ref_from_parts<U>(inner: Inner) -> *const str
        where
            U: Capacity
        {
            U::make_valid::<u8>(inner.as_ptr()) as *const str
        }

        #[inline]
        fn owned_into_parts<U>(owned: String) -> (Inner, U::Field)
        where
            U: Capacity,
        {
            // Convert to `String::into_raw_parts` once stabilized
            let mut owned = ManuallyDrop::new(owned);
            let (inner, cap) = U::store(owned.as_mut_ptr(), owned.len(), owned.capacity());

            (unsafe { NonNull::new_unchecked(inner) }, cap)
        }

        #[inline]
        unsafe fn owned_from_parts<U>(inner: Inner, capacity: U::NonZero) -> String
        where
            U: Capacity,
        {
            let len = (*inner.as_ptr()).len();
            let (len, cap) = U::unpack(len, capacity);

            String::from_utf8_unchecked(
                Vec::from_raw_parts(inner.cast().as_ptr(), len, cap),
            )
        }
    }

    unsafe impl<T: Clone> Beef for [T] {
        #[inline]
        fn ref_into_parts<U>(&self) -> (Inner, U::Field)
        where
            U: Capacity
        {
            // A note on soundness:
            //
            // We are casting *const T to *mut T, however for all borrowed values
            // this raw pointer is only ever dereferenced back to &T.
            let (inner, cap) = U::empty(self as *const [T]);

            (unsafe { NonNull::new_unchecked(inner) }, cap)
        }

        #[inline]
        unsafe fn ref_from_parts<U>(inner: Inner) -> *const [T]
        where
            U: Capacity
        {
            U::make_valid(inner.as_ptr())
        }

        #[inline]
        fn owned_into_parts<U>(owned: Vec<T>) -> (Inner, U::Field)
        where
            U: Capacity,
        {
            // Convert to `Vec::into_raw_parts` once stabilized
            let mut owned = ManuallyDrop::new(owned);
            let (inner, cap) = U::store(owned.as_mut_ptr(), owned.len(), owned.capacity());

            (unsafe { NonNull::new_unchecked(inner) }, cap)
        }

        #[inline]
        unsafe fn owned_from_parts<U>(inner: Inner, capacity: U::NonZero) -> Vec<T>
        where
            U: Capacity,
        {
            let len = (*inner.as_ptr()).len();
            let (len, cap) = U::unpack(len, capacity);

            Vec::from_raw_parts(inner.cast().as_ptr() as *mut T, len, cap)
        }
    }
}