use std::net::TcpListener as StdTcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use fleet_crypto::generate_certs;
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, RootCertStore};
use tokio::net::TcpStream;
use tokio::time::sleep;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;

use crate::domain::config::ServerConfig;

struct Setup {
    config: Arc<ServerConfig>,
    connector: TlsConnector,
    dir: PathBuf,
}

impl Drop for Setup {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn setup(name: &str, max_connections: usize, handshake_timeout: Duration) -> Setup {
    let dir = std::env::temp_dir().join(format!("fleet-receiver-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let certs = generate_certs(&["127.0.0.1".to_owned()]).unwrap();
    std::fs::write(dir.join("cert.pem"), &certs.cert_pem.content).unwrap();
    std::fs::write(dir.join("key.pem"), &certs.key_pem.content).unwrap();

    // a free port: bind to 0, read it, release it
    let port = StdTcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();

    let mut roots = RootCertStore::empty();
    for cert in rustls_pemfile::certs(&mut certs.cert_pem.content.as_slice()) {
        roots.add(cert.unwrap()).unwrap();
    }
    let connector = TlsConnector::from(Arc::new(
        ClientConfig::builder().with_root_certificates(roots).with_no_client_auth(),
    ));

    let config = ServerConfig {
        host: "127.0.0.1".to_owned(),
        port,
        max_connections,
        handshake_timeout,
        cert_path: dir.join("cert.pem"),
        key_path: dir.join("key.pem"),
    };
    Setup { config: Arc::new(config), connector, dir }
}

async fn connect(setup: &Setup) -> std::io::Result<TlsStream<TcpStream>> {
    let addr = format!("127.0.0.1:{}", setup.config.port);
    let mut last = None;
    for _ in 0..40 {
        match TcpStream::connect(&addr).await {
            Ok(tcp) => {
                return setup.connector.connect(ServerName::try_from("127.0.0.1").unwrap(), tcp).await;
            }
            Err(err) => {
                last = Some(err);
                sleep(Duration::from_millis(50)).await;
            }
        }
    }
    Err(last.unwrap())
}
