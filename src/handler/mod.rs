use std::sync::Arc;

use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::cli::Args;

use self::https::handle_https_connection;

mod http;
mod https;

pub async fn handle_connection(mut cstream: TcpStream, _: Arc<Args>) -> anyhow::Result<()> {
    let mut buff = [0; 1028];
    let bytes_read = cstream.read(&mut buff).await?;
    if bytes_read == 0 {
        tracing::info!("Connection closed by client");
        return Ok(());
    }
    let data = &buff[..bytes_read];

    if data.starts_with(b"CONNECT") {
        match handle_https_connection(String::from_utf8_lossy(data).as_ref(), &mut cstream).await {
            Ok(mut sstream) => {
                if let Err(e) = tokio::io::copy_bidirectional(&mut sstream, &mut cstream).await {
                    tracing::debug!("Bidrectional copy error {e}");
                    return Err(e.into());
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    } else {
        // HTTP only
        // tracing::warn!("Blocked request non-HTTP request");
    }
    Ok(())
}
