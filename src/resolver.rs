use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::lookup_ip::LookupIp;
use hickory_resolver::TokioAsyncResolver;

pub async fn resolve_host(host: &str) -> anyhow::Result<LookupIp> {
    let resolver =
        TokioAsyncResolver::tokio(ResolverConfig::cloudflare_tls(), ResolverOpts::default());
    Ok(resolver.lookup_ip(host).await?)
}
