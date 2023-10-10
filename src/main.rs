mod api;
mod dns;
mod ip;

use anyhow::Result;
use api::Cli;
use clap::Parser;
use ip::DynDns;
use std::sync::Arc;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<()> {
	let cli = Cli::parse();
	let (mut set_v4, mut set_v6) = (cli.ipv4, cli.ipv6);

	pretty_env_logger::formatted_builder()
		.filter_level(cli.verbose.log_level_filter())
		.init();

	let (public_ipv4, public_ipv6) = ip::get_ips(cli.local).await?;
	let api_client = Arc::new(api::get_client(&cli)?);
	let records = dns::get_records(&cli, api_client.clone()).await?;
	let mut handles: Vec<JoinHandle<Result<()>>> =
		Vec::with_capacity(cli.records.len());

	if (false, false) == (set_v4, set_v6) {
		(set_v4, set_v6) = (true, true);
	}

	for (name, id, a, aaaa) in records {
		if let Some(id) = id {
			if set_v4 {
				if let Some(handle) = public_ipv4.update(
					api_client.clone(),
					a,
					name.clone(),
					id.clone(),
				) {
					handles.push(handle);
				}
			}
			if set_v6 {
				if let Some(handle) = public_ipv6.update(
					api_client.clone(),
					aaaa,
					name.clone(),
					id.clone(),
				) {
					handles.push(handle);
				}
			}
		} else {
			log::warn!("skipping `{}`; no valid DNS zone", name)
		}
	}

	let mut results = vec![];
	for handle in handles {
		results.push(handle.await)
	}
	for result in results {
		result??
	}

	Ok(())
}
