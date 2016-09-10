#[macro_use]
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate trust_dns;

use hyper::Client;
use hyper::header::Connection;

use serde_json::value::*;

use std::env;
use std::io;
use std::io::prelude::*;

use trust_dns::client::Client as DnsClient;
use trust_dns::rr::dns_class::DNSClass;
use trust_dns::rr::record_type::RecordType;
use trust_dns::rr::domain;
use trust_dns::rr::record_data::RData;
use trust_dns::udp::UdpClientConnection;

header! { (XAuthKey, "X-Auth-Key") => [String] }
header! { (XAuthEmail, "X-Auth-Email") => [String] }

const NS1_GOOGLE_COM_IP_ADDR: &'static str = "216.239.32.10:53";

fn env_var(n: &str) -> String {
    let err = format!("Environment Variable '{}' must be set!", &n);
    env::var(n).ok().expect(&err)
}

// overloaded function. no body is treated as a get, body is treated as a put
fn cloudflare_api(client: &hyper::client::Client, url: &str, body: Option<&str>) -> Result<Value, String> {
    let cloudflare_apikey = env_var("CLOUDFLARE_APIKEY");
    let cloudflare_email = env_var("CLOUDFLARE_EMAIL");

    let builder = match body {
        Some(body) => { client.put(url).body(body) }
        None => { client.get(url) }
    };

    let mut response = builder
        .header(Connection::close())
        .header(XAuthKey(cloudflare_apikey.to_owned()))
        .header(XAuthEmail(cloudflare_email.to_owned()))
        .send().unwrap();

    if response.status != hyper::status::StatusCode::Ok {
        let mut body = String::new();
        response.read_to_string(&mut body).unwrap();
        return Err(body);
    }

    let response_json: Value = serde_json::from_iter(response.by_ref().bytes()).unwrap();

    let success = response_json
        .as_object().unwrap()
        .get("success").unwrap()
        .as_bool().unwrap();
    if !success {
        println!("response status={}, but cloudflare success={}", response.status, success);
    }

    Ok(response_json)
}

fn get_current_ip() -> Result<String, ()> {
    let gdns_addr = (NS1_GOOGLE_COM_IP_ADDR).parse().expect("Couldn't get Google DNS Socket Addr");
    let conn = UdpClientConnection::new(gdns_addr).expect("Couldn't open DNS UDP Connection");
    let client = DnsClient::new(conn);

    let name = domain::Name::new();
    let name = name.label("o-o")
        .label("myaddr")
        .label("l")
        .label("google")
        .label("com");
    let response = client.query(&name, DNSClass::IN, RecordType::TXT).unwrap();

    let record = &response.get_answers()[0];
    match record.get_rdata() {
        &RData::TXT(ref txt) => {
            let val = txt.get_txt_data();
            return Ok(val[0].clone())
        },
        _ => return Err(())
    }
}

fn main() {
    let current_ip = get_current_ip().ok().expect("Was unable to determine current IP address.");
    println!("{}", current_ip);
    let client = Client::new();
    let cloudflare_records_env = env_var("CLOUDFLARE_RECORDS");
    let cloudflare_records: Vec<&str> = cloudflare_records_env.split(|c: char| c == ',').collect();

    let zones_url = "https://api.cloudflare.com/client/v4/zones";
    let zones_json = cloudflare_api(&client, zones_url, None).unwrap();

    let zone_ids = zones_json
        .as_object().unwrap()
        .get("result").unwrap()
        .as_array().unwrap()
        .iter()
        .map(|ref zone_node|
            zone_node.find("id").unwrap()
            .as_str().unwrap());

    for zone_id in zone_ids {
        let records_url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records", zone_id);
        let records_json = cloudflare_api(&client, &*records_url, None).unwrap();

        let records = records_json
            .as_object().unwrap()
            .get("result").unwrap()
            .as_array().unwrap()
            .iter();

        for record in records {
            let record_id = record.find("id").unwrap().as_str().unwrap();
            let record_type = record.find("type").unwrap().as_str().unwrap();
            let record_name = record.find("name").unwrap().as_str().unwrap();
            let record_content = record.find("content").unwrap().as_str().unwrap();

            if !cloudflare_records.contains(&record_name) || record_type != "A" {
                continue
            }

            if record_content == current_ip
            {
                println!("{} skipped, up to date", record_name);
                continue;
            }

            print!("{} ({} -> {})... ", record_name, record_content, current_ip);
            io::stdout().flush().ok();

            let record_url = format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                zone_id,
                record_id);
            let record_update_body = format!(
                r#"{{ "id": "{}", "name": "{}", "content": "{}", "type": "{}" }}"#,
                record_id,
                record_name,
                current_ip,
                record_type);
            cloudflare_api(&client, &*record_url, Some(&*record_update_body)).unwrap();
        }
    }
}
