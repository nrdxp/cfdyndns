use log::info;

use serde_json::value::*;

use std::env;
use std::io;
use std::io::prelude::*;

use public_ip::{http, Version};

fn env_var(n: &str) -> String {
	let err = "Environment Variables CLOUDFLARE_RECORDS and either CLOUDFLARE_APITOKEN or CLOUDFLARE_EMAIL and CLOUDFLARE_APIKEY must be set!";
	env::var(n).expect(err)
}

// overloaded function. no body is treated as a get, body is treated as a put
fn cloudflare_api(
	client: &reqwest::blocking::Client,
	url: &str,
	body: Option<String>,
) -> Result<Value, String> {
	let request = match body {
		Some(body) => client.put(url).body(body),
		None => client.get(url),
	};

	let authorized_request = match env::var("CLOUDFLARE_APITOKEN") {
		Ok(val) => {
			let mut bearer = "Bearer ".to_owned();
			bearer.push_str(&val);
			request.header("Authorization", bearer.to_owned())
		}
		Err(_e) => {
			let cloudflare_apikey = env_var("CLOUDFLARE_APIKEY");
			let cloudflare_email = env_var("CLOUDFLARE_EMAIL");
			request
				.header("X-Auth-Key", cloudflare_apikey.to_owned())
				.header("X-Auth-Email", cloudflare_email.to_owned())
		}
	};

	let response_json: Value =
		authorized_request.send().unwrap().json().unwrap();

	let success = response_json
		.as_object()
		.unwrap()
		.get("success")
		.unwrap()
		.as_bool()
		.unwrap();
	if !success {
		return Err(format!("Request not successful: {}", response_json));
	}

	Ok(response_json)
}

#[tokio::main]
async fn main() {
	pretty_env_logger::init();

	let current_ipv4 = public_ip::addr_with(http::ALL, Version::V4)
		.await
		.expect("Was unable to determine current IP address.");
	info!("{}", current_ipv4);
	let client = reqwest::blocking::Client::new();

	let cloudflare_records_env = env_var("CLOUDFLARE_RECORDS");
	let cloudflare_records: Vec<&str> =
		cloudflare_records_env.split(|c: char| c == ',').collect();

	let zones_url = "https://api.cloudflare.com/client/v4/zones";
	let zones_json = cloudflare_api(&client, zones_url, None).unwrap();

	let zone_ids = zones_json
		.as_object()
		.unwrap()
		.get("result")
		.unwrap()
		.as_array()
		.unwrap()
		.iter()
		.map(|zone_node| zone_node.get("id").unwrap().as_str().unwrap());

	for zone_id in zone_ids {
		let records_url = format!(
			"https://api.cloudflare.com/client/v4/zones/{}/dns_records",
			zone_id
		);
		let records_json = cloudflare_api(&client, &records_url, None).unwrap();

		let records = records_json
			.as_object()
			.unwrap()
			.get("result")
			.unwrap()
			.as_array()
			.unwrap()
			.iter();

		for record in records {
			let record_id = record.get("id").unwrap().as_str().unwrap();
			let record_type = record.get("type").unwrap().as_str().unwrap();
			let record_name = record.get("name").unwrap().as_str().unwrap();
			let record_content =
				record.get("content").unwrap().as_str().unwrap();
			let record_proxied =
				record.get("proxied").unwrap().as_bool().unwrap();

			if !cloudflare_records.contains(&record_name) || record_type != "A"
			{
				continue;
			}

			if record_content == current_ipv4.to_string() {
				info!("{} skipped, up to date", record_name);
				continue;
			}

			print!(
				"{} ({} -> {})... ",
				record_name, record_content, current_ipv4
			);
			io::stdout().flush().ok();

			let record_url = format!(
				"https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
				zone_id, record_id
			);
			let record_update_body = format!(
				r#"{{"name": "{}", "content": "{}", "type": "{}", "proxied": {}}}"#,
				record_name, current_ipv4, record_type, record_proxied
			);
			cloudflare_api(
				&client,
				&record_url,
				Some(record_update_body.to_string()),
			)
			.unwrap();
		}
	}
}
