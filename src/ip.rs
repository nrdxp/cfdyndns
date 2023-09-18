use anyhow::Result;
use public_ip::{http, Version};
use std::net::IpAddr;

type IpPair = (Option<IpAddr>, Option<IpAddr>);

pub async fn get_ips() -> Result<IpPair> {
	let (ipv4, ipv6) = tokio::join!(
		public_ip::addr_with(http::ALL, Version::V4),
		public_ip::addr_with(public_ip::ALL, Version::V6)
	);

	if (None, None) == (ipv4, ipv6) {
		Err(anyhow::anyhow!(
			"Could not determine your current public IP address."
		))?
	}

	if let Some(ip) = ipv4 {
		log::info!("{}", ip);
	}
	if let Some(ip) = ipv6 {
		log::info!("{}", ip);
	};
	Ok((ipv4, ipv6))
}
