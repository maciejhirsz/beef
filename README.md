# beef

Alternative implementation of `Cow` that's more compact in memory.

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