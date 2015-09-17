extern crate env_logger;
#[macro_use] 
extern crate hyper;
extern crate serde;
extern crate serde_json;

use std::env;
use std::io;
use std::io::Read;

use hyper::Client;
use hyper::header::Connection;
use hyper::header::Headers;
use hyper::client::RequestBuilder;

use serde_json::value::*;

header! { (XAuthKey, "X-Auth-Key") => [String] }
header! { (XAuthEmail, "X-Auth-Email") => [String] }

fn main() {
    env_logger::init().unwrap();

    let cloudflare_apikey = env::var("cloudflare_apikey").unwrap();
    let cloudflare_email = env::var("cloudflare_email").unwrap();

    let zonesUrl = "https://api.cloudflare.com/client/v4/zones";

    let client = Client::new();

    let mut auth_headers = hyper::header::Headers::new();

    auth_headers.set(XAuthKey(cloudflare_apikey.to_owned()));
    auth_headers.set(XAuthEmail(cloudflare_email.to_owned()));

    let mut res = client
        .get(zonesUrl)
        .header(Connection::close())
        .headers(auth_headers)
        .send()
        .unwrap();

    // Read the Response.
    let mut body = String::new();
    //res.read_to_string(&mut body).unwrap();

    // serde
    // let json = json::from_str(body).unwrap();
    // let json: serde_json::value::Value = serde_json::from_str(&body).unwrap();
    
    let json: serde_json::value::Value = serde_json::from_iter(res.bytes()).unwrap();

    let zoneIds = json
        .as_object().unwrap()
        .get("result").unwrap()
        .as_array().unwrap()
        .iter()
        .map(|ref id| id.find("id").unwrap()
            .as_string().unwrap());

    for id in zoneIds {
        println!("{}", id);
    }

    println!("Response: {}", res.status);
    println!("Headers:\n{}", res.headers);
    io::copy(&mut res, &mut io::stdout()).unwrap();
}