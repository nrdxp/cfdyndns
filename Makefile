APIKEY:=`cat apikey.md`
EXPORTSTUFF:=`cut -d= -f1 /etc/cloudflare-dyndns/cloudflare-dyndns.config`

run:
	source /etc/cloudflare-dyndns/cloudflare-dyndns.config; \
	export CLOUDFLARE_APIKEY; \
	export CLOUDFLARE_EMAIL; \
	export CLOUDFLARE_RECORDS; \
	cargo run --verbose