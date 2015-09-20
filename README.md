# cloudflare-dyndns-rust

Reimplementation of [cloudflare-dyndns](https://github.com/colemickens/cloudflare-dyndns) in [Rust](https://www.rust-lang.org).

Builds with `rust 1.2.0 stable`.

## building

1. install `make` and `rust 1.2.0`.

2. clone this repo

3. (debug build) `make build`

4. (release build) `make build-release`

## running

1. (debug build) `make run`

2. (debug build) `make run-release`

## installing as systemd service

1. edit systemd/cloudflare-dyndns.service to point to your `cloudflare-dyndns` binary.

2. `make install-systemd`

## uninstalling systemd service

1. `make uninstall-systemd`

## todo

1. pin dependency versions
