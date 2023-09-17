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

use public_ip::{http, Version};

use std::net::IpAddr;

use anyhow::Result;

use crate::api::Cli;
use std::sync::Arc;

use crate::clone::Clone_;

use std::collections::HashSet;

pub async fn get_ips() -> Result<(Option<IpAddr>, Option<IpAddr>)> {
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

pub fn get_client(cli: &Cli) -> Result<Client> {
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

pub async fn get_records(
	cli: &Cli,
	client: Arc<Client>,
) -> Result<Vec<(String, Option<String>, Option<DnsRecord>, Option<DnsRecord>)>>
{
	let params = ListZones {
		params: ListZonesParams::default(),
	};
	let zones = client.request(&params);
	// HACK: make a second call since dns::Zone does not implement Clone upstream
	let zones2 = client.request(&params).await?.result;

	let mut handles = Vec::with_capacity(zones2.len());
	let mut records = Vec::with_capacity(zones2.len() * 10);

	for zone in zones.await?.result {
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

	let mut start: HashSet<String> = cli.records.clone().into_iter().collect();
	let mut invalid = vec![];
	start.retain(|name| {
		let mut res = false;
		if let Ok(n) = addr::parse_domain_name(name) {
			res = n.has_known_suffix();
		}
		if !res {
			invalid.push(name.clone());
		}
		res
	});

	for name in invalid {
		log::warn!("{} is an invalid domain name; skipping...", name);
	}

	let locals = start
		.iter()
		.map(|r| {
			(
				r.to_owned(),
				zones2
					.iter()
					.find(|z| r.contains(&z.name))
					.map(|z| z.id.to_owned()),
				records
					.iter()
					.find(|rec| {
						if let DnsContent::A { content: _ } = rec.content {
							return rec.name == *r;
						}
						false
					})
					.map(|r| r.clone()),
				records
					.iter()
					.find(|rec| {
						if let DnsContent::AAAA { content: _ } = rec.content {
							return rec.name == *r;
						}
						false
					})
					.map(|r| r.clone()),
			)
		})
		.collect();
	Ok(locals)
}

pub async fn update_record(
	record: DnsRecord,
	record_ip: IpAddr,
	public_ip: IpAddr,
	client: Arc<Client>,
) -> Result<()> {
	if public_ip == record_ip {
		let kind = match record_ip {
			IpAddr::V4(_) => "A",
			IpAddr::V6(_) => "AAAA",
		};
		log::info!("{} record {} skipped, up to date", kind, record.name);
		return Ok(());
	}

	log::info!("{} ({} → {})\n", record.name, record_ip, public_ip);

	client
		.request(&UpdateDnsRecord {
			zone_identifier: &record.zone_id,
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
