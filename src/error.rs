use std::fmt;

/// The error type which is returned from decoding MUTF-8 data to UTF-8.
///
/// This type does not support transmission of an error other than that an error
/// occurred. This is desired, as MUTF-8 should only be used for internal use,
/// any error should signify an issue with a developer's code or some attacker
/// is trying to improperly encode data to evade security checks.
///
/// ```rust
/// let bytes: &[u8] = &[];
/// if let Err(mutf8::DecodingError) = mutf8::decode(bytes) {
///     panic!("An error occurred");
/// }
/// ```
#[derive(Debug)]
pub struct DecodingError;

impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not convert MUTF-8 data to UTF-8 data")
    }
}

impl std::error::Error for DecodingError {}
