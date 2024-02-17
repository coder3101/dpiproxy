use std::{net::SocketAddr, sync::Arc};
use tracing::{instrument, Instrument};

use anyhow::{anyhow, Result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpSocket, TcpStream},
};

use crate::{resolver::resolve_host, Args};

fn parse_host_from_data(first_data: &str) -> Result<String> {
    Ok(first_data
        .lines()
        .nth(1)
        .ok_or_else(|| anyhow!("Failed to parse CONNECT message"))?
        .split_once(':')
        .ok_or_else(|| anyhow!("Does not contains Host header"))?
        .1
        .trim()
        .to_owned())
}

#[instrument(skip(first_data, cstream, args), name = "http-handler")]
pub(super) async fn handle_connection(
    first_data: &str,
    cstream: &mut TcpStream,
    args: Arc<Args>,
) -> anyhow::Result<()> {
    let host = parse_host_from_data(first_data)?;
    let ip = resolve_host(&host, args.dns, args.prefer_ipv6)
        .await?
        .iter()
        .next()
        .ok_or_else(|| anyhow!("DNS resolution produced no results"))?;

    tracing::info!("DNS resolution for {host} resolved to {ip:?}",);

    let socket = if ip.is_ipv4() {
        TcpSocket::new_v4()
    } else {
        TcpSocket::new_v6()
    }?;

    match socket
        .connect(SocketAddr::new(ip, 80))
        .instrument(tracing::info_span!("connect"))
        .await
    {
        Ok(mut sstream) => {
            let data = first_data.bytes().collect::<Vec<u8>>();
            let chunks = data.chunks(args.tls_segment_size).collect::<Vec<_>>();

            for chunk in chunks {
                sstream.write_all(chunk).await?;
            }

            let mut buff = [0u8; 1024];

            let iospan = tracing::info_span!("io span");
            let _guard = iospan.enter();

            let mut total_bytes_read = 0;
            let mut total_bytes_written = 0;
            loop {
                let bytes_read = sstream.read(&mut buff).in_current_span().await?;
                if bytes_read == 0 {
                    break;
                }

                total_bytes_read += bytes_read;

                let bytes_wrote = cstream.write(&buff[..bytes_read]).in_current_span().await?;
                if bytes_wrote == 0 {
                    break;
                }
                total_bytes_written += bytes_wrote
            }
            tracing::info!("{total_bytes_read} bytes read from server");
            tracing::info!("{total_bytes_written} bytes wrote to client");
            Ok(())
        }
        Err(e) => {
            tracing::warn!("Proxy server cannot establish connection to {ip}:80 for host {host}");
            Err(e.into())
        }
    }
}
