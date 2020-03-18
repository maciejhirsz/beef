pub(crate) use internal::Capacity;
pub(crate) use internal::Beef;

pub(crate) mod internal {
    use alloc::borrow::ToOwned;
    use alloc::string::String;
    use alloc::vec::Vec;
    use core::mem::ManuallyDrop;
    use core::ptr::NonNull;

    pub trait Capacity {
        type Field: Copy;
        type NonZero: Copy;

        fn as_ref<T>(ptr: *const T, len: usize) -> *const [T];

        fn empty(len: usize) -> (usize, Self::Field);

        fn store(len: usize, capacity: usize) -> (usize, Self::Field);

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
        type PointerT;

        fn ref_into_parts<U>(&self) -> (NonNull<Self::PointerT>, usize, U::Field)
        where
            U: Capacity;

        unsafe fn ref_from_parts<U>(ptr: NonNull<Self::PointerT>, len: usize) -> *const Self
        where
            U: Capacity;

        /// Convert `T::Owned` to `NonNull<T>` and capacity.
        /// Return `None` for `0` capacity.
        fn owned_into_parts<U>(owned: Self::Owned) -> (NonNull<Self::PointerT>, usize, U::Field)
        where
            U: Capacity;

        /// Rebuild `T::Owned` from `NonNull<T>` and `capacity`. This can be done by the likes
        /// of [`Vec::from_raw_parts`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.from_raw_parts).
        unsafe fn owned_from_parts<U>(ptr: NonNull<Self::PointerT>, len: usize, capacity: U::NonZero) -> Self::Owned
        where
            U: Capacity;
    }

    unsafe impl Beef for str {
        type PointerT = u8;

        #[inline]
        fn ref_into_parts<U>(&self) -> (NonNull<u8>, usize, U::Field)
        where
            U: Capacity
        {
            let (len, cap) = U::empty(self.len());

            // A note on soundness:
            //
            // We are casting *const T to *mut T, however for all borrowed values
            // this raw pointer is only ever dereferenced back to &T.
            (unsafe { NonNull::new_unchecked(self.as_ptr() as *mut u8) }, len, cap)
        }

        #[inline]
        unsafe fn ref_from_parts<U>(ptr: NonNull<u8>, len: usize) -> *const str
        where
            U: Capacity
        {
            U::as_ref(ptr.as_ptr(), len) as *const str
        }

        #[inline]
        fn owned_into_parts<U>(owned: String) -> (NonNull<u8>, usize, U::Field)
        where
            U: Capacity,
        {
            // Convert to `String::into_raw_parts` once stabilized
            let mut owned = ManuallyDrop::new(owned);
            let (len, cap) = U::store(owned.len(), owned.capacity());

            (unsafe { NonNull::new_unchecked(owned.as_mut_ptr()) }, len, cap)
        }

        #[inline]
        unsafe fn owned_from_parts<U>(ptr: NonNull<u8>, len: usize, capacity: U::NonZero) -> String
        where
            U: Capacity,
        {
            let (len, cap) = U::unpack(len, capacity);

            String::from_utf8_unchecked(
                Vec::from_raw_parts(ptr.as_ptr(), len, cap),
            )
        }
    }

    unsafe impl<T: Clone> Beef for [T] {
        type PointerT = T;

        #[inline]
        fn ref_into_parts<U>(&self) -> (NonNull<T>, usize, U::Field)
        where
            U: Capacity
        {
            let (len, cap) = U::empty(self.len());

            // A note on soundness:
            //
            // We are casting *const T to *mut T, however for all borrowed values
            // this raw pointer is only ever dereferenced back to &T.
            (unsafe { NonNull::new_unchecked(self.as_ptr() as *mut T) }, len, cap)
        }

        #[inline]
        unsafe fn ref_from_parts<U>(ptr: NonNull<T>, len: usize) -> *const [T]
        where
            U: Capacity
        {
            U::as_ref(ptr.as_ptr(), len)
        }

        #[inline]
        fn owned_into_parts<U>(owned: Vec<T>) -> (NonNull<T>, usize, U::Field)
        where
            U: Capacity,
        {
            // Convert to `Vec::into_raw_parts` once stabilized
            let mut owned = ManuallyDrop::new(owned);
            let (len, cap) = U::store(owned.len(), owned.capacity());

            (unsafe { NonNull::new_unchecked(owned.as_mut_ptr()) }, len, cap)
        }

        #[inline]
        unsafe fn owned_from_parts<U>(ptr: NonNull<T>, len: usize, capacity: U::NonZero) -> Vec<T>
        where
            U: Capacity,
        {
            let (len, cap) = U::unpack(len, capacity);

            Vec::from_raw_parts(ptr.as_ptr(), len, cap)
        }
    }
}