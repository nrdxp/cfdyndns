mod api;
mod clone;
mod fns;

use cloudflare::endpoints::dns::{
	CreateDnsRecord, CreateDnsRecordParams, DeleteDnsRecord, DnsContent,
};

use cloudflare::framework::async_api::ApiClient;
use std::net::IpAddr;

use anyhow::Result;

use api::Cli;
use clap::Parser;

use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
	let cli = Cli::parse();

	pretty_env_logger::formatted_builder()
		.filter_level(cli.verbose.log_level_filter())
		.init();

	let (public_ipv4, public_ipv6) = fns::get_ips().await?;
	let api_client = Arc::new(fns::get_client(&cli)?);
	let records = fns::get_records(&cli, api_client.clone()).await?;
	let mut handles = Vec::with_capacity(cli.records.len());

	for (record, zone, dns_v4, dns_v6) in records {
		if let Some(zone) = zone {
			if let Some(ip) = public_ipv4 {
				let client = api_client.clone();
				if let Some(dns) = dns_v4 {
					match dns.content {
						DnsContent::A { content: ipv4 } => {
							handles.push(tokio::spawn(async move {
								fns::update_record(
									dns,
									IpAddr::V4(ipv4),
									ip,
									client,
								)
								.await
							}));
						}
						_ => continue,
					}
				} else {
					log::info!("{} → {}\n", record, ip);
					let name = record.clone();
					let id = zone.clone();
					handles.push(tokio::spawn(async move {
						client
							.request(&CreateDnsRecord {
								zone_identifier: &id,
								params: CreateDnsRecordParams {
									ttl: Some(1),
									priority: None,
									proxied: Some(false),
									name: &name,
									content: DnsContent::A {
										content: match ip {
											IpAddr::V4(ip) => ip,
											_ => unreachable!(),
										},
									},
								},
							})
							.await?;
						Ok(())
					}));
				}
			} else if let Some(dns) = dns_v4 {
				log::info!("deleting A record: {}\n", record);
				let client = api_client.clone();
				let id = zone.clone();
				handles.push(tokio::spawn(async move {
					client
						.request(&DeleteDnsRecord {
							zone_identifier: &id,
							identifier: &dns.id,
						})
						.await?;
					Ok(())
				}))
			}
			if let Some(ip) = public_ipv6 {
				let client = api_client.clone();
				if let Some(dns) = dns_v6 {
					match dns.content {
						DnsContent::AAAA { content: ipv6 } => {
							handles.push(tokio::spawn(async move {
								fns::update_record(
									dns,
									IpAddr::V6(ipv6),
									ip,
									client,
								)
								.await
							}));
						}
						_ => continue,
					}
				} else {
					log::info!("{} → {}\n", record, ip);
					handles.push(tokio::spawn(async move {
						client
							.request(&CreateDnsRecord {
								zone_identifier: &zone,
								params: CreateDnsRecordParams {
									ttl: Some(1),
									priority: None,
									proxied: Some(false),
									name: &record,
									content: DnsContent::AAAA {
										content: match ip {
											IpAddr::V6(ip) => ip,
											_ => unreachable!(),
										},
									},
								},
							})
							.await?;
						Ok(())
					}));
				}
			} else if let Some(dns) = dns_v6 {
				log::info!("deleting AAAA record: {}\n", record);
				let client = api_client.clone();
				handles.push(tokio::spawn(async move {
					client
						.request(&DeleteDnsRecord {
							zone_identifier: &zone,
							identifier: &dns.id,
						})
						.await?;
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
