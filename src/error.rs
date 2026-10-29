use std::fmt;
use std::io;
use std::process::ExitCode;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Parse { line: usize, reason: &'static str },
    InvalidProfileName(String),
    ReservedNameUsed(String),
    ProfileNotFound(String),
    ProfileAlreadyExists(String),
    MissingDependency(&'static str),
    CommandFailed { cmd: String, status: i32 },
    VersionParseError(String),
    SelfUpdateFailed(String),
    OperationCancelled,
    Generic(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Parse { line, reason } => {
                write!(f, "config parse error on line {line}: {reason}")
            }
            Self::InvalidProfileName(name) => write!(f, "invalid profile name: '{name}'"),
            Self::ReservedNameUsed(name) => write!(
                f,
                "'{name}' is a reserved command and cannot be used as a profile name"
            ),
            Self::ProfileNotFound(name) => write!(f, "profile '{name}' not found"),
            Self::ProfileAlreadyExists(name) => write!(f, "profile '{name}' already exists"),
            Self::MissingDependency(tool) => write!(f, "missing essential dependency: {tool}"),
            Self::CommandFailed { cmd, status } => {
                write!(f, "command '{cmd}' failed with status {status}")
            }
            Self::VersionParseError(ver) => write!(f, "failed to parse version string: '{ver}'"),
            Self::SelfUpdateFailed(msg) => write!(f, "self-update failed: {msg}"),
            Self::OperationCancelled => write!(f, "operation cancelled by user"),
            Self::Generic(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    #[inline]
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl Error {
    #[inline]
    #[must_use]
    pub const fn exit_code(&self) -> ExitCode {
        match self {
            Self::OperationCancelled => ExitCode::SUCCESS,
            _ => ExitCode::FAILURE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert_eq!(
            Error::InvalidProfileName("bad name".into()).to_string(),
            "invalid profile name: 'bad name'"
        );
        assert_eq!(
            Error::ReservedNameUsed("list".into()).to_string(),
            "'list' is a reserved command and cannot be used as a profile name"
        );
        assert_eq!(
            Error::ProfileNotFound("missing".into()).to_string(),
            "profile 'missing' not found"
        );
        assert_eq!(
            Error::ProfileAlreadyExists("exist".into()).to_string(),
            "profile 'exist' already exists"
        );
        assert_eq!(
            Error::MissingDependency("git").to_string(),
            "missing essential dependency: git"
        );
        assert_eq!(
            Error::CommandFailed {
                cmd: "git clone".into(),
                status: 128
            }
            .to_string(),
            "command 'git clone' failed with status 128"
        );
        assert_eq!(
            Error::VersionParseError("abc".into()).to_string(),
            "failed to parse version string: 'abc'"
        );
        assert_eq!(
            Error::SelfUpdateFailed("permission denied".into()).to_string(),
            "self-update failed: permission denied"
        );
        assert_eq!(
            Error::OperationCancelled.to_string(),
            "operation cancelled by user"
        );
        assert_eq!(Error::Generic("unknown".into()).to_string(), "unknown");
        assert_eq!(
            Error::Parse {
                line: 42,
                reason: "unexpected EOF"
            }
            .to_string(),
            "config parse error on line 42: unexpected EOF"
        );
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "disk read error");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
        assert_eq!(err.to_string(), "I/O error: disk read error");
        assert!(std::error::Error::source(&err).is_some());
    }

    #[test]
    fn test_exit_codes() {
        assert_eq!(Error::OperationCancelled.exit_code(), ExitCode::SUCCESS);
        assert_eq!(
            Error::InvalidProfileName("bad".into()).exit_code(),
            ExitCode::FAILURE
        );
    }
}
