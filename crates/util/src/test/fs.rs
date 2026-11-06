use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use fleet_model::error::FsError;

use crate::fs::{create_dir_if_not_exist, executable_dir, read_file, write_file, write_secret_file};

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("fleet-util-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test]
async fn written_content_can_be_read_back() {
    let tmp = TempDir::new("roundtrip");
    let path = tmp.0.join("a.bin");
    write_file(path.clone(), &[1, 2, 3]).await.unwrap();
    assert_eq!(read_file(path).await.unwrap(), vec![1, 2, 3]);
}

#[tokio::test]
async fn reading_a_missing_file_and_writing_into_a_missing_directory_fail_with_the_path() {
    let tmp = TempDir::new("errors");
    let missing = tmp.0.join("missing.bin");
    assert!(matches!(read_file(missing.clone()).await, Err(FsError::CanNotRead { path, .. }) if path == missing));

    let nested = tmp.0.join("no/such/dir/a.bin");
    assert!(matches!(write_file(nested.clone(), b"x").await, Err(FsError::CanNotWrite { path, .. }) if path == nested));
}

#[tokio::test]
async fn secret_files_are_owner_only_even_when_overwritten() {
    let tmp = TempDir::new("secret");
    let path = tmp.0.join("secret");
    write_file(path.clone(), b"old").await.unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    write_secret_file(path.clone(), b"new").await.unwrap();
    assert_eq!(read_file(path.clone()).await.unwrap(), b"new");
    assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
}

#[tokio::test]
async fn creating_a_directory_twice_is_fine_and_creates_parents() {
    let tmp = TempDir::new("dirs");
    let dir = tmp.0.join("a/b/c");
    create_dir_if_not_exist(dir.clone()).await.unwrap();
    create_dir_if_not_exist(dir.clone()).await.unwrap();
    assert!(dir.is_dir());
}

#[test]
fn executable_dir_exists() {
    assert!(executable_dir().unwrap().is_dir());
}
