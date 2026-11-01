use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use fleet_model::error::FsError;

use crate::domain::model::{FileKind, FileMeta};
use crate::fs::fs_io::{file_meta_of, fs_home, fs_root, fs_scan};

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn scan(dir: PathBuf) -> Vec<FileMeta> {
    let mut out = Vec::new();
    fs_scan(dir, u8::MAX, &mut out).unwrap();
    out
}

fn find<'a>(entries: &'a [FileMeta], path: &Path) -> &'a FileMeta {
    entries
        .iter()
        .find(|e| e.path == path)
        .unwrap_or_else(|| panic!("{} not found in scan", path.display()))
}

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("fleet-agent-{}-{name}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn file_path_is_not_a_directory() {
    let file = project_root().join("Cargo.toml");
    let mut out = Vec::new();
    let r = fs_scan(file, u8::MAX, &mut out);
    assert!(matches!(r, Err(FsError::NotADirectory { .. })));
    assert!(out.is_empty());
}

#[test]
fn scans_project_sources_recursively() {
    let root = project_root();
    let entries = scan(root.join("crates/agent/src"));

    assert_eq!(
        find(&entries, &root.join("crates/agent/src/main.rs")).kind,
        FileKind::File
    );
    assert_eq!(
        find(&entries, &root.join("crates/agent/src/fs/fs_io.rs")).kind,
        FileKind::File
    );
    assert_eq!(
        find(&entries, &root.join("crates/agent/src/domain")).kind,
        FileKind::Dir
    );
}

#[test]
fn file_metadata_matches_filesystem() {
    let root = project_root();
    let entries = scan(root.join("crates"));
    let manifest = root.join("crates/model/Cargo.toml");
    let meta = fs::metadata(&manifest).unwrap();
    let entry = find(&entries, &manifest);

    assert_eq!(entry.kind, FileKind::File);
    assert_eq!(entry.size, meta.len());
    assert_eq!(entry.modified, meta.modified().ok());
    assert_eq!(entry.readonly, meta.permissions().readonly());
    assert!(entry.symlink_target.is_none());
    assert!(!entry.hidden);
}

#[test]
fn dot_files_are_hidden() {
    let tmp = TempDir::new("hidden");
    fs::write(tmp.0.join(".secret"), b"x").unwrap();
    fs::write(tmp.0.join("visible"), b"x").unwrap();
    let entries = scan(tmp.0.clone());

    assert!(find(&entries, &tmp.0.join(".secret")).hidden);
    assert!(!find(&entries, &tmp.0.join("visible")).hidden);
}

fn scan_to_depth(dir: PathBuf, depth: u8) -> Vec<FileMeta> {
    let mut out = Vec::new();
    fs_scan(dir, depth, &mut out).unwrap();
    out
}

#[test]
fn depth_limits_nested_directories() {
    let tmp = TempDir::new("depth");
    fs::create_dir_all(tmp.0.join("a/b/c")).unwrap();
    fs::write(tmp.0.join("a/b/c/deep.txt"), b"x").unwrap();
    let entries = scan_to_depth(tmp.0.clone(), 1);

    find(&entries, &tmp.0.join("a"));
    find(&entries, &tmp.0.join("a/b"));
    assert!(
        !entries
            .iter()
            .any(|e| e.path == tmp.0.join("a/b/c"))
    );

    let all = scan_to_depth(tmp.0.clone(), u8::MAX);
    find(&all, &tmp.0.join("a/b/c/deep.txt"));
}

#[test]
fn symlinked_directories_are_not_followed() {
    let tmp = TempDir::new("loop");
    fs::create_dir_all(tmp.0.join("real")).unwrap();
    fs::write(tmp.0.join("real/file.txt"), b"x").unwrap();
    std::os::unix::fs::symlink(&tmp.0, tmp.0.join("real/loop")).unwrap();
    std::os::unix::fs::symlink(tmp.0.join("real"), tmp.0.join("alias")).unwrap();
    let entries = scan(tmp.0.clone());

    assert_eq!(
        find(&entries, &tmp.0.join("real/loop")).kind,
        FileKind::Symlink
    );
    assert_eq!(find(&entries, &tmp.0.join("alias")).kind, FileKind::Symlink);
    assert_eq!(
        entries
            .iter()
            .filter(|e| e.path.file_name() == Some(OsStr::new("file.txt")))
            .count(),
        1
    );
    // fs_scan also lists parent directories, so count only what is inside the temp dir
    assert_eq!(entries.iter().filter(|e| e.path != tmp.0 && e.path.starts_with(&tmp.0)).count(), 4);
}

#[test]
fn scan_around_reads_children_and_parents_within_depth() {
    let tmp = TempDir::new("around");
    fs::create_dir_all(tmp.0.join("a/b/c/d")).unwrap();
    fs::write(tmp.0.join("a/b/c/d/deep.txt"), b"x").unwrap();
    fs::write(tmp.0.join("a/b/sibling.txt"), b"x").unwrap();
    fs::write(tmp.0.join("a/top.txt"), b"x").unwrap();
    let start = tmp.0.join("a/b/c");

    let mut one = Vec::new();
    fs_scan(start.clone(), 1, &mut one).unwrap();
    find(&one, &tmp.0.join("a/b/c/d/deep.txt"));
    find(&one, &tmp.0.join("a/b/sibling.txt"));
    find(&one, &tmp.0.join("a/b/c"));
    assert!(!one.iter().any(|e| e.path.file_name().is_some_and(|n| n == "top.txt")));

    let mut two = Vec::new();
    fs_scan(start, 2, &mut two).unwrap();
    find(&two, &tmp.0.join("a/top.txt"));
    assert_eq!(two.iter().filter(|e| e.path.file_name().is_some_and(|n| n == "deep.txt")).count(), 1);
}

#[test]
fn fs_root_is_a_directory_with_dir_meta() {
    let root = fs_root();
    assert!(root.is_dir());
    assert_eq!(root, Path::new("/"));
    assert_eq!(file_meta_of(root.clone()).unwrap().kind, FileKind::Dir);
    assert_eq!(file_meta_of(root).unwrap().hidden, false);
}

#[test]
fn home_is_a_directory() {
    assert!(fs_home().is_dir());
}

#[test]
fn unreadable_nested_directory_is_skipped() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new("unreadable");
    fs::create_dir_all(tmp.0.join("locked")).unwrap();
    fs::write(tmp.0.join("locked/hidden.txt"), b"x").unwrap();
    fs::write(tmp.0.join("ok.txt"), b"x").unwrap();
    fs::set_permissions(tmp.0.join("locked"), fs::Permissions::from_mode(0o000)).unwrap();

    let mut out = Vec::new();
    let result = fs_scan(tmp.0.clone(), u8::MAX, &mut out);
    fs::set_permissions(tmp.0.join("locked"), fs::Permissions::from_mode(0o755)).unwrap();

    result.unwrap();
    find(&out, &tmp.0.join("ok.txt"));
    find(&out, &tmp.0.join("locked"));
}
