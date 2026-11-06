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
        let mut ticker: Interval = interval(shared.config.watch_interval);
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

            let mut entries: Vec<FileMeta> = Vec::new();
            let fresh: Vec<FileMeta> = match fs_scan(dir.clone(), depth, &mut entries).await {
                Ok(()) => entries,
                Err(err) => {
                    tracing::warn!("overwatch scan of {} failed: {err}", dir.display());
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
