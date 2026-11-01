use failsafe::backoff::Constant;
use failsafe::failure_policy::ConsecutiveFailures;
use failsafe::futures::CircuitBreaker;
use failsafe::{Config as BreakerConfig, StateMachine, backoff as breaker_backoff, failure_policy};
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, RootCertStore};
use std::future::Future;
use std::io;
use std::sync::Arc;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::task::{JoinError, JoinHandle};
use tokio::time::timeout;
use tokio_retry::Retry;
use tokio_retry::strategy::{ExponentialBackoff, jitter};
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use url::Url;

use crate::domain::config::AgentConfig;
use crate::domain::model::Shared;

pub fn run<F, Fut>(state: Arc<Shared>, session: F) -> JoinHandle<io::Result<()>>
where
    F: Fn(
            Arc<Shared>,
            Arc<ReadHalf<TlsStream<TcpStream>>>,
            Arc<Mutex<WriteHalf<TlsStream<TcpStream>>>>,
        ) -> Fut
        + Send
        + 'static,
    Fut: Future<Output = Result<(), JoinError>> + Send + 'static,
{
    tokio::spawn(connection_loop(state, session))
}

async fn connection_loop<F, Fut>(state: Arc<Shared>, session: F) -> io::Result<()>
where
    F: Fn(
            Arc<Shared>,
            Arc<ReadHalf<TlsStream<TcpStream>>>,
            Arc<Mutex<WriteHalf<TlsStream<TcpStream>>>>,
        ) -> Fut
        + Send
        + 'static,
    Fut: Future<Output = Result<(), JoinError>>,
{
    let config: &AgentConfig = &state.config;
    let connector: TlsConnector = tls_connector(&state)?;
    let connector: &TlsConnector = &connector;

    let breaker: StateMachine<ConsecutiveFailures<Constant>, ()> = BreakerConfig::new()
        .failure_policy(failure_policy::consecutive_failures(
            config.breaker_failure_threshold,
            breaker_backoff::constant(config.breaker_open_for()),
        ))
        .build();
    
    let breaker: &StateMachine<ConsecutiveFailures<Constant>, ()> = &breaker;
    let multiplier: u64 = config.backoff_multiplier.max(1);

    loop {
        let strategy = ExponentialBackoff::from_millis(multiplier)
            .factor((config.backoff_min().as_millis() as u64 / multiplier).max(1))
            .max_delay(config.backoff_max())
            .map(jitter)
            .take(config.connect_max_retries.map_or(usize::MAX, |retries| retries as usize));

        let stream: TlsStream<TcpStream> = Retry::start(strategy, || async move {
            breaker
                .call(connect(config, connector))
                .await
                .map_err(|err| {
                    tracing::warn!("connect failed: {err}");
                    match err {
                        failsafe::Error::Inner(err) => err,
                        failsafe::Error::Rejected => io::Error::other("circuit breaker is open"),
                    }
                })
        })
        .await?;

        let (read_stream, write_stream): (
            ReadHalf<TlsStream<TcpStream>>,
            WriteHalf<TlsStream<TcpStream>>,
        ) = tokio::io::split(stream);

        let write_stream = Arc::new(Mutex::new(write_stream));
        let read_stream = Arc::new(read_stream);

        tracing::info!("connected to {}", config.receiver_url);
        if let Err(err) = session(state.clone(), read_stream, write_stream).await {
            tracing::warn!("session ended unexpectedly: {err}");
        }
    }
}

fn tls_connector(state: &Shared) -> io::Result<TlsConnector> {
    let mut roots: RootCertStore = RootCertStore::empty();
    for cert in rustls_pemfile::certs(&mut state.certificate.content.as_slice()) {
        roots
            .add(cert?)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    }

    let tls_config: ClientConfig = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();

    Ok(TlsConnector::from(Arc::new(tls_config)))
}

async fn connect(
    config: &AgentConfig,
    connector: &TlsConnector,
) -> io::Result<TlsStream<TcpStream>> {
    let url: &Url = &config.receiver_url;
    let host: &str = url
        .host_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "receiver_url has no host"))?;
    let port: u16 = url
        .port()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "receiver_url has no port"))?;

    let tcp: TcpStream = timeout(config.connect_timeout(), TcpStream::connect((host, port)))
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "tcp connect timed out"))??;

    tcp.set_nodelay(true)?;

    let server_name: ServerName<'static> = ServerName::try_from(host.to_owned())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    timeout(
        config.handshake_timeout(),
        connector.connect(server_name, tcp),
    )
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "tls handshake timed out"))?
}
