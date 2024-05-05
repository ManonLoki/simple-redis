use anyhow::Result;
use simple_redis::{network, Backend};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr = "0.0.0.0:6379";
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Simple-Redis-Server listening on: {}", addr);

    let backend = Backend::new();

    loop {
        let cloned_backend = backend.clone();
        let (stream, remote_addr) = listener.accept().await?;
        tracing::info!("Accepted connection from: {}", remote_addr);
        tokio::spawn(async move {
            match network::stream_handler(stream, cloned_backend).await {
                Ok(_) => {
                    tracing::info!("Connection from {} exited", remote_addr);
                }
                Err(e) => {
                    tracing::warn!("handle error for {}: {:?}", remote_addr, e);
                }
            }
        });
    }
}
