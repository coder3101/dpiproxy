use std::net::IpAddr;

use clap::{Parser, ValueEnum};

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

    /// DNS server IP to use incase of custom DNS resolution
    #[arg(long)]
    pub dns_ip: Option<IpAddr>,

    /// DNS server Port to use incase of custom DNS resolution
    #[arg(long)]
    pub dns_port: Option<u16>,

    /// DNS to use for name resolution
    #[arg(long, value_enum, default_value_t = DnsProviders::System)]
    pub dns: DnsProviders,
}

#[derive(Debug, Clone, ValueEnum, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DnsProviders {
    System,
    CustomInsecure,
    GoogleDnsOverTLS,
    GoogleDnsOverHTTPS,
    CloudflareDnsOverTLS,
    CloudflareDnsOverHTTPS,
    Quad9DnsOverTLS,
    Quad9DnsOverHTTPS,
}
