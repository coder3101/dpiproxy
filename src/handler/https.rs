use std::net::SocketAddr;

use anyhow::anyhow;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpSocket, TcpStream},
};

use crate::resolver::resolve_host;

fn parse_connect_request(first_data: &str) -> anyhow::Result<(&str, &str)> {
    let params = first_data
        .lines()
        .next()
        .map(|l| l.split_whitespace().collect::<Vec<_>>())
        .ok_or_else(|| anyhow!("Failed to parse CONNECT message"))?;

    params
        .get(1)
        .ok_or_else(|| anyhow!("CONNECT does not contain host/port"))?
        .split_once(':')
        .ok_or_else(|| anyhow!("CONNECT host does not contain ':'"))
}

pub async fn handle_https_connection(
    first_data: &str,
    cstream: &mut TcpStream,
) -> anyhow::Result<TcpStream> {
    let (host, port) = parse_connect_request(first_data)?;
    tracing::debug!(
        handler = "https",
        "Requesting to connect to {host} on {port}"
    );
    let result = resolve_host(host)
        .await?
        .iter()
        .next()
        .ok_or_else(|| anyhow!("DNS resolution produced no results"))?;

    let server = if result.is_ipv4() {
        TcpSocket::new_v4()
    } else {
        TcpSocket::new_v6()
    }?;

    let sock_addr = SocketAddr::new(result, port.parse().unwrap_or(443));
    tracing::info!(
        handler = "https",
        "DNS resolution for {host} resolved to {sock_addr:?}",
    );

    match server.connect(sock_addr).await {
        Err(e) => {
            tracing::warn!(
                handler = "https",
                "Proxy server cannot establish connection to {sock_addr} for {host}. Error {e}"
            );
            Err(e.into())
        }
        Ok(mut sstream) => {
            tracing::debug!(handler = "https", "Connection to remote {host} is success");
            cstream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await?;

            // Capture TLS handshake
            let mut buff = [0; 1028];
            let bytes_read = cstream.read(&mut buff).await?;
            if bytes_read == 0 {
                return Err(anyhow!("Client closed before CLIENT HELLO"));
            }

            let data = &buff[..bytes_read];
            for chunk in data.chunks(6) {
                sstream.write_all(chunk).await?;
            }
            Ok(sstream)
        }
    }
}