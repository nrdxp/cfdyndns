mod api;
mod dns;
mod ip;

use anyhow::Result;
use api::Cli;
use clap::Parser;
use cloudflare::endpoints::dns::{DnsContent, DnsRecord};
use cloudflare::framework::async_api::ApiClient;
use dns::Requests;
use std::sync::Arc;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<()> {
	let cli = Cli::parse();

	pretty_env_logger::formatted_builder()
		.filter_level(cli.verbose.log_level_filter())
		.init();

	let (public_ipv4, public_ipv6) = ip::get_ips().await?;
	let api_client = Arc::new(api::get_client(&cli)?);
	let records = dns::get_records(&cli, api_client.clone()).await?;
	let mut handles: Vec<JoinHandle<Result<()>>> =
		Vec::with_capacity(cli.records.len());

	for (name, id, a, aaaa) in records {
		if let Some(id) = id {
			if let Some(ip) = public_ipv4 {
				let client = api_client.clone();
				if let Some(a) = a {
					match a.content {
						DnsContent::A { content: _ } => {
							handles.push(tokio::spawn(async move {
								if let Some(req) = a.update_request(ip) {
									client.request(&req).await?;
								}
								Ok(())
							}));
						}
						_ => continue,
					}
				} else {
					let name = name.clone();
					let id = id.clone();
					handles.push(tokio::spawn(async move {
						client
							.request(&DnsRecord::create_request(ip, &name, &id))
							.await?;
						Ok(())
					}));
				}
			} else if let Some(a) = a {
				let client = api_client.clone();
				handles.push(tokio::spawn(async move {
					client.request(&a.delete_request()).await?;
					Ok(())
				}))
			}
			if let Some(ip) = public_ipv6 {
				let client = api_client.clone();
				if let Some(aaaa) = aaaa {
					match aaaa.content {
						DnsContent::AAAA { content: _ } => {
							handles.push(tokio::spawn(async move {
								if let Some(req) = aaaa.update_request(ip) {
									client.request(&req).await?;
								}
								Ok(())
							}));
						}
						_ => continue,
					}
				} else {
					handles.push(tokio::spawn(async move {
						client
							.request(&DnsRecord::create_request(ip, &name, &id))
							.await?;
						Ok(())
					}));
				}
			} else if let Some(aaaa) = aaaa {
				let client = api_client.clone();
				handles.push(tokio::spawn(async move {
					client.request(&aaaa.delete_request()).await?;
					Ok(())
				}))
			}
		}
	}

	// await all results before handling errors
	let mut results = vec![];
	for handle in handles {
		results.push(handle.await)
	}
	for result in results {
		result??
	}

	Ok(())
}
