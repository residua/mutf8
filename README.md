# residua-mutf8

A simple library for converting between MUTF-8 and UTF-8.

[![Build Status]][actions]
[![Latest Version]][crates.io]

[Build Status]: https://img.shields.io/github/workflow/status/residua/mutf8/ci?logo=github
[actions]: https://github.com/residua/mutf8/actions/workflows/ci.yml
[Latest Version]: https://img.shields.io/crates/v/residua-mutf8?logo=rust
[crates.io]: https://crates.io/crates/residua-mutf8

## Documentation

View the full reference on `docs.rs` [here](https://docs.rs/residua-mutf8).

## Usage

This crate is [on crates.io][crates] and can be used by adding `residua-mutf8`
to your dependencies in your project's `Cargo.toml`:

```toml
[dependencies]
residua-mutf8 = "1"
```

[crates]: https://crates.io/crates/residua-mutf8

## Examples

Basic usage

```rust
use std::borrow::Cow;
use mutf8::{to_mutf8, from_mutf8};

let str = "Hello, world!";
// 16-bit Unicode characters are the same in UTF-8 and MUTF-8:
assert_eq!(to_mutf8(str), Cow::Borrowed(str.as_bytes()));
assert_eq!(from_mutf8(str.as_bytes()), Ok(Cow::Borrowed(str)));

let str = "\u{10401}";
let mutf8_data = &[0xED, 0xA0, 0x81, 0xED, 0xB0, 0x81];
// 'mutf8_data' is a byte slice containing a 6-byte surrogate pair which
// becomes a 4-byte UTF-8 character.
assert_eq!(from_mutf8(mutf8_data), Ok(Cow::Owned(str.to_string())));

let str = "\0";
let mutf8_data = &[0xC0, 0x80];
// 'str' is a null character which becomes a two-byte MUTF-8 representation.
assert_eq!(to_mutf8(str), Cow::<[u8]>::Owned(mutf8_data.to_vec()));
```

## License

Licensed under either of

-   Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
-   MIT license
    ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
