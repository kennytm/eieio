# `eieio` — Error Implementing `Eq + Clone` replacing `std::io::Error`

`eieio::Error` is a replacement of `std::io::Error` which implements `Eq + Clone`. This type is
intended as a "source" of other errors where a derived `Eq + Clone` implemented is desired.

```rust
// Constructs a standard io::Error...
let ioe = std::fs::read_dir("/dev/null").unwrap_err();

// Converts into an Eq + Clone error
let e1 = eieio::Error::from(ioe);
let e2 = e1.clone();
assert_eq!(e1, e2);
```

## Clone

`eieio::Error` stores custom errors in an `Arc` rather than a `Box` to allow universal cloning.

Conversion from `std::io::Error` to `eieio::Error` may need to perform a copy of the entire custom error.

## Equality

`eieio::Error` uses `Arc::ptr_eq` to compare equality of custom errors.

If the custom error carried by the original `std::io::Error` itself implements `Eq`, that custom equality is ignored:

```rust
use std::io::ErrorKind;

let e1 = eieio::Error::new(ErrorKind::Other, Box::from("foo"));
let e2 = eieio::Error::new(ErrorKind::Other, Box::from("foo"));
assert_ne!(e1, e2);
```
