# cfdyndns

`CloudFlare Dynamic DNS Client`

Reimplementation of [cloudflare-dyndns](https://github.com/colemickens/cloudflare-dyndns) in [Rust](https://www.rust-lang.org).

Builds with `rust 1.3.0 stable`, possibly `rust 1.2.0` as well.

## building

1. install `make` and `rust 1.3.0`.

2. clone this repo

3. (debug build) `make build`

4. (release build) `make build-release`

## running

1. (debug build) `make run`

2. (release build) `make run-release`

## installing as systemd service

1. edit `systemd/cloudflare-dyndns.service` to point to your `cloudflare-dyndns` binary.

2. copy `systemd/cloudflare-dyndns.config.example` to `systemd/cloudflare-dyndns.config` and update as appropriate

2. `make install-systemd`

## uninstalling systemd service

1. `make uninstall-systemd`

## example systemd journalctl log

```
Sep 20 15:36:40 chimera systemd[1]: Started Cloudflare-dyndns.
Sep 20 15:36:43 chimera cloudflare-dyndns[22760]: *.mickens.tv (74.125.186.11 -> 66.235.2.123)... done
Sep 20 15:36:44 chimera cloudflare-dyndns[22760]: mickens.tv (74.125.186.11 -> 66.235.2.123)... done
Sep 20 15:36:44 chimera cloudflare-dyndns[22760]: *.mickens.xxx (74.125.186.11 -> 66.235.2.123)... done
Sep 20 15:36:46 chimera cloudflare-dyndns[22760]: mickens.xxx (74.125.186.11 -> 66.235.2.123)... done
Sep 20 15:36:46 chimera cloudflare-dyndns[22760]: cole.mickens.us (74.125.186.11 -> 66.235.2.123)... done
Sep 20 15:36:46 chimera cloudflare-dyndns[22760]: *.mickens.us (74.125.186.11 -> 66.235.2.123)... done
Sep 20 15:36:47 chimera cloudflare-dyndns[22760]: mickens.us (74.125.186.11 -> 66.235.2.123)... done
Sep 20 15:36:47 chimera cloudflare-dyndns[22760]: *.mickens.me (74.125.186.11 -> 66.235.2.123)... done
Sep 20 15:36:48 chimera cloudflare-dyndns[22760]: mickens.me (74.125.186.11 -> 66.235.2.123)... done
Sep 20 15:36:48 chimera cloudflare-dyndns[22760]: recessionomics.us (74.125.186.11 -> 66.235.2.123)... done
Sep 20 15:36:49 chimera cloudflare-dyndns[22760]: www.recessionomics.us (74.125.186.11 -> 66.235.2.123)... done
Sep 20 15:36:49 chimera cloudflare-dyndns[22760]: *.mickens.io (74.125.186.11 -> 66.235.2.123)... done
Sep 20 15:36:49 chimera cloudflare-dyndns[22760]: mickens.io (74.125.186.11 -> 66.235.2.123)... done
Sep 20 15:41:45 chimera systemd[1]: Started Cloudflare-dyndns.
Sep 20 15:41:45 chimera cloudflare-dyndns[23288]: recessionomics.us skipped, up to date
Sep 20 15:41:45 chimera cloudflare-dyndns[23288]: www.recessionomics.us skipped, up to date
Sep 20 15:41:45 chimera cloudflare-dyndns[23288]: *.mickens.tv skipped, up to date
Sep 20 15:41:45 chimera cloudflare-dyndns[23288]: mickens.tv skipped, up to date
Sep 20 15:41:45 chimera cloudflare-dyndns[23288]: *.mickens.io skipped, up to date
Sep 20 15:41:45 chimera cloudflare-dyndns[23288]: mickens.io skipped, up to date
Sep 20 15:41:46 chimera cloudflare-dyndns[23288]: cole.mickens.us skipped, up to date
Sep 20 15:41:46 chimera cloudflare-dyndns[23288]: *.mickens.us skipped, up to date
Sep 20 15:41:46 chimera cloudflare-dyndns[23288]: mickens.us skipped, up to date
Sep 20 15:41:46 chimera cloudflare-dyndns[23288]: *.mickens.xxx skipped, up to date
Sep 20 15:41:46 chimera cloudflare-dyndns[23288]: mickens.xxx skipped, up to date
Sep 20 15:41:46 chimera cloudflare-dyndns[23288]: *.mickens.me skipped, up to date
Sep 20 15:41:46 chimera cloudflare-dyndns[23288]: mickens.me skipped, up to date
```
