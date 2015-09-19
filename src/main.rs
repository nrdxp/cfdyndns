extern crate env_logger;
#[macro_use] 
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate trust_dns;

use hyper::Client;
use hyper::header::Connection;

use serde_json::value::*;

use std::env;
use std::io::Read;

use trust_dns::rr::dns_class::DNSClass;
use trust_dns::rr::record_type::RecordType;
use trust_dns::rr::domain;
use trust_dns::rr::record_data::RData;
use trust_dns::udp::client::Client as DnsClient;

header! { (XAuthKey, "X-Auth-Key") => [String] }
header! { (XAuthEmail, "X-Auth-Email") => [String] }

// TODO(colemickens): none of the implementations handle paging properly

fn cloudflare_api(client: &hyper::client::Client, url: &str, body: Option<&str>) -> Result<Value, String> {
    let cloudflare_apikey = env::var("CLOUDFLARE_APIKEY").expect("missing apikey");
    let cloudflare_email = env::var("CLOUDFLARE_EMAIL").expect("missing email");

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
        println!("she's gonna blow! {}", response.status);
        let mut body = String::new();
        response.read_to_string(&mut body).unwrap();
        println!("{}", body);
        println!("exit early"); // TODO(colemickens): fix
    }

    let response_json: Value = serde_json::from_iter(response.by_ref().bytes()).unwrap();

    let status = response_json
        .as_object().unwrap()
        .get("success").unwrap()
        .as_boolean().unwrap();
    if !status {
        println!("request sent 200, but cloudflare reported !success"); // TODO(colemickens): this is inaccurate due to above "fix" todo
    }

    Ok(response_json)
}

fn get_current_ip() -> Result<String, ()> {
    let client = DnsClient::new(("8.8.8.8").parse().unwrap()).unwrap();

    let name = domain::Name::with_labels(vec![
        "o-o".to_string(),
        "myaddr".to_string(),
        "l".to_string(),
        "google".to_string(),
        "com".to_string()]);
    let response = client.query(name.clone(), DNSClass::IN, RecordType::TXT).unwrap();

    let record = &response.get_answers()[0];
    if let &RData::TXT{ ref txt_data } = record.get_rdata() {
        return Ok(txt_data[0].to_string());
    } else {
        return Err(())
    }
}

fn main() {
    env_logger::init().unwrap();

    let current_ip = get_current_ip().expect("must have current ip");
    let client = Client::new();
    let cloudflare_records_env = env::var("CLOUDFLARE_RECORDS").expect("missing records");
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
            .as_string().unwrap());

    for zone_id in zone_ids {
        let records_url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records", zone_id);
        let records_json = cloudflare_api(&client, &*records_url, None).unwrap();

        let records = records_json
            .as_object().unwrap()
            .get("result").unwrap()
            .as_array().unwrap()
            .iter();

        for record in records {
            let record_id = record.find("id").unwrap().as_string().unwrap();
            let record_type = record.find("type").unwrap().as_string().unwrap();
            let record_name = record.find("name").unwrap().as_string().unwrap();
            let record_content = record.find("content").unwrap().as_string().unwrap();

            if record_type == "A" && cloudflare_records.contains(&record_name) {
                if record_content == current_ip
                {
                    println!("{} skipped, up to date", record_name);
                    continue;
                } else {
                    println!("{} needs update... ({} != {})", record_name, record_content, current_ip);
                    
                    let record_url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}", zone_id, record_id);
                    let record_update_body = format!(
                        "{{ \"id\": \"{}\", \"name\": \"{}\", \"content\": \"{}\", \"type\": \"{}\" }}",
                        record_id,
                        record_name,
                        current_ip,
                        record_type);
                    println!("{}", record_update_body);
                    cloudflare_api(&client, &*record_url, Some(&*record_update_body)).unwrap();
                    
                    println!("{} updated!", record_name);
                }
            }
        }
    }
}
