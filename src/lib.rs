//! A library for converting between MUTF-8 and UTF-8.
//!
//! MUTF-8 is the same as CESU-8 except for its handling of embedded null
//! characters. This library builds on top of the `residua-cesu8` crate found
//! [here][residua-cesu8].
//!
//! [residua-cesu8]: https://github.com/residua/cesu8
//!
//! ```
//! use std::borrow::Cow;
//! use mutf8::{to_mutf8, from_mutf8};
//!
//! let str = "Hello, world!";
//! // 16-bit Unicode characters are the same in UTF-8 and MUTF-8:
//! assert_eq!(to_mutf8(str), Cow::Borrowed(str.as_bytes()));
//! assert_eq!(from_mutf8(str.as_bytes()), Ok(Cow::Borrowed(str)));
//!
//! let str = "\u{10401}";
//! let mutf8_data = &[0xED, 0xA0, 0x81, 0xED, 0xB0, 0x81];
//! // 'mutf8_data' is a byte slice containing a 6-byte surrogate pair which
//! // becomes a 4-byte UTF-8 character.
//! assert_eq!(from_mutf8(mutf8_data), Ok(Cow::<str>::Owned(str.to_string())));
//!
//! let str = "\0";
//! let mutf8_data = &[0xC0, 0x80];
//! // 'str' is a null character which becomes a two-byte MUTF-8 representation.
//! assert_eq!(to_mutf8(str), Cow::<[u8]>::Owned(mutf8_data.to_vec()));
//! ```

#![deny(clippy::pedantic)]

use std::{borrow::Cow, error::Error, fmt, str::from_utf8};

use cesu8::{cesu8_len, from_cesu8, is_valid_cesu8, to_cesu8};

/// Converts a slice of bytes to a string slice.
///
/// First, if the slice of bytes is already valid UTF-8, this function is
/// functionally no different than `std::str::from_utf8`; this means that
/// `decode()` does not need to perform any further operations and doesn't need
/// to allocate additional memory.
///
/// If the slice of bytes is not valid UTF-8, `decode()` works on the assumption
/// that the slice of bytes, if not valid UTF-8, is valid MUTF-8. It will then
/// decode the bytes given to it and return the newly constructed string slice.
///
/// If the slice of bytes is found not to be valid MUTF-8 data, `decode()`
/// returns `Err(DecodingError)` to signify that an error has occured.
///
/// # Errors
///
/// Returns [`DecodingError`] if the input is invalid MUTF-8 data.
///
/// # Examples
///
/// ```
/// use std::borrow::Cow;
/// use mutf8::from_mutf8;
///
/// let str = "Hello, world!";
/// // Since 'str' contains valid UTF-8 and MUTF-8 data, 'from_mutf8' can
/// // decode the string slice without allocating memory.
/// assert_eq!(from_mutf8(str.as_bytes()), Ok(Cow::Borrowed(str)));
///
/// let str = "\u{10401}";
/// let mutf8_data = &[0xED, 0xA0, 0x81, 0xED, 0xB0, 0x81];
/// // 'mutf8_data' is a byte slice containing a 6-byte surrogate pair which
/// // becomes the 4-byte UTF-8 character 'str'.
/// assert_eq!(from_mutf8(mutf8_data), Ok(Cow::Owned(str.to_string())));
///
/// let str = "\0";
/// let mutf8_data = &[0xC0, 0x80];
/// // 'mutf8_data' is a byte slice containing MUTF-8 data containing a null
/// // code point which becomes a null character.
/// assert_eq!(from_mutf8(mutf8_data), Ok(Cow::Owned(str.to_string())));
/// ```
#[inline]
pub fn from_mutf8(bytes: &[u8]) -> Result<Cow<str>, DecodingError> {
    from_utf8(bytes)
        .map(Cow::Borrowed)
        .or_else(|_| decode_mutf8(bytes).map(Cow::Owned))
}

#[inline(never)]
#[cold]
fn decode_mutf8(bytes: &[u8]) -> Result<String, DecodingError> {
    macro_rules! err {
        () => {{
            return Err(DecodingError);
        }};
    }

    let mut decoded = Vec::with_capacity(bytes.len());
    let mut iter = bytes.iter();

    while let Some(&byte) = iter.next() {
        let value = if byte == NULL_PAIR[0] {
            match iter.next() {
                Some(&byte) => {
                    if byte != NULL_PAIR[1] {
                        err!()
                    }
                }
                _ => err!(),
            }
            NULL_CODE_POINT
        } else {
            byte
        };
        decoded.push(value);
    }

    from_cesu8(&decoded)
        .map(Cow::into_owned)
        .map_err(From::from)
}

/// Converts a string slice to MUTF-8 bytes.
///
/// If the string slice's representation in MUTF-8 would be identical to its
/// present UTF-8 representation, this function is functionally no different
/// than `(&str).as_bytes()`; this means that `encode()` does not need to
/// perform any further operations and doesn't need to allocate any additional
/// memory.
///
/// If the string slice's representation in UTF-8 is not equivalent in MUTF-8,
/// `encode()` encodes the string slice to its MUTF-8 representation as a slice
/// of bytes.
///
/// ```
/// use std::borrow::Cow;
/// use mutf8::to_mutf8;
///
/// let str = "Hello, world!";
/// // Since 'str' contains valid UTF-8 and MUTF-8 data, 'to_mutf8' can
/// // encode data without allocating memory.
/// assert_eq!(to_mutf8(str), Cow::Borrowed(str.as_bytes()));
///
/// let str = "\u{10401}";
/// let mutf8_data = Cow::Borrowed(&[0xED, 0xA0, 0x81, 0xED, 0xB0, 0x81]);
/// // 'str' is a 4-byte UTF-8 character, which becomes the 6-byte MUTF-8
/// // surrogate pair 'mutf8_data'.
/// assert_eq!(to_mutf8(str), mutf8_data);
///
/// let str = "\0";
/// let mutf8_data = Cow::Borrowed(&[0xC0, 0x80]);
/// // 'str' is a null character which becomes a two byte representation in
/// // MUTF-8.
/// assert_eq!(to_mutf8(str), mutf8_data);
/// ```
#[must_use]
#[inline]
pub fn to_mutf8(s: &str) -> Cow<[u8]> {
    if is_valid_mutf8(s) {
        Cow::Borrowed(s.as_bytes())
    } else {
        Cow::Owned(encode_mutf8(s))
    }
}

#[must_use]
#[inline(never)]
#[cold]
fn encode_mutf8(s: &str) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(mutf8_len(s));

    for &byte in to_cesu8(s).iter() {
        if byte == NULL_CODE_POINT {
            encoded.extend_from_slice(&NULL_PAIR);
        } else {
            encoded.push(byte);
        }
    }

    encoded
}

/// The pair of bytes the null code point (`0x00`) is represented by in MUTF-8.
const NULL_PAIR: [u8; 2] = [0xC0, 0x80];

/// Given a string slice, this function returns how many bytes in MUTF-8 are
/// required to encode the string slice.
#[must_use]
pub fn mutf8_len(s: &str) -> usize {
    let mut len = cesu8_len(s);
    s.as_bytes().iter().for_each(|&b| {
        if b == NULL_CODE_POINT {
            len += 1;
        }
    });
    len
}

/// Returns `true` if a string slice contains UTF-8 data that is also valid
/// MUTF-8. This is mainly used in testing if a string slice needs to be
/// explicitly encoded using [`to_mutf8`].
///
/// If `is_valid_mutf8()` returns `false`, it implies that
/// [`&str.as_bytes()`](str::as_bytes) is directly equivalent to the string
/// slice's MUTF-8 representation.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use mutf8::is_valid_mutf8;
///
/// // Code points below U+10400 encoded in UTF-8 IS valid MUTF-8.
/// assert!(is_valid_mutf8("Hello, world!"));
///
/// // Any code point above U+10400 encoded in UTF-8 IS NOT valid MUTF-8.
/// assert!(!is_valid_mutf8("\u{10400}"));
///
/// // The use of a null character IS NOT valid MUTF-8.
/// assert!(!is_valid_mutf8("\0"));
/// ```
#[must_use]
#[inline]
pub fn is_valid_mutf8(s: &str) -> bool {
    !s.contains(NULL_CHAR) && is_valid_cesu8(s)
}

const NULL_CODE_POINT: u8 = 0x00;
const NULL_CHAR: char = '\0';

/// An error thrown by [`from_mutf8`] when the input is invalid MUTF-8 data.
///
/// This type does not support transmission of an error other than that an error
/// occurred.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodingError;

impl fmt::Display for DecodingError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid MUTF-8 data")
    }
}

impl From<cesu8::DecodingError> for DecodingError {
    #[inline]
    fn from(_: cesu8::DecodingError) -> Self {
        DecodingError
    }
}

impl Error for DecodingError {}
