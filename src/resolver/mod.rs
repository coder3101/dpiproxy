use hickory_resolver::config::{LookupIpStrategy, ResolverConfig, ResolverOpts};
use hickory_resolver::lookup_ip::LookupIp;
use hickory_resolver::TokioAsyncResolver;
use tracing::instrument;

use self::provider::DnsProviders;

pub mod provider;

#[instrument]
pub async fn resolve_host(
    host: &str,
    provider: DnsProviders,
    prefer_ipv6: bool,
) -> anyhow::Result<LookupIp> {
    // Match the provided provider and generate appropriate resolver
    //
    let mut opts = ResolverOpts::default();

    opts.ip_strategy = if prefer_ipv6 {
        LookupIpStrategy::Ipv6thenIpv4
    } else {
        LookupIpStrategy::Ipv4thenIpv6
    };

    match provider {
        DnsProviders::System => {
            let resolver =
                TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
            Ok(resolver.lookup_ip(host).await?)
        }
        DnsProviders::GoogleDnsOverTLS => {
            let resolver =
                TokioAsyncResolver::tokio(ResolverConfig::google_tls(), ResolverOpts::default());
            Ok(resolver.lookup_ip(host).await?)
        }
        DnsProviders::GoogleDnsOverHTTPS => {
            let resolver =
                TokioAsyncResolver::tokio(ResolverConfig::google_https(), ResolverOpts::default());
            Ok(resolver.lookup_ip(host).await?)
        }
        DnsProviders::CloudflareDnsOverTLS => {
            let resolver = TokioAsyncResolver::tokio(
                ResolverConfig::cloudflare_tls(),
                ResolverOpts::default(),
            );
            Ok(resolver.lookup_ip(host).await?)
        }
        DnsProviders::CloudflareDnsOverHTTPS => {
            let resolver = TokioAsyncResolver::tokio(
                ResolverConfig::cloudflare_https(),
                ResolverOpts::default(),
            );
            Ok(resolver.lookup_ip(host).await?)
        }
        DnsProviders::Quad9DnsOverTLS => {
            let resolver =
                TokioAsyncResolver::tokio(ResolverConfig::quad9_tls(), ResolverOpts::default());
            Ok(resolver.lookup_ip(host).await?)
        }
        DnsProviders::Quad9DnsOverHTTPS => {
            let resolver =
                TokioAsyncResolver::tokio(ResolverConfig::quad9_https(), ResolverOpts::default());
            Ok(resolver.lookup_ip(host).await?)
        }
    }
}
