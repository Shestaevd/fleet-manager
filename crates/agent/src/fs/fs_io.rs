use crate::domain::model::{FileKind, FileMeta};
use fleet_model::error::FsError;
use std::path::PathBuf;
use std::{env, fs};

pub fn write_file(file_path: PathBuf, content: &[u8]) -> Result<(), FsError> {
    fs::write(&file_path, content).map_err(|err| FsError::Write {
        path: file_path,
        source: err,
    })
}

pub fn read_file(file_path: PathBuf) -> Result<Vec<u8>, FsError> {
    fs::read(&file_path).map_err(|err| FsError::Read {
        path: file_path,
        source: err,
    })
}

pub fn read_pem() -> Result<Vec<u8>, FsError> {
    let executable_path: PathBuf = get_executable_path()?;
    read_file(executable_path.join("host.pem"))
}

fn fs_read_with_depth(
    dir: PathBuf,
    current_depth: u8,
    depth: u8,
    scan_vec: &mut Vec<FileMeta>,
) -> Result<(), FsError> {
    if !dir.is_dir() {
        return Err(FsError::NotADirectory { path: dir });
    }
    let read_dir: fs::ReadDir = dir.read_dir().map_err(|err| FsError::CanNotReadDirectory {
        path: dir,
        source: err,
    })?;
    for entry in read_dir.flatten() {
        let path: PathBuf = entry.path();
        let Ok(file_type) = entry.file_type() else {
            tracing::debug!("skipping unreadable entry {}", path.display());
            continue;
        };
        if current_depth < depth && file_type.is_dir() {
            if let Err(err) = fs_read_with_depth(path.clone(), current_depth + 1, depth, scan_vec) {
                tracing::debug!("skipping directory {}: {err}", path.display());
            }
        }
        match file_meta_of(path.clone()) {
            Ok(file_meta) => scan_vec.push(file_meta),
            Err(err) => tracing::debug!("skipping entry {}: {err}", path.display()),
        }
    }
    Ok(())
}

fn get_executable_path() -> Result<PathBuf, FsError> {
    let exe: PathBuf = env::current_exe().map_err(|err| FsError::CanNotGetExecutablePath { source: err })?;
    exe.parent()
        .map(PathBuf::from)
        .ok_or_else(|| FsError::CanNotGetExecutablePath {
            source: std::io::Error::other("executable has no parent directory"),
        })
}

pub fn fs_home() -> PathBuf {
    env::home_dir().unwrap_or_else(fs_root)
}

pub fn fs_root() -> PathBuf {
    PathBuf::from("/")
}

pub fn file_meta_of(path: PathBuf) -> Result<FileMeta, FsError> {
    let metadata: fs::Metadata = fs::symlink_metadata(&path).map_err(|err| FsError::CanNotReadMetadata {
        path: path.clone(),
        source: err,
    })?;
    let file_type: fs::FileType = metadata.file_type();
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
        fs::read_link(&path).ok()
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

pub fn fs_scan(
    dir: PathBuf,
    depth: u8,
    scan_vec: &mut Vec<FileMeta>,
) -> Result<(), FsError> {
    fs_read_with_depth(dir.clone(), 0, depth, scan_vec)?;

    for parent in dir
        .ancestors()
        .skip(1)
        .take(depth as usize)
        .take_while(|p| !p.as_os_str().is_empty())
    {
        fs_read_with_depth(parent.to_path_buf(), 0, 0, scan_vec)?;
    }
    Ok(())
}

fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}
