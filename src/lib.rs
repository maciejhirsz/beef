//! Faster, more compact implementation of `Cow`.
//!
//! **[Changelog](https://github.com/maciejhirsz/beef/releases) -**
//! **[Cargo](https://crates.io/crates/beef) -**
//! **[Repository](https://github.com/maciejhirsz/beef)**
//!
//! ```rust
//! use beef::Cow;
//!
//! let borrowed: Cow<str> = Cow::borrowed("Hello");
//! let owned: Cow<str> = Cow::owned(String::from("World"));
//!
//! assert_eq!(
//!     format!("{} {}!", borrowed, owned),
//!     "Hello World!",
//! );
//! ```
//!
//! There are two versions of `Cow` exposed by this crate:
//!
//! + `beef::Cow` is 3 words wide: pointer, length, and capacity. It stores the ownership tag in capacity.
//! + `beef::lean::Cow` is 2 words wide, storing length, capacity, and the ownership tag all in a fat pointer.
//!
//! Both versions are leaner than the `std::borrow::Cow`:
//!
//! ```rust
//! use std::mem::size_of;
//!
//! const WORD: usize = size_of::<usize>();
//!
//! assert_eq!(size_of::<std::borrow::Cow<str>>(), 4 * WORD);
//! assert_eq!(size_of::<beef::Cow<str>>(), 3 * WORD);
//! assert_eq!(size_of::<beef::lean::Cow<str>>(), 2 * WORD);
//! ```
#![cfg_attr(feature = "const_fn", feature(const_fn))]
#![warn(missing_docs)]

#![cfg_attr(not(test), no_std)]
extern crate alloc;

mod fat;
mod traits;

#[cfg(target_pointer_width = "64")]
pub mod lean;
pub mod generic;

pub use fat::Cow;

#[cfg(test)]
mod tests {
    use crate::Cow;

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
