# dpiproxy
Deep packet inspection is a technique used by ISPs across the globe to censor content on web, this is a proxy server which makes Deep packet inspection impossible for the ISPs by breaking the `CLIENT HELLO` packet to multiple small packets to prevent censorship.

## Enabling IPv6 support inside docker (Optional)

### Configurng docker
By default docker does not provides IPv6 support to containers so you need to enable some features in your docker installation.

Create a file `/etc/docker/daemon.json` with following contents:
```json
{
    "ipv6": true,
    "experimental": true,
    "ip6tables": true,
    "fixed-cidr-v6": "fd00::/80"
}
```

Restart docker with `systemctl restart docker` and you can run the container as usual

### Docker compose with IPv6
Use the docker compose-ipv6.yaml instead of default compose.yaml to use with IPv6. Use `docker compose -f compose-ipv6.yaml up -d --build .`
