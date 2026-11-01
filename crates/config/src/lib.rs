use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use fleet_model::ConfigReadingError;
use serde::de::DeserializeOwned;

pub fn read_config<T: DeserializeOwned>(file_name: &str) -> Result<T, ConfigReadingError> {
    read_config_from(&executable_dir()?.join(file_name))
}
pub fn read_config_or_create<T: DeserializeOwned>(
    file_name: &str,
    default_toml: &str,
) -> Result<(T, bool), ConfigReadingError> {
    read_config_or_create_at(&executable_dir()?.join(file_name), default_toml)
}

pub fn read_config_or_create_at<T: DeserializeOwned>(
    path: &Path,
    default_toml: &str,
) -> Result<(T, bool), ConfigReadingError> {
    let created = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(mut file) => {
            file.write_all(default_toml.as_bytes())
                .map_err(|source| ConfigReadingError::Write {
                    path: path.to_path_buf(),
                    source,
                })?;
            true
        }
        Err(err) if err.kind() == ErrorKind::AlreadyExists => false,
        Err(source) => {
            return Err(ConfigReadingError::Write {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    read_config_from(path).map(|value| (value, created))
}

pub fn read_config_from<T: DeserializeOwned>(path: &Path) -> Result<T, ConfigReadingError> {
    let text = std::fs::read_to_string(path).map_err(|source| ConfigReadingError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str(&text).map_err(|source| ConfigReadingError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

fn executable_dir() -> Result<PathBuf, ConfigReadingError> {
    let exe = std::env::current_exe().map_err(ConfigReadingError::ExecutablePath)?;
    let exe = exe
        .canonicalize()
        .map_err(ConfigReadingError::ExecutablePath)?;
    exe.parent().map(Path::to_path_buf).ok_or_else(|| {
        ConfigReadingError::ExecutablePath(std::io::Error::other(
            "executable has no parent directory",
        ))
    })
}

#[cfg(test)]
mod test;
