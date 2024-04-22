use std::{net::SocketAddr, sync::Arc};

use anyhow::anyhow;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpSocket, TcpStream},
};

use crate::{cli::Args, resolver::resolve_host};

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

pub(super) async fn handle_connection(
    first_data: &str,
    cstream: &mut TcpStream,
    args: Arc<Args>,
) -> anyhow::Result<TcpStream> {
    let (host, port) = parse_connect_request(first_data)?;
    tracing::debug!("Requesting to connect to {host} on {port}");
    let result = resolve_host(host, args.dns, args.prefer_ipv6)
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
    tracing::info!("DNS resolution for {host} resolved to {sock_addr:?}",);

    match server.connect(sock_addr).await {
        Err(e) => {
            tracing::warn!(
                "Proxy server cannot establish connection to {sock_addr} for {host}. Error {e}"
            );
            Err(e.into())
        }
        Ok(mut sstream) => {
            tracing::debug!("Connection to remote {host} is success");
            cstream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await?;

            let iospan = tracing::info_span!("io span");
            let _guard = iospan.enter();

            // Capture TLS handshake
            let mut buff = [0; 1028];
            let bytes_read = cstream.read(&mut buff).await?;
            if bytes_read == 0 {
                return Err(anyhow!("Client closed before CLIENT HELLO"));
            }

            let data = &buff[..bytes_read];
            let chunks = data.chunks(args.tls_segment_size).collect::<Vec<_>>();

            for chunk in chunks {
                sstream.write_all(chunk).await?;
            }
            Ok(sstream)
        }
    }
}
