# cfdyndns

`CloudFlare Dynamic DNS Client`

Reimplementation of [cloudflare-dyndns](https://github.com/colemickens/cloudflare-dyndns) in [Rust](https://www.rust-lang.org).

## building

`cargo build`

## usage

```shell
Usage: cfdyndns [OPTIONS]

Options:
  -r, --records <RECORDS>  Comma separated DNS records to update with the host's public IP [env:
                           CLOUDFLARE_RECORDS=panzy.nrd.sh,panzy.nrd.sh,nrd.xp]
  -t, --token <TOKEN>      recommended: The CloudFlare API token to authenticate with deprecated: The CloudFlare
                           API key to authenticate with, also requires email [env: CLOUDFLARE_APITOKEN]
  -k, --key <KEY>          deprecated: The CloudFlare email to authenticate with, also requires API key [env:
                           CLOUDFLARE_APIKEY]
  -e, --email <EMAIL>      [env: CLOUDFLARE_EMAIL=]
  -v, --verbose...         More output per occurrence
  -q, --quiet...           Less output per occurrence
  -h, --help               Print help
  -V, --version            Print version
```

### installing as systemd service

1. edit `systemd/cloudflare-dyndns.service` to point to your `cloudflare-dyndns` binary.

2. copy `systemd/cloudflare-dyndns.config.example` to `systemd/cloudflare-dyndns.config` and update as appropriate

2. `make install-systemd`

### uninstalling systemd service

1. `make uninstall-systemd`
