use std::env;
use std::path::PathBuf;

use fleet_model::error::FsError;
use tokio::fs;

pub async fn read_file(path: PathBuf) -> Result<Vec<u8>, FsError> {
    fs::read(&path).await.map_err(|err| FsError::CanNotRead { path, source: err })
}

pub async fn write_file(path: PathBuf, content: &[u8]) -> Result<(), FsError> {
    fs::write(&path, content).await.map_err(|err| FsError::CanNotWrite { path, source: err })
}

pub async fn delete_file(path: PathBuf) -> Result<(), FsError> {
    fs::remove_file(&path).await.map_err(|err| FsError::CanNotDelete { path, source: err } )
}

pub async fn write_secret_file(path: PathBuf, content: &[u8]) -> Result<(), FsError> {
    use std::os::unix::fs::PermissionsExt;

    let result: std::io::Result<()> = async {
        fs::write(&path, content).await?;
        fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).await
    }
    .await;
    result.map_err(|err| FsError::CanNotWrite { path, source: err })
}

pub async fn create_dir_if_not_exist(path: PathBuf) -> Result<(), FsError> {
    fs::create_dir_all(&path).await.map_err(|err| FsError::CanNotWrite { path, source: err })
}

pub fn executable_dir() -> Result<PathBuf, FsError> {
    let exe: PathBuf = env::current_exe().map_err(|err| FsError::CanNotGetExecutablePath { source: err })?;
    exe.parent().map(PathBuf::from).ok_or_else(|| FsError::CanNotGetExecutablePath {
        source: std::io::Error::other("executable has no parent directory"),
    })
}
