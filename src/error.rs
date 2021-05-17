use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result},
    io,
};

#[derive(Debug, Clone)]
pub enum ParseError {
    ImsufficentBuffer(usize, Option<usize>),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            ParseError::ImsufficentBuffer(width, None) => {
                write!(f, "Undefined buffer size, required {}", width)
            }
            ParseError::ImsufficentBuffer(width, Some(max)) => write!(
                f,
                "Insufficient buffer size, required {} only {} available",
                width, max
            ),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
/// An error produced while parsing fixed width data.
pub enum Error {
    /// An IO error occured while reading the data.
    IOError(io::Error),
    /// An error occured while parsing the data.
    ParserError(ParseError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Error::IOError(ref e) => write!(f, "{}", e),
            Error::ParserError(ref e) => e.fmt(f),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IOError(e)
    }
}

impl From<ParseError> for Error {
    fn from(kind: ParseError) -> Self {
        Error::ParserError(kind)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::IOError(ref e) => Some(e),
            Error::ParserError(ref _e) => None,
        }
    }

    #[allow(deprecated)]
    fn cause(&self) -> Option<&(dyn StdError)> {
        self.source()
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod test {
    use super::*;

    #[test]
    fn check_buffer_insufficient() {
        let error = Error::from(ParseError::ImsufficentBuffer(10, Some(5)));

        assert!(matches!(error, Error::ParserError(_)));
        assert_eq!(
            error.to_string(),
            String::from("Insufficient buffer size, required 10 only 5 available")
        );
        assert!(matches!(error.source(), None));
        assert!(matches!(error.cause(), None));
    }

    #[test]
    fn check_buffer_undefined() {
        let error = Error::from(ParseError::ImsufficentBuffer(10, None));

        assert_eq!(
            error.to_string(),
            String::from("Undefined buffer size, required 10")
        );
    }

    #[test]
    fn check_io_error() {
        let io_error = io::Error::new(io::ErrorKind::Other, "test");
        let error = Error::from(io_error);

        assert!(matches!(error, Error::IOError(_)));
        assert_eq!(error.to_string(), String::from("test"));
        assert!(matches!(error.source(), Some(_)));
        assert!(matches!(error.cause(), Some(_)));
    }
}
