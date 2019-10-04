use super::error::Error;

/// A specialized [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html)
/// type for the rocket-config operations.
///
/// This type is broadly used across [`rocket-config`] for any operation which
/// may produce an error.
///
/// This typedef is generally used to avoid writing out [`error::Error`]
/// directly and is otherwise a direct mapping to [`Result`].
///
/// While usual Rust style is to import types directly, aliases of [`Result`]
/// often are not, to make it easier to distinguish between them. [`Result`] is
/// generally assumed to be [`std::result::Result`], and so users of
/// this alias will generally use `rocket_config::Result` instead of shadowing
/// the prelude's import of [`std::result::Result`].
///
/// [`rocket-config`]: ../index.html
/// [`error::Error`]: ../error/struct.Error.html
/// [`Result`]: ../result/enum.Result.html
/// [`std::result::Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use std::error::Error as _;
    use super::super::error;
    use super::Result;

    #[test]
    fn result() {
        let result: Result<&str> = Ok("test");
        assert_eq!(result.unwrap(), "test");

        let result: Result<&str> = Err(error::Error::from(error::ErrorKind::Other));
        assert_eq!(result.unwrap_err().description(), "other");

        let result: Result<&str> = Err(error::Error::new(
            error::ErrorKind::Other, "test other"
        ));
        assert_eq!(result.unwrap_err().description(), "test other");
    }
}