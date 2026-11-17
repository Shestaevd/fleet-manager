use std::future::Future;
use std::io;
use std::sync::Arc;
use std::time::Duration;

use fleet_util::fs::{executable_dir, read_file};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::task::{JoinError, JoinHandle};
use tokio::time::timeout;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::server::TlsStream;

use crate::domain::config::ServerConfig;

pub type Reader = ReadHalf<TlsStream<TcpStream>>;
pub type Writer = WriteHalf<TlsStream<TcpStream>>;

pub fn run<F, Fut>(config: ServerConfig, session: F) -> JoinHandle<io::Result<()>>
where
    F: Fn(Reader, Writer) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), JoinError>> + Send + 'static,
{
    tokio::spawn(server_loop(config, session))
}

async fn server_loop<F, Fut>(config: ServerConfig, session: F) -> io::Result<()>
where
    F: Fn(Reader, Writer) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), JoinError>> + Send + 'static,
{
    let acceptor: TlsAcceptor = TlsAcceptor::from(Arc::new(tls_config(&config).await?));
    let listener: TcpListener =
        TcpListener::bind((config.host.as_str(), config.port)).await?;
    let permits: Arc<Semaphore> = Arc::new(Semaphore::new(config.max_connections));

    let session: Arc<F> = Arc::new(session);

    tracing::info!(
        "listening on {}:{}",
        config.host,
        config.port
    );

    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(accepted) => accepted,
            Err(err) => {
                tracing::warn!("accept failed: {err}");
                continue;
            }
        };

        let Ok(permit) = permits.clone().try_acquire_owned() else {
            tracing::warn!("connection limit reached, dropping {peer}");
            continue;
        };

        let acceptor: TlsAcceptor = acceptor.clone();
        let session: Arc<F> = session.clone();

        tokio::spawn(async move {
            let _permit = permit;
            let handshake_timeout: Duration = config.handshake_timeout;
            let tls: TlsStream<TcpStream> = match accept(stream, &acceptor, handshake_timeout).await {
                Ok(tls) => tls,
                Err(err) => {
                    tracing::debug!("{peer} dropped due to: {err}");
                    return;
                }
            };

            tracing::debug!("{peer} connected");
            let (reader, writer) = tokio::io::split(tls);
            let ended: Result<(), JoinError> =
                session(reader, writer).await;
            match ended {
                Ok(()) => tracing::debug!("{peer} disconnected"),
                Err(err) => tracing::warn!("session of {peer} ended unexpectedly: {err}"),
            }
        });
    }
}

async fn accept(
    stream: TcpStream,
    acceptor: &TlsAcceptor,
    handshake_timeout: Duration,
) -> io::Result<TlsStream<TcpStream>> {
    stream.set_nodelay(true)?;
    timeout(handshake_timeout, acceptor.accept(stream))
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "tls handshake timed out"))?
}

async fn tls_config(config: &ServerConfig) -> io::Result<rustls::ServerConfig> {
    let dir = executable_dir().map_err(io::Error::other)?;
    let cert_pem: Vec<u8> = read_file(dir.join(&config.cert_path))
        .await
        .map_err(io::Error::other)?;
    let key_pem: Vec<u8> = read_file(dir.join(&config.key_path))
        .await
        .map_err(io::Error::other)?;

    let certs: Vec<CertificateDer<'static>> =
        rustls_pemfile::certs(&mut cert_pem.as_slice()).collect::<Result<_, _>>()?;
    let key: PrivateKeyDer<'static> = rustls_pemfile::private_key(&mut key_pem.as_slice())?
        .ok_or_else(|| io::Error::other("no private key in the key file"))?;

    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(io::Error::other)
}
