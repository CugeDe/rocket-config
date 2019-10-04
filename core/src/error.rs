#![allow(dead_code)]
//! Type representing various errors that can occur in a Rocket application.

use std::error;
use std::fmt;

/// The error type for rocket-config operations of the associated traits.
///
/// Custom instances of `Error` can be created with crafted error messages
/// and a particular value of [`rocket-config::error::ErrorKind`].
///
/// [`rocket-config::error::ErrorKind`]: enum.ErrorKind.html
pub struct Error {
    repr: Repr
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.repr, f)
    }
}

enum Repr {
    Simple(ErrorKind),
    Custom(Box<Custom>),
}

#[derive(Debug)]
struct Custom {
    kind: ErrorKind,
    error: Box<dyn error::Error+Send+Sync>,
}

/// A list specifying general categories of rocket-config error.
///
/// This list is intended to grow over time and it is not recommended to
/// exhaustively match against it.
///
/// It is used with the [`rocket-config::error::Error`] type.
///
/// [`rocket-config::error::Error`]: struct.Error.html
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    FormatError,
    MissingValue,
    UnimplementedFormat,
    Other,
}

impl ErrorKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ErrorKind::FormatError          => "format_error",
            ErrorKind::MissingValue         => "missing_value",
            ErrorKind::UnimplementedFormat  => "unimplemented_format",
            ErrorKind::Other                => "other",
        }
    }
}

/// Intended for use for errors not exposed to the user, where allocating onto
/// the heap (for normal construction via Error::new) is too costly.
impl From<ErrorKind> for Error {
    /// Converts an [`ErrorKind`] into an [`Error`].
    ///
    /// This conversion allocates a new error with a simple representation of error kind.
    /// [`ErrorKind`]: ./enum.ErrorKind.html
    /// [`Error`]: ./struct.Error.html
    #[inline]
    fn from(kind: ErrorKind) -> Error {
        Error {
            repr: Repr::Simple(kind)
        }
    }
}

impl Error {
    /// Creates a new I/O error from a known kind of error as well as an
    /// arbitrary error payload.
    ///
    /// This function is used to generically create I/O errors which do not
    /// originate from the OS itself. The `error` argument is an arbitrary
    /// payload which will be contained in this `Error`.
    pub fn new<E>(kind: ErrorKind, error: E) -> Error
        where E: Into<Box<dyn error::Error+Send+Sync>>
    {
        Self::_new(kind, error.into())
    }

    fn _new(kind: ErrorKind, error: Box<dyn error::Error+Send+Sync>) -> Error {
        Error {
            repr: Repr::Custom(Box::new(Custom {
                kind,
                error,
            }))
        }
    }

    /// Returns a reference to the inner error wrapped by this error (if any).
    ///
    /// If this `Error` was constructed via `new` then this function will
    /// return `Some`, otherwise it will return `None`.
    pub fn get_ref(&self) -> Option<&(dyn error::Error+Send+Sync+'static)> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::Custom(ref c) => Some(&*c.error),
        }
    }

    /// Returns a mutable reference to the inner error wrapped by this error
    /// (if any).
    ///
    /// If this `Error` was constructed via `new` then this function will
    /// return `Some`, otherwise it will return `None`.
    pub fn get_mut(&mut self) -> Option<&mut (dyn error::Error+Send+Sync+'static)> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::Custom(ref mut c) => Some(&mut *c.error),
        }
    }

    /// Consumes the `Error`, returning its inner error (if any).
    ///
    /// If this `Error` was constructed via `new` then this function will
    /// return `Some`, otherwise it will return `None`.
    pub fn into_inner(self) -> Option<Box<dyn error::Error+Send+Sync>> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::Custom(c) => Some(c.error)
        }
    }

    /// Returns the corresponding `ErrorKind` for this error.
    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            Repr::Custom(ref c) => c.kind,
            Repr::Simple(kind) => kind,
        }
    }
}

impl fmt::Debug for Repr {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Repr::Custom(ref c) => fmt::Debug::fmt(&c, fmt),
            Repr::Simple(kind) => fmt.debug_tuple("Kind").field(&kind).finish(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.repr {
            Repr::Custom(ref c) => c.error.fmt(fmt),
            Repr::Simple(kind) => write!(fmt, "{}", kind.as_str()),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self.repr {
            Repr::Simple(..) => self.kind().as_str(),
            Repr::Custom(ref c) => c.error.description(),
        }
    }

    #[allow(deprecated)]
    fn cause(&self) -> Option<&dyn error::Error> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::Custom(ref c) => c.error.cause(),
        }
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::Custom(ref c) => c.error.source(),
        }
    }
}

fn _assert_error_is_sync_send() {
    fn _is_sync_send<T: Sync+Send>() {}
    _is_sync_send::<Error>();
}

#[cfg(test)]
mod tests {
    use std::error::Error as _;
    use super::{Error, ErrorKind};

    #[test]
    fn custom() {
        let error = Error::new(
            ErrorKind::Other, "test error"
        );

        assert_eq!(error.kind(), ErrorKind::Other);
    }

    #[test]
    fn simple() {
        let error = Error::from(
            ErrorKind::Other
        );

        assert_eq!(error.kind(), ErrorKind::Other);
    }

    #[test]
    fn errorkind_as_str() {
        let error_format_error = Error::from(ErrorKind::FormatError);
        let error_missing_value = Error::from(ErrorKind::MissingValue);
        let error_other = Error::from(ErrorKind::Other);
        let error_unimplemented_format = Error::from(ErrorKind::UnimplementedFormat);

        assert_eq!(error_format_error.kind().as_str(), "format_error");
        assert_eq!(error_missing_value.kind().as_str(), "missing_value");
        assert_eq!(error_other.kind().as_str(), "other");
        assert_eq!(error_unimplemented_format.kind().as_str(), "unimplemented_format");
    }

    #[test]
    fn custom_get_ref() {
        let error = Error::new(ErrorKind::Other, "test error");
        let ref_error = error.get_ref();

        assert!(ref_error.is_some());
        assert_eq!(ref_error.unwrap().description(), "test error");
    }

    #[test]
    fn simple_get_ref() {
        let error = Error::from(ErrorKind::Other);

        assert!(error.get_ref().is_none());
    }

    #[test]
    fn custom_get_mut() {
        let mut error = Error::new(ErrorKind::Other, "test error");
        let ref_error = error.get_mut();

        assert!(ref_error.is_some());
        assert_eq!(ref_error.unwrap().description(), "test error");
    }

    #[test]
    fn simple_get_mut() {
        let mut error = Error::from(ErrorKind::Other);

        assert!(error.get_mut().is_none());
    }

    #[test]
    fn custom_into_inner() {
        let error = Error::new(ErrorKind::Other, "test error");
        let inner_error = error.into_inner();

        assert!(inner_error.is_some());
        assert_eq!(inner_error.unwrap().description(), "test error");
    }

    #[test]
    fn simple_into_inner() {
        let error = Error::from(ErrorKind::Other);

        assert!(error.into_inner().is_none());
    }

    #[test]
    fn custom_description() {
        let error = Error::new(ErrorKind::Other, "test error");

        assert_eq!(error.description(), "test error");
    }

    #[test]
    fn simple_description() {
        let error = Error::from(ErrorKind::Other);

        assert_eq!(error.description(), ErrorKind::Other.as_str());
    }

    #[test]
    #[allow(deprecated)]
    fn custom_cause() {
        let error = Error::new(ErrorKind::Other, "test error");
        let error_cause = error.cause();

        assert!(error_cause.is_none());
    }

    #[test]
    #[allow(deprecated)]
    fn simple_cause() {
        let error = Error::from(ErrorKind::Other);
        let error_cause = error.cause();

        assert!(error_cause.is_none());
    }

    #[test]
    fn custom_source() {
        let error = Error::new(ErrorKind::Other, "test error");
        let error_source = error.source();

        assert!(error_source.is_none());
    }

    #[test]
    fn simple_source() {
        let error = Error::from(ErrorKind::Other);
        let error_source = error.source();

        assert!(error_source.is_none());
    }

    #[test]
    fn custom_debug() {
        let error = Error::new(ErrorKind::Other, "test error");

        assert_eq!(
            format!("{:?}", error),
            "Custom { kind: Other, error: \"test error\" }"
        );
    }

    #[test]
    fn simple_debug() {
        let error = Error::from(ErrorKind::Other);

        assert_eq!(format!("{:?}", error), "Kind(Other)");
    }

    #[test]
    fn custom_display() {
        let error = Error::new(ErrorKind::Other, "test error");

        assert_eq!(format!("{}", error), "test error");
    }

    #[test]
    fn simple_display() {
        let error = Error::from(ErrorKind::Other);

        assert_eq!(format!("{}", error), "other");
    }

    #[test]
    fn assert_error_is_sync_send() {
        super::_assert_error_is_sync_send();
    }
}