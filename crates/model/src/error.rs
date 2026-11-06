use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigReadingError {
    #[error("cannot determine executable location: {0}")]
    ExecutablePath(#[source] std::io::Error),
    #[error("cannot read config file {}: {source}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot write default config file {}: {source}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot parse config file {}: {source}", path.display())]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
}

#[derive(Debug, Error)]
pub enum FsError {
    #[error("path is not a directory: {}", path.display())]
    NotADirectory { path: PathBuf },
    #[error("can not read a directory {}: {source}", path.display())]
    CanNotReadDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("path is not valid UTF-8: {}", path.display())]
    NonUtf8Path { path: PathBuf },
    #[error("can not read a metadata of {}: {source}", path.display())]
    CanNotReadMetadata {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("can not write file to {}: {source}", path.display())]
    CanNotWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("can not read file from {}: {source}", path.display())]
    CanNotRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("can not delete file from {}: {source}", path.display())]
    CanNotDelete {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("can not get filepath for executable: {source}")]
    CanNotGetExecutablePath {
        #[source]
        source: std::io::Error,
    }
}


