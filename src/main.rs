use crate::handler::handle_connection;
use std::sync::Arc;

use clap::Parser;
use cli::Args;
use tokio::net::TcpSocket;

mod cli;
mod handler;
mod resolver;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Arc::new(Args::parse());
    let socket = TcpSocket::new_v4()?;
    if args.reuse_address {
        socket.set_reuseaddr(true)?;
    }
    let addr = format!("{}:{}", args.host, args.port);
    socket.bind(addr.parse().unwrap())?;

    tracing::info!("Proxy is listening for connection on {addr}");
    tracing::info!("Proxy is using {:?} DNS", args.dns);

    let listener = socket.listen(128)?;

    loop {
        let args2 = args.clone();
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, args2).await {
                // All critical errors are already propagated in handler
                tracing::debug!("Error handling connection {e}");
            }
        });
    }
}
