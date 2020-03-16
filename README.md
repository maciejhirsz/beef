# beef

[![Travis shield](https://travis-ci.org/maciejhirsz/beef.svg)](https://travis-ci.org/maciejhirsz/beef)
[![Crates.io version shield](https://img.shields.io/crates/v/beef.svg)](https://crates.io/crates/beef)
[![Crates.io license shield](https://img.shields.io/crates/l/beef.svg)](https://crates.io/crates/beef)

Faster, more compact implementation of `Cow`.

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
```

There are two versions of `Cow` exposed by this crate:

+ `beef::Cow` is 3 words wide: pointer, length, and capacity. It stores the ownership tag in capacity.
+ `beef::skinny::Cow` is 2 words wide, storing length, capacity, and the ownership tag all in a fat pointer.

Both versions are leaner than the `std::borrow::Cow`:

```rust
use std::mem::size_of;

const WORD: usize = size_of::<usize>();

assert_eq!(size_of::<std::borrow::Cow<str>>(), 4 * WORD);
assert_eq!(size_of::<beef::Cow<str>>(), 3 * WORD);
assert_eq!(size_of::<beef::skinny::Cow<str>>(), 2 * WORD);
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
          +-----------+-----------+-----------+-----------+
Borrowed: | Tag       | Pointer   | Length    | XXXXXXXXX |
          +-----------+-----------+-----------+-----------+

          +-----------+-----------+-----------+-----------+
Owned:    | Tag       | Pointer   | Length    | Capacity  |
          +-----------+-----------+-----------+-----------+
```

Instead of being an enum with a tag, `beef::Cow` uses capacity to determine whether the
value it's holding is owned (capacity is greater than 0), or borrowed (capacity is 0).

`beef::skinny::Cow` goes even further and puts length and capacity on a single 64 word.

```text
                   +-----------+-----------+-----------+
beef::Cow          | Pointer   | Length    | Capacity? |
                   +-----------+-----------+-----------+

                   +-----------+-----+-----+
beef::skinny::Cow  | Pointer   | Cap | Len |
                   +-----------+-----+-----+
```


## Benchmarks

```
cargo +nightly bench
```

Microbenchmarking obtaining a `&str` reference is rather flaky and you can have widely different results. In general the following seems to hold true:

+ `beef::Cow` and `beef::skinny::Cow` are faster than `std::borrow::Cow` at obtaining a reference `&T`, we don't have to check the tag to do that and the fat pointer is in the same place for both owned and borrowed values.
+ The 3-word `beef::Cow` is faster at creating borrowed variants, but slower at creating owned variants than `std::borrow::Cow`.
+ The 2-word `beef::skinny::Cow` is faster at both.

```
running 9 tests
test beef_as_ref              ... bench:          58 ns/iter (+/- 6)
test beef_create              ... bench:         137 ns/iter (+/- 3)
test beef_create_mixed        ... bench:         692 ns/iter (+/- 29)
test skinny_beef_as_ref       ... bench:          53 ns/iter (+/- 2)
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
