use std::fmt;

/// Application-level error type.
///
/// Uses a manual `Display` + `Error` impl to keep the dependency tree empty.
/// `From<std::io::Error>` enables the `?` operator for IO fallibility.
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    GitPullFailed,
    InvalidArgs,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {e}"),
            Error::GitPullFailed => write!(f, "Failed to pull changes"),
            Error::InvalidArgs => write!(f, "Invalid arguments"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::GitPullFailed | Error::InvalidArgs => None,
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
    fn display_io_error_contains_io_prefix() {
        let err = Error::Io(std::io::Error::other("test"));
        assert!(err.to_string().contains("IO error"));
    }

    #[test]
    fn display_git_pull_failed_matches_literal() {
        assert_eq!(Error::GitPullFailed.to_string(), "Failed to pull changes");
    }

    #[test]
    fn source_io_error_yields_some() {
        let err = Error::Io(std::io::Error::other("test"));
        assert!(std::error::Error::source(&err).is_some());
    }

    #[test]
    fn source_git_pull_failed_yields_none() {
        assert!(std::error::Error::source(&Error::GitPullFailed).is_none());
    }

    #[test]
    fn display_invalid_args_matches_literal() {
        assert_eq!(Error::InvalidArgs.to_string(), "Invalid arguments");
    }

    #[test]
    fn source_invalid_args_yields_none() {
        assert!(std::error::Error::source(&Error::InvalidArgs).is_none());
    }

    #[test]
    fn from_io_error_produces_io_variant() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file");
        let err: Error = io_err.into();
        assert_eq!(
            std::mem::discriminant(&err),
            std::mem::discriminant(&Error::Io(std::io::Error::other("")))
        );
    }
}
