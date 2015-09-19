APIKEY:=`cat apikey.md`

build:
	cargo build

build-release:
	cargo build --release

run: build
	source /etc/cloudflare-dyndns/cloudflare-dyndns.config; \
	export CLOUDFLARE_APIKEY; \
	export CLOUDFLARE_EMAIL; \
	export CLOUDFLARE_RECORDS; \
	cargo run

run-release:
	source /etc/cloudflare-dyndns/cloudflare-dyndns.config; \
	export CLOUDFLARE_APIKEY; \
	export CLOUDFLARE_EMAIL; \
	export CLOUDFLARE_RECORDS; \
	cargo run --release

install-systemd: build-release
	sudo mkdir -p /etc/cloudflare-dyndns
	sudo cp systemd/cloudflare-dyndns.config /etc/cloudflare-dyndns/
	sudo cp systemd/cloudflare-dyndns.service /etc/systemd/system/
	sudo cp systemd/cloudflare-dyndns.timer /etc/systemd/system/
	sudo systemctl enable cloudflare-dyndns.service
	sudo systemctl enable cloudflare-dyndns.timer
	sudo systemctl start cloudflare-dyndns.service
	sudo systemctl start cloudflare-dyndns.timer

uninstall-systemd:
	sudo systemctl stop cloudflare-dyndns.service
	sudo systemctl stop cloudflare-dyndns.timer
	sudo systemctl disable cloudflare-dyndns.service
	sudo systemctl disable cloudflare-dyndns.timer
	sudo rm /etc/systemd/system/cloudflare-dyndns.service
	sudo rm /etc/systemd/system/cloudflare-dyndns.timer
	sudo rm -r /etc/cloudflare-dyndns
