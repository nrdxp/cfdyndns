extern crate env_logger;
#[macro_use] 
extern crate hyper;
extern crate serde;
extern crate serde_json;

use std::env;
use std::io::Read;

use hyper::Client;
use hyper::header::Connection;

use serde_json::value::*;

header! { (XAuthKey, "X-Auth-Key") => [String] }
header! { (XAuthEmail, "X-Auth-Email") => [String] }

// TODO(colemickens): none of the implementations handle paging properly

fn main() {
    env_logger::init().unwrap();

    let cloudflare_apikey = env::var("CLOUDFLARE_APIKEY").expect("missing apikey");
    let cloudflare_email = env::var("CLOUDFLARE_EMAIL").expect("missing email");

    let zones_url = "https://api.cloudflare.com/client/v4/zones";

    let client = Client::new();

    let mut auth_headers = hyper::header::Headers::new();

    auth_headers.set(XAuthKey(cloudflare_apikey.to_owned()));
    auth_headers.set(XAuthEmail(cloudflare_email.to_owned()));

    let res = client
        .get(zones_url)
        .header(Connection::close())
        .headers(auth_headers)
        .send()
        .unwrap();

    let json: serde_json::value::Value = serde_json::from_iter(res.bytes()).unwrap();

    let status = json
        .as_object().unwrap()
        .get("success").unwrap()
        .as_boolean().unwrap();
    if !status {
        println!("failed request!");
    }

    let zone_ids = json
        .as_object().unwrap()
        .get("result").unwrap()
        .as_array().unwrap()
        .iter()
        .map(|ref id| id.find("id").unwrap()
            .as_string().unwrap());

    for id in zone_ids {
        println!("{}", id);
    }

    // println!("Response: {}", res.status);
    // println!("Headers:\n{}", res.headers);
}
