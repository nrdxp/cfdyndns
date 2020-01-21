extern crate log;
extern crate pretty_env_logger;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate trust_dns;

use log::info;

use serde_json::value::*;

use std::env;
use std::io;
use std::io::prelude::*;

use trust_dns::client::{Client as _Client, SyncClient};
use trust_dns::rr::dns_class::DNSClass;
use trust_dns::rr::domain;
use trust_dns::rr::record_data::RData;
use trust_dns::rr::record_type::RecordType;
use trust_dns::udp::UdpClientConnection;

const NS1_GOOGLE_COM_IP_ADDR: &'static str = "216.239.32.10:53";

fn env_var(n: &str) -> String {
    let err = format!("Environment Variable '{}' must be set!", &n);
    env::var(n).ok().expect(&err)
}

// overloaded function. no body is treated as a get, body is treated as a put
fn cloudflare_api(
    client: &reqwest::Client,
    url: &str,
    body: Option<String>,
) -> Result<Value, String> {
    let cloudflare_apikey = env_var("CLOUDFLARE_APIKEY");
    let cloudflare_email = env_var("CLOUDFLARE_EMAIL");

    let request = match body {
        Some(body) => client.put(url).body(body),
        None => client.get(url),
    };
    let response_json: Value = request
        .header("X-Auth-Key", cloudflare_apikey.to_owned())
        .header("X-Auth-Email", cloudflare_email.to_owned())
        .send()
        .unwrap()
        .json()
        .unwrap();

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

fn get_current_ip() -> Result<String, ()> {
    let gdns_addr = (NS1_GOOGLE_COM_IP_ADDR)
        .parse()
        .expect("Couldn't get Google DNS Socket Addr");
    let conn = UdpClientConnection::new(gdns_addr).expect("Couldn't open DNS UDP Connection");
    let client = SyncClient::new(conn);

    let name = domain::Name::new();
    let name = name
        .append_label("o-o")
        .unwrap()
        .append_label("myaddr")
        .unwrap()
        .append_label("l")
        .unwrap()
        .append_label("google")
        .unwrap()
        .append_label("com")
        .unwrap();
    let response = client.query(&name, DNSClass::IN, RecordType::TXT).unwrap();

    let record = &response.answers()[0];
    match record.rdata() {
        &RData::TXT(ref txt) => {
            let val = txt.txt_data();
            return Ok(String::from_utf8(val[0].to_vec()).unwrap());
        }
        _ => return Err(()),
    }
}

fn main() {
    pretty_env_logger::init();

    let current_ip = get_current_ip()
        .ok()
        .expect("Was unable to determine current IP address.");
    info!("{}", current_ip);
    let client = reqwest::Client::new();

    let cloudflare_records_env = env_var("CLOUDFLARE_RECORDS");
    let cloudflare_records: Vec<&str> = cloudflare_records_env.split(|c: char| c == ',').collect();

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
        .map(|ref zone_node| zone_node.get("id").unwrap().as_str().unwrap());

    for zone_id in zone_ids {
        let records_url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        );
        let records_json = cloudflare_api(&client, &*records_url, None).unwrap();

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
            let record_content = record.get("content").unwrap().as_str().unwrap();

            if !cloudflare_records.contains(&record_name) || record_type != "A" {
                continue;
            }

            if record_content == current_ip {
                info!("{} skipped, up to date", record_name);
                continue;
            }

            print!("{} ({} -> {})... ", record_name, record_content, current_ip);
            io::stdout().flush().ok();

            let record_url = format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                zone_id, record_id
            );
            let record_update_body = format!(
                r#"{{"name": "{}", "content": "{}", "type": "{}", "proxied": true}}"#,
                record_name, current_ip, record_type
            );
            cloudflare_api(&client, &*record_url, Some(record_update_body.to_string())).unwrap();
        }
    }
}
