use std::{thread, time};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use reqwest::blocking::Client as ReqwestClient;
use reqwest::header::HeaderMap;

use serde::{Deserialize, Serialize};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ReqwestClient::new();

    let api_key = "";
    let country = ""; // DK, EE, FI, FR, DE, HK, ID, KZ, LV, LT, MX, NL, PH, PL, PT, RO, RU, ES, SE, UA, GB, VN

    let balance = get_balance(&mut client, api_key)?;
    println!("Balance: {}", balance);

    let (stock, price) = get_number_and_price_count(&mut client, api_key, country)?;
    println!("Stock: {}", stock);
    println!("Price: {}", price);

    let (phone_number, tzid) = order_number(&mut client, api_key, country)?;
    println!("Phone number: {}", phone_number);
    println!("TZID: {}", tzid);
    println!("Waiting for message..");

    let (tx, rx) = channel();
    let handle = thread::spawn(move || loop {
        let messages = get_messages(&mut client, api_key, tzid);
        for message in messages {
            tx.send(message).unwrap();
        }
        thread::sleep(Duration::from_secs(5));
    });

    let message = rx.recv().unwrap();
    println!("Message: {}", message);

    delete_number(&mut client, api_key, tzid);

    handle.join().unwrap();

    Ok(())
}

fn get_balance(client: &mut ReqwestClient, api_key: &str) -> Result<Balance, Box<dyn std::error::Error>> {
    let url = format!("https://vak-sms.com/api/getBalance/?apiKey={}", api_key);
    let resp: Balance = client.get(&url).send()?.error_for_status()?.json()?;

    match resp {
        Balance { balance: _, error: Some(error) } => {
            if error == "apiKeyNotFound" {
                println!("Invalid API Key!");
                std::process::exit(1);
            }
        }
        _ => {}
    }

    Ok(resp)
}

#[derive(Deserialize)]
struct Balance {
    balance: f32,
    error: Option<String>
}

fn get_number_and_price_count(client: &mut ReqwestClient, api_key: &str, country: &str) -> Result<(i32, f32), Box<dyn std::error::Error>> {
    let url = format!("https://vak-sms.com/api/getCountNumber/?apiKey={}&service=dc&country={}&price", api_key, country);
    let resp: HashMap<String, i32> = client.get(&url).send()?.error_for_status()?.json()?;

    Ok((resp["dc"], resp["price"] as f32))
}

#[derive(Deserialize)]
pub struct GetNumberResponse {
    tel: String,
    idNum: String
}

fn order_number(client: &mut ReqwestClient, api_key: &str, country: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let url = format!("https://vak-sms.com/api/getNumber/?apiKey={}&service=dc&country={}", api_key, country);
    let resp: GetNumberResponse = client.get(&url).send()?.error_for_status()?.json()?;

    Ok((format!("+{}", resp.tel), resp.idNum))
}

#[derive(Deserialize)]
pub struct Message {
    tzid: String,
    tel: String,
    text: String
}

fn get_messages(client: &mut ReqwestClient, api_key: &str, tzid: &str) -> Vec<String> {
    let url = format!("https://vak-sms.com/api/getSms/?apiKey={}&idNum={}", api_key, tzid);
    let resp: Vec<Message> = client.get(&url).send().unwrap().error_for_status().unwrap().json().unwrap();

    let mut messages = vec![];
    for message in resp {
        messages.push(message.text);
    }

    messages
}

fn delete_number(client: &mut ReqwestClient, api_key: &str, tzid: &str) {
    let url = format!("https://vak-sms.com/api/setStatus/?apiKey={}&status=end&idNum={}", api_key, tzid);
    let _: HashMap<String, String> = client.get(&url).send().unwrap().error_for_status().unwrap().json().unwrap();
}
