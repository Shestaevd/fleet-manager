use std::io::ErrorKind;
use std::path::PathBuf;

use fleet_model::error::FsError;
use fleet_util::fs::read_file;

const USER_STORE_DIR: &str = "users";

/// The content of a file, or `None` if it does not exist.
pub(crate) async fn read_optional(path: PathBuf) -> Result<Option<Vec<u8>>, FsError> {
    match read_file(path).await {
        Ok(content) => Ok(Some(content)),
        Err(FsError::CanNotRead { source, .. }) if source.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

pub(crate) async fn exists(path: PathBuf) -> Result<bool, FsError> {
    tokio::fs::try_exists(&path)
        .await
        .map_err(|source| FsError::CanNotReadMetadata { path, source })
}

/// Deletes a directory with everything in it; succeeds if it does not exist.
pub(crate) async fn remove_dir_if_exists(dir: PathBuf) -> Result<(), FsError> {
    match tokio::fs::remove_dir_all(&dir).await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(source) => Err(FsError::CanNotDelete { path: dir, source }),
    }
}
