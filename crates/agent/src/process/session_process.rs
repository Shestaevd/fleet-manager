use std::sync::Arc;
use std::time::Duration;

use crate::domain::model::{IntoHeartBeat, Shared};
use crate::process::telemetry::collect_telemetry;
use fleet_protocol::agent_receiver::HeartBeatMessage;
use fleet_protocol::protocol::Package;
use sysinfo::System;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::sync::{Mutex, MutexGuard};
use tokio::task::JoinHandle;
use tokio::time::{Instant, Interval, interval_at};

pub fn heart_beat_process<W>(shared: Arc<Shared>, writer: Arc<Mutex<W>>) -> JoinHandle<()>
where
    W: AsyncWrite + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let period: Duration = shared.config.heart_beat_interval();
        let mut system: System = System::new();
        let mut ticker: Interval = interval_at(Instant::now(), period);
        loop {
            ticker.tick().await;

            let heart_beat: HeartBeatMessage =
                collect_telemetry(&mut system).into_heart_beat(shared.config.agent_id);
            let bytes: Vec<u8> = match heart_beat.encode() {
                Ok(bytes) => bytes,
                Err(err) => {
                    tracing::error!("can not encode heart beat, continuing: {err}");
                    continue;
                }
            };

            let mut writer: MutexGuard<W> = writer.lock().await;
            if let Err(err) = writer.write_all(&bytes).await {
                tracing::warn!("stream is broken, stopping heart beat: {err}");
                return;
            }
            if let Err(err) = writer.flush().await {
                tracing::warn!("stream is broken, stopping heart beat: {err}");
                return;
            }
        }
    })
}

pub fn session_process<R, W>(
    shared: Arc<Shared>,
    reader: Arc<R>,
    writer: Arc<Mutex<W>>,
) -> JoinHandle<()>
where
    W: AsyncWrite + Unpin + Send + 'static,
    R: AsyncRead + Unpin + Send + 'static,
{
    let shared = shared.clone();
    let writer_clone = writer.clone();
    heart_beat_process(shared, writer_clone)
}
