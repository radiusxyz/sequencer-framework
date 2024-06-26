#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Open,
    Get,
    GetMut,
    Put,
    Delete,
    Commit,
    SerializeKey,
    SerializeValue,
    DeserializeValue,
    KeyDoesNotExist,
    Initialize,
}

pub(crate) enum ErrorSource {
    Bincode(bincode::Error),
    RocksDB(rocksdb::Error),
    NoneType,
    Custom(String),
}

impl From<bincode::Error> for ErrorSource {
    fn from(value: bincode::Error) -> Self {
        Self::Bincode(value)
    }
}

impl From<rocksdb::Error> for ErrorSource {
    fn from(value: rocksdb::Error) -> Self {
        Self::RocksDB(value)
    }
}

impl std::fmt::Display for ErrorSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bincode(error) => write!(f, "{}", error),
            Self::RocksDB(error) => write!(f, "{}", error),
            Self::NoneType => write!(f, "The value returned None"),
            Self::Custom(error) => write!(f, "{}", error),
        }
    }
}

pub struct Error {
    kind: ErrorKind,
    source: ErrorSource,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.source)
    }
}

impl std::error::Error for Error {}

impl<E> From<(ErrorKind, E)> for Error
where
    E: Into<ErrorSource>,
{
    fn from(value: (ErrorKind, E)) -> Self {
        Self {
            kind: value.0,
            source: value.1.into(),
        }
    }
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn custom<E>(kind: ErrorKind, error: E) -> Self
    where
        E: std::fmt::Display,
    {
        Self {
            kind,
            source: ErrorSource::Custom(error.to_string()),
        }
    }
}
