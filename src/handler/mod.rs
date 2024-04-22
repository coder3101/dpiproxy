use std::sync::Arc;

use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::cli::Args;

mod http;
mod https;

pub async fn handle_connection(mut cstream: TcpStream, args: Arc<Args>) -> anyhow::Result<()> {
    let mut buff = [0; 1028];
    let bytes_read = cstream.read(&mut buff).await?;
    if bytes_read == 0 {
        tracing::info!("Connection closed by client");
        return Ok(());
    }
    let data = &buff[..bytes_read];
    let first_data = String::from_utf8_lossy(data);

    if data.starts_with(b"CONNECT") {
        tracing::info!("handling with https handler");
        match https::handle_connection(&first_data, &mut cstream, args).await {
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
    } else if let Err(e) = http::handle_connection(&first_data, &mut cstream, args).await {
        return Err(e);
    }
    Ok(())
}
