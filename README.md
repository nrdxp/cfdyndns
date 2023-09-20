# cfdyndns

`CloudFlare Dynamic DNS Client`

Reimplementation of [cloudflare-dyndns](https://github.com/colemickens/cloudflare-dyndns) in [Rust](https://www.rust-lang.org).

## building

`cargo build`

## usage

```console
Usage: cfdyndns [OPTIONS] --records <RECORDS>

Options:
  -r, --records <RECORDS>  Comma separated DNS records to update with the host's public IP [env: CLOUDFLARE_RECORDS=]
  -t, --token <TOKEN>      recommended: The CloudFlare API token to authenticate with [env: CLOUDFLARE_APITOKEN]
  -k, --key <KEY>          deprecated: The CloudFlare API key to authenticate with, also requires email [env: CLOUDFLARE_APIKEY]
  -e, --email <EMAIL>      deprecated: The CloudFlare email to authenticate with, also requires API key [env: CLOUDFLARE_EMAIL=]
  -v, --verbose...         More output per occurrence
  -q, --quiet...           Less output per occurrence
  -6                       set an AAAA record to the host's ipv6 address
  -4                       set an A record to the host's ipv4 address
  -h, --help               Print help
  -V, --version            Print version
```

### installing as systemd service

1. edit `systemd/cloudflare-dyndns.service` to point to your `cloudflare-dyndns` binary.

2. copy `systemd/cloudflare-dyndns.config.example` to `systemd/cloudflare-dyndns.config` and update as appropriate

2. `make install-systemd`

### uninstalling systemd service

1. `make uninstall-systemd`

### acknowledgement

Special thanks to [colemickens](https://github.com/colemickens) for bootstrapping and transferring ownership of this project. 
