use clap::Parser;

use crate::resolver::provider::DnsProviders;

/// dpiproxy is a proxy built to bypass censorship using Deep packet inspection
#[derive(Debug, Parser)]
#[clap(name = "dpiproxy", version)]
pub struct Args {
    /// Port for the Proxy Server
    #[arg(long, default_value_t = 8000)]
    pub port: u16,

    /// Interface or host for the Proxy Server
    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,

    /// Allow proxy server socket to bind in use address
    #[arg(long, default_value_t = false)]
    pub reuse_address: bool,

    /// Prefer IPv6 over IPv4 for DNS resolution
    #[arg(long, default_value_t = false)]
    pub prefer_ipv6: bool,

    /// DNS to use for name resolution
    #[arg(long, value_enum, default_value_t = DnsProviders::System)]
    pub dns: DnsProviders,

    /// TLS CLIENT HELLO segmentation size
    #[arg(long, default_value_t = 6)]
    pub tls_segment_size: usize,

    /// TLS CLIENT HELLO shuffling
    #[arg(long, default_value_t = false)]
    pub tls_segment_shuffle: bool,
}
