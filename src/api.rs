use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
	#[clap(
		long,
		short,
		env = "CLOUDFLARE_RECORDS",
		value_name = "RECORDS",
		value_delimiter(',')
	)]
	/// Comma separated DNS records to update with the host's public IP
	pub records: Vec<String>,
	#[clap(
		long,
		short,
		env = "CLOUDFLARE_APITOKEN",
		hide_env_values = true,
		value_name = "TOKEN",
		required_unless_present_all(["key", "email"])
	)]
	/// recommended: The CloudFlare API token to authenticate with
	pub token: Option<String>,
	#[clap(
		long,
		short,
		env = "CLOUDFLARE_APIKEY",
		hide_env_values = true,
		value_name = "KEY",
		required_unless_present("token"),
		requires("email")
	)]
	/// deprecated: The CloudFlare API key to authenticate with, also requires email
	pub key: Option<String>,
	#[clap(
		long,
		short,
		env = "CLOUDFLARE_EMAIL",
		value_name = "EMAIL",
		required_unless_present("token"),
		requires("key")
	)]
	/// deprecated: The CloudFlare email to authenticate with, also requires API key
	pub email: Option<String>,
}
