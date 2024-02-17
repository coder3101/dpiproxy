use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DnsProviders {
    System,
    GoogleDnsOverTLS,
    GoogleDnsOverHTTPS,
    CloudflareDnsOverTLS,
    CloudflareDnsOverHTTPS,
    Quad9DnsOverTLS,
    Quad9DnsOverHTTPS,
}
