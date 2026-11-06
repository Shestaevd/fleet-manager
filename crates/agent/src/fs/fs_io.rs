use crate::domain::error::AgentError;
use fleet_crypto::validate_token_hash;
use fleet_model::error::FsError;
use fleet_model::{FileKind, FileMeta};
use fleet_util::fs::{delete_file, executable_dir, read_file, write_file, write_secret_file};
use std::env;
use std::fs::{FileType, Metadata};
use std::path::PathBuf;
use tokio::fs;

const TOKEN_HASH_FILE: &str = ".token.txt";
const PROBATION: &str = ".probation";
const HOST_PEM_HASH_FILE: &str = "host.pem";

pub async fn read_pem() -> Result<Vec<u8>, FsError> {
    let executable_path: PathBuf = executable_dir()?;
    read_file(executable_path.join(HOST_PEM_HASH_FILE)).await
}

pub async fn read_token_hash() -> Result<Vec<u8>, AgentError> {
    read_token_hash_from(executable_dir()?.join(TOKEN_HASH_FILE)).await
}

pub async fn read_is_in_probation() -> Result<bool, FsError> {
    let executable_path: PathBuf = executable_dir()?;
    Ok(executable_path.join(PROBATION).exists())
}

pub async fn write_in_probation() -> Result<(), FsError> {
    let executable_path: PathBuf = executable_dir()?;
    write_file(executable_path.join(PROBATION), &[]).await
}

pub async fn delete_in_probation(path: PathBuf) -> Result<(), FsError> {
    let executable_path: PathBuf = executable_dir()?;
    delete_file(executable_path.join(PROBATION)).await
}

pub async fn write_token_hash(token_hash: &[u8]) -> Result<(), AgentError> {
    write_token_hash_to(executable_dir()?.join(TOKEN_HASH_FILE), token_hash).await
}

pub async fn read_token_hash_from(path: PathBuf) -> Result<Vec<u8>, AgentError> {
    let token_hash: Vec<u8> = read_file(path).await?;
    validate_token_hash(&token_hash)?;
    Ok(token_hash)
}

pub async fn write_token_hash_to(path: PathBuf, token_hash: &[u8]) -> Result<(), AgentError> {
    validate_token_hash(token_hash)?;
    write_secret_file(path, token_hash).await?;
    Ok(())
}

async fn is_dir(path: &PathBuf) -> bool {
    fs::metadata(path).await.map(|metadata| metadata.is_dir()).unwrap_or(false)
}

async fn fs_read_with_depth(
    dir: PathBuf,
    depth: u8,
    scan_vec: &mut Vec<FileMeta>,
) -> Result<(), FsError> {
    if !is_dir(&dir).await {
        return Err(FsError::NotADirectory { path: dir });
    }

    let mut pending: Vec<(PathBuf, u8)> = vec![(dir.clone(), 0)];
    while let Some((current, current_depth)) = pending.pop() {
        let mut read_dir: fs::ReadDir = match fs::read_dir(&current).await {
            Ok(read_dir) => read_dir,
            Err(err) if current == dir => {
                return Err(FsError::CanNotReadDirectory {
                    path: current,
                    source: err,
                });
            }
            Err(err) => {
                tracing::debug!("skipping directory {}: {err}", current.display());
                continue;
            }
        };

        loop {
            let entry: fs::DirEntry = match read_dir.next_entry().await {
                Ok(Some(entry)) => entry,
                Ok(None) => break,
                Err(err) => {
                    tracing::debug!("stopped reading {}: {err}", current.display());
                    break;
                }
            };
            let path: PathBuf = entry.path();
            let Ok(file_type) = entry.file_type().await else {
                tracing::debug!("skipping unreadable entry {}", path.display());
                continue;
            };
            if current_depth < depth && file_type.is_dir() {
                pending.push((path.clone(), current_depth + 1));
            }
            match file_meta_of(path.clone()).await {
                Ok(file_meta) => scan_vec.push(file_meta),
                Err(err) => tracing::debug!("skipping entry {}: {err}", path.display()),
            }
        }
    }
    Ok(())
}

pub fn fs_home() -> PathBuf {
    env::home_dir().unwrap_or_else(fs_root)
}

pub fn fs_root() -> PathBuf {
    PathBuf::from("/")
}

pub async fn file_meta_of(path: PathBuf) -> Result<FileMeta, FsError> {
    let metadata: Metadata = fs::symlink_metadata(&path).await.map_err(|err| FsError::CanNotReadMetadata {
        path: path.clone(),
        source: err,
    })?;
    let file_type: FileType = metadata.file_type();
    let kind: FileKind = if file_type.is_symlink() {
        FileKind::Symlink
    } else if file_type.is_dir() {
        FileKind::Dir
    } else if file_type.is_file() {
        FileKind::File
    } else {
        FileKind::Other
    };
    let name: String = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let symlink_target: Option<PathBuf> = if kind == FileKind::Symlink {
        fs::read_link(&path).await.ok()
    } else {
        None
    };
    Ok(FileMeta {
        hidden: is_hidden(&name),
        size: metadata.len(),
        modified: metadata.modified().ok(),
        accessed: metadata.accessed().ok(),
        created: metadata.created().ok(),
        readonly: metadata.permissions().readonly(),
        symlink_target,
        kind,
        path,
    })
}

pub async fn fs_scan(
    dir: PathBuf,
    depth: u8,
    scan_vec: &mut Vec<FileMeta>,
) -> Result<(), FsError> {
    fs_read_with_depth(dir.clone(), depth, scan_vec).await?;

    for parent in dir
        .ancestors()
        .skip(1)
        .take(depth as usize)
        .take_while(|p| !p.as_os_str().is_empty())
    {
        fs_read_with_depth(parent.to_path_buf(), 0, scan_vec).await?;
    }
    Ok(())
}

fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}
