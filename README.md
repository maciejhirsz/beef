# beef

[![Travis shield](https://travis-ci.org/maciejhirsz/beef.svg)](https://travis-ci.org/maciejhirsz/beef)
[![Crates.io version shield](https://img.shields.io/crates/v/beef.svg)](https://crates.io/crates/beef)
[![Crates.io license shield](https://img.shields.io/crates/l/beef.svg)](https://crates.io/crates/beef)

Alternative implementation of `Cow` that's more compact in memory.

**[Changelog](https://github.com/maciejhirsz/beef/releases) -**
**[Documentation](https://docs.rs/beef/) -**
**[Cargo](https://crates.io/crates/beef) -**
**[Repository](https://github.com/maciejhirsz/beef)**

```rust
use beef::Cow;

let borrowed = Cow::borrowed("Hello");
let owned = Cow::from(String::from("World"));

assert_eq!(
    format!("{} {}!", borrowed, owned),
    "Hello World!",
);

// beef::Cow is 3 word sized, while std::borrow::Cow is 4 word sized
assert!(std::mem::size_of::<Cow<str>>() < std::mem::size_of::<std::borrow::Cow<str>>());
```

## How does it work?

The standard library `Cow` is an enum with two variants:

```rust
pub enum Cow<'a, B> where
    B: 'a + ToOwned + ?Sized,
{
    Borrowed(&'a B),
    Owned(<B as ToOwned>::Owned),
}
```

For the most common pairs of values - `&str` and `String`, or `&[u8]` and `Vec<u8>` - this
means that the entire enum is 4 words wide:

```text
                                             Padding
                                                |
                                                v
          +----------+----------+----------+----------+
Borrowed: | Tag      | Pointer  | Length   | XXXXXXXX |
          +----------+----------+----------+----------+

          +----------+----------+----------+----------+
Owned:    | Tag      | Pointer  | Length   | Capacity |
          +----------+----------+----------+----------+
```

Instead of being an enum with a tag, `beef::Cow` uses capacity to determine whether the
value it's holding is owned (capacity is greater than 0), or borrowed (capacity is 0).

## Benchmarks

+ The 3-word `beef::Cow` dereferences to `&T` faster than `std::borrow::Cow`, and is also faster at creating borrows. It suffers when creating owned values.
+ The 2-word `beef::skinny::Cow` is faster than `std::borrow::Cow` _and_ `beef::Cow` in all benchmarks.

```
running 9 tests
test beef_as_ref              ... bench:          58 ns/iter (+/- 6)
test beef_create              ... bench:         137 ns/iter (+/- 3)
test beef_create_mixed        ... bench:         692 ns/iter (+/- 29)
test skinny_beef_as_ref       ... bench:          29 ns/iter (+/- 0)
test skinny_beef_create       ... bench:          79 ns/iter (+/- 4)
test skinny_beef_create_mixed ... bench:         595 ns/iter (+/- 15)
test std_as_ref               ... bench:          72 ns/iter (+/- 2)
test std_create               ... bench:         150 ns/iter (+/- 2)
test std_create_mixed         ... bench:         671 ns/iter (+/- 30)
```

## License

This crate is distributed under the terms of both the MIT license
and the Apache License (Version 2.0). Choose whichever one works best for you.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
