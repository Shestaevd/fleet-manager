use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use tokio::task::JoinHandle;
use tokio::time::{Interval, MissedTickBehavior, interval};

use crate::domain::model::{FileMetaWindow, Shared};
use fleet_model::FileMeta;
use crate::fs::fs_io::fs_scan;

pub fn fs_scan_process(
    shared: Arc<Shared>,
    lookup_window: Arc<RwLock<FileMetaWindow>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let depth: u8 = shared.config.scan_depth;
        let mut ticker: Interval = interval(shared.config.watch_interval());
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;

            let dir: PathBuf = match lookup_window.read() {
                Ok(window) => window.search_from.path.clone(),
                Err(_) => {
                    tracing::error!("lookup window lock is poisoned");
                    continue;
                }
            };

            let scanned = tokio::task::spawn_blocking({
                let dir: PathBuf = dir.clone();
                move || {
                    let mut entries: Vec<FileMeta> = Vec::new();
                    fs_scan(dir, depth, &mut entries).map(|()| entries)
                }
            })
            .await;

            let fresh: Vec<FileMeta> = match scanned {
                Ok(Ok(entries)) => entries,
                Ok(Err(err)) => {
                    tracing::warn!("overwatch scan of {} failed: {err}", dir.display());
                    continue;
                }
                Err(err) => {
                    tracing::error!("overwatch scan task failed: {err}");
                    continue;
                }
            };

            let Ok(mut window) = lookup_window.write() else {
                tracing::error!("lookup window lock is poisoned, stopping overwatch");
                continue;
            };
            if window.search_from.path != dir {
                continue;
            }
            window.loaded_file_meta_window = fresh;
        }
    })
}
