APIKEY:=`cat apikey.md`

run:
	CLOUDFLARE_APIKEY=$(APIKEY) CLOUDFLARE_EMAIL=cole.mickens@gmail.com cargo run --verbose