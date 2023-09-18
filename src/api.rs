use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Cli {
	/// Comma separated DNS records to update with the host's public IP
	#[clap(
		long,
		short,
		env = "CLOUDFLARE_RECORDS",
		value_name = "RECORDS",
		value_delimiter(',')
	)]
	pub records: Vec<String>,
	/// recommended: The CloudFlare API token to authenticate with
	#[clap(
		long,
		short,
		env = "CLOUDFLARE_APITOKEN",
		hide_env_values = true,
		value_name = "TOKEN",
		required_unless_present_all(["key", "email"])
	)]
	/// deprecated: The CloudFlare API key to authenticate with, also requires email
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
	/// deprecated: The CloudFlare email to authenticate with, also requires API key
	pub key: Option<String>,
	#[clap(
		long,
		short,
		env = "CLOUDFLARE_EMAIL",
		value_name = "EMAIL",
		required_unless_present("token"),
		requires("key")
	)]
	pub email: Option<String>,

	#[clap(flatten)]
	pub verbose: Verbosity<InfoLevel>,
}
