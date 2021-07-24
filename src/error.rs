
use core::fmt::{self, Debug, Display};
// use core::error::Error;

/// Error that occures when converting the `Cow` into a borrowed value using
/// the `TryFrom` or `TryInto` trait.
///
/// This error occurs because it's impossible to borrow owned value of `Cow<'a, T>`
/// for a lifetime `'a` while dropping it.
///
/// For infallible way to convert `Cow<'a, T>` to a reference `&T` use `AsRef` instead:
///
/// ```
/// use beef::{Cow, OwnedVariantError};
/// use std::convert::TryInto;
///
/// let borrowed: Cow<str> = Cow::borrowed("foobar");
/// let owned: Cow<str> = Cow::owned("foobar".to_owned());
///
/// let slice_from_borrowed: Result<&str, _> = borrowed.clone().try_into();
/// let slice_from_owned: Result<&str, _> = owned.clone().try_into();
///
/// // Converting from owned value fails
/// assert_eq!(slice_from_borrowed, Ok("foobar"));
/// assert_eq!(slice_from_owned, Err(OwnedVariantError));
///
/// let slice_from_borrowed: &str = borrowed.as_ref();
/// let slice_from_owned: &str = owned.as_ref();
///
/// // Using `as_ref` is infallibe, but borrow live for a local lifetime
/// // and cannot outlive the original `Cow`s.
/// assert_eq!(slice_from_borrowed, "foobar");
/// assert_eq!(slice_from_owned, "foobar");
/// ```
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct OwnedVariantError;

impl Display for OwnedVariantError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Owned variant of beef::Cow cannot be converted to a borrowed slice, try using `as_ref()` instead.")
    }
}

impl Debug for OwnedVariantError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <OwnedVariantError as Display>::fmt(self, f)
    }
}

// impl Error for OwnedVariantError {}