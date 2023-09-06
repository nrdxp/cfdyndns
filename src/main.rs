use log::info;

use cloudflare::{
	endpoints::{
		dns::{
			DnsContent, DnsRecord, ListDnsRecords, ListDnsRecordsParams,
			UpdateDnsRecord, UpdateDnsRecordParams,
		},
		zone::{ListZones, ListZonesParams, Zone},
	},
	framework::{
		async_api::{ApiClient, Client},
		auth::Credentials,
		Environment, HttpApiClientConfig,
	},
};
use std::env;
use std::io;
use std::io::prelude::Write;

use std::net::IpAddr;

use public_ip::{http, Version};

use anyhow::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
	pretty_env_logger::init();

	let (public_ipv4, public_ipv6) = tokio::join!(
		public_ip::addr_with(http::ALL, Version::V4),
		public_ip::addr_with(public_ip::ALL, Version::V6)
	);

	if (None, None) == (public_ipv6, public_ipv4) {
		anyhow::bail!("Could not determine your current public IP address.")
	}

	if let Some(ipv4) = public_ipv4 {
		info!("{}", ipv4);
	}
	if let Some(ipv6) = public_ipv6 {
		info!("{}", ipv6);
	}

	let cloudflare_records_env = env::var("CLOUDFLARE_RECORDS").context("")?;
	let cloudflare_records: Vec<&str> =
		cloudflare_records_env.split(|c: char| c == ',').collect();

	let credentials: Credentials =
		if let Ok(token) = env::var("CLOUDFLARE_APITOKEN") {
			Credentials::UserAuthToken { token }
		} else if let (Ok(key), Ok(email)) =
			(env::var("CLOUDFLARE_APIKEY"), env::var("CLOUDFLARE_EMAIL"))
		{
			Credentials::UserAuthKey { email, key }
		} else {
			anyhow::bail!(
				"Either API token or API key + email pair must be provided"
			)
		};

	let api_client = Client::new(
		credentials,
		HttpApiClientConfig::default(),
		Environment::Production,
	)?;

	let zones = api_client
		.request(&ListZones {
			params: ListZonesParams::default(),
		})
		.await?
		.result;

	for zone in zones {
		let records = api_client
			.request(&ListDnsRecords {
				zone_identifier: &zone.id,
				params: ListDnsRecordsParams::default(),
			})
			.await?
			.result;

		for record in records {
			if !cloudflare_records.contains(&record.name.as_str()) {
				continue;
			}

			match record.content {
				DnsContent::A { content: ipv4 } => match public_ipv4 {
					Some(public) => {
						update_record(
							&record,
							&IpAddr::V4(ipv4),
							&public,
							&api_client,
							&zone,
						)
						.await?
					}
					None => continue,
				},
				DnsContent::AAAA { content: ipv6 } => match public_ipv6 {
					Some(public) => {
						update_record(
							&record,
							&IpAddr::V6(ipv6),
							&public,
							&api_client,
							&zone,
						)
						.await?
					}
					None => continue,
				},
				_ => continue,
			}
		}
	}
	Ok(())
}

async fn update_record(
	record: &DnsRecord,
	record_ip: &IpAddr,
	public_ip: &IpAddr,
	client: &Client,
	zone: &Zone,
) -> Result<()> {
	if public_ip == record_ip {
		info!("{} skipped, up to date", record.name);
		return Ok(());
	}

	print!("{} ({} -> {})... ", record.name, record_ip, public_ip);
	io::stdout().flush().ok();

	client
		.request(&UpdateDnsRecord {
			zone_identifier: &zone.id,
			identifier: &record.id,
			params: UpdateDnsRecordParams {
				name: &record.name,
				ttl: record.ttl.into(),
				proxied: record.proxied.into(),
				content: match public_ip {
					IpAddr::V4(ip) => DnsContent::A { content: *ip },
					IpAddr::V6(ip) => DnsContent::AAAA { content: *ip },
				},
			},
		})
		.await?;
	Ok(())
}
