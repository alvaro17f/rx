use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    GitPullFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {e}"),
            Error::GitPullFailed => write!(f, "Failed to pull changes"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::GitPullFailed => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_io_error() {
        let err = Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        assert!(err.to_string().contains("IO error"));
    }

    #[test]
    fn display_git_pull_failed() {
        assert_eq!(Error::GitPullFailed.to_string(), "Failed to pull changes");
    }

    #[test]
    fn source_io_error() {
        let err = Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        assert!(std::error::Error::source(&err).is_some());
    }

    #[test]
    fn source_git_pull_failed() {
        assert!(std::error::Error::source(&Error::GitPullFailed).is_none());
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }
}