mod api;

use cloudflare::{
	endpoints::{
		dns::{
			DnsContent, DnsRecord, ListDnsRecords, ListDnsRecordsParams,
			UpdateDnsRecord, UpdateDnsRecordParams,
		},
		zone::{ListZones, ListZonesParams},
	},
	framework::{
		async_api::{ApiClient, Client},
		auth::Credentials,
		Environment, HttpApiClientConfig,
	},
};
use std::io;
use std::io::prelude::Write;

use std::net::IpAddr;

use public_ip::{http, Version};

use anyhow::Result;

use api::Cli;
use clap::Parser;

use std::sync::Arc;

async fn get_ips() -> Result<(Option<IpAddr>, Option<IpAddr>)> {
	let (ipv4, ipv6) = tokio::join!(
		public_ip::addr_with(http::ALL, Version::V4),
		public_ip::addr_with(public_ip::ALL, Version::V6)
	);

	if (None, None) == (ipv6, ipv4) {
		anyhow::bail!("Could not determine your current public IP address.")
	}

	if let Some(ip) = ipv4 {
		log::info!("{}", ip);
	}
	if let Some(ip) = ipv6 {
		log::info!("{}", ip);
	};
	Ok((ipv4, ipv6))
}

fn get_client(cli: &Cli) -> Result<Client> {
	let credentials: Credentials = if let Some(token) = cli.token.clone() {
		Credentials::UserAuthToken { token }
	} else if let (Some(key), Some(email)) =
		(cli.key.clone(), cli.email.clone())
	{
		log::warn!("API Key & Email combo is deprecated. Please switch to using an API token");
		Credentials::UserAuthKey { email, key }
	} else {
		unreachable!()
	};

	Client::new(
		credentials,
		HttpApiClientConfig::default(),
		Environment::Production,
	)
}

async fn get_records(client: Arc<Client>) -> Result<Vec<DnsRecord>> {
	let zones = client
		.clone()
		.request(&ListZones {
			params: ListZonesParams::default(),
		})
		.await?
		.result;
	let mut handles = Vec::with_capacity(zones.len());
	let mut records = Vec::with_capacity(zones.len() * 10);
	for zone in zones {
		let client = client.clone();
		handles.push(tokio::spawn(async move {
			client
				.request(&ListDnsRecords {
					zone_identifier: &zone.id,
					params: ListDnsRecordsParams::default(),
				})
				.await
		}));
	}

	for handle in handles {
		records.extend(handle.await??.result)
	}
	Ok(records)
}

async fn update_record(
	record: DnsRecord,
	record_ip: IpAddr,
	public_ip: IpAddr,
	client: Arc<Client>,
	zone: String,
) -> Result<()> {
	if public_ip == record_ip {
		log::info!("{} skipped, up to date", record.name);
		return Ok(());
	}

	log::info!("{} ({} → {})\n", record.name, record_ip, public_ip);
	io::stdout().flush().ok();

	client
		.request(&UpdateDnsRecord {
			zone_identifier: &zone,
			identifier: &record.id,
			params: UpdateDnsRecordParams {
				name: &record.name,
				ttl: record.ttl.into(),
				proxied: record.proxied.into(),
				content: match public_ip {
					IpAddr::V4(ip) => DnsContent::A { content: ip },
					IpAddr::V6(ip) => DnsContent::AAAA { content: ip },
				},
			},
		})
		.await?;
	Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
	let cli = Cli::parse();

	pretty_env_logger::formatted_builder()
		.filter_level(cli.verbose.log_level_filter())
		.init();

	log::debug!("Rquested records to update: {:#?}", cli.records);

	let (public_ipv4, public_ipv6) = get_ips().await?;
	let api_client = Arc::new(get_client(&cli)?);
	let records = get_records(api_client.clone()).await?;
	let mut handles = Vec::with_capacity(records.len());

	for record in records {
		if !cli.records.contains(&record.name) {
			continue;
		}

		let id = record.zone_id.clone();
		let client = api_client.clone();
		match record.content {
			DnsContent::A { content: ipv4 } => match public_ipv4 {
				Some(public) => {
					handles.push(tokio::spawn(async move {
						update_record(
							record,
							IpAddr::V4(ipv4),
							public,
							client,
							id,
						)
						.await
					}));
				}
				None => continue,
			},
			DnsContent::AAAA { content: ipv6 } => match public_ipv6 {
				Some(public) => {
					handles.push(tokio::spawn(async move {
						update_record(
							record,
							IpAddr::V6(ipv6),
							public,
							client,
							id,
						)
						.await
					}));
				}
				None => continue,
			},
			_ => continue,
		}
	}

	for handle in handles {
		handle.await??
	}
	Ok(())
}
