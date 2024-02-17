use std::sync::Arc;

use anyhow::{anyhow, Result};
use tokio::net::TcpStream;

use crate::{resolver::resolve_host, Args};

fn parse_host_from_data(first_data: &str) -> Result<String> {
    Ok(first_data.lines().nth(1).ok_or_else(|| anyhow!("Failed to parse CONNECT message"))?.split_once(':').ok_or_else(|| anyhow!("Does not contains Host header"))?.1.trim().to_owned())
}

pub(super) async fn handle_connection(
    first_data: &str,
    cstream: &mut TcpStream,
    args: Arc<Args>,
) -> anyhow::Result<TcpStream> {
    let host = parse_host_from_data(first_data)?;
    let ip = resolve_host(&host, args.dns, args.prefer_ipv6).await?
        .iter()
        .next()
        .ok_or_else(|| anyhow!("DNS resolution produced no results"))?;
    let sstream = if ip.is_ipv4() { TcpStream::new() } else {TcpStream::new_v6() }


    Ok(sstream)
}

