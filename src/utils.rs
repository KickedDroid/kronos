use clap::Parser;
use reqwest::header::HeaderMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
const TARGET_STRING: &str = "27b47455f301788ebf9f85d0d1aa90d5";
const API_URL: &str = "https://labs.hackthebox.com/";
use reqwest::Client;
use serde_json::json;
use std::io::{Read, Result};
use std::path::PathBuf;
#[derive(Deserialize)]
pub struct Config {
    pub htb: HashMap<String, String>,
}

#[derive(Parser, Debug)]
#[clap(name = "Kronos")]
#[clap(version = "0.1")]
#[clap(about = "keep track of the hack")]
#[clap(long_about = None)]
pub struct Args {
    #[clap(short, long, long_help = "Disable auto flag submission")]
    pub disable_auto: bool,
    #[clap(default_value = "./", long_help = "Output directory")]
    pub output: PathBuf,
    #[clap(default_value = "htb", long_help = "Session name")]
    pub name: PathBuf,
}

pub fn is_valid_hex(s: &str) -> bool {
    for c in s.chars() {
        match c {
            '0'..='9' | 'a'..='f' | 'A'..='F' => continue,
            _ => return false,
        }
    }
    assert!(s.len() == TARGET_STRING.len());
    true
}

#[derive(PartialEq)]
pub enum SessionType {
    X11,
    Wayland,
    Unknown,
    FAILED,
}

pub fn session_type() -> SessionType {
    if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
        match session_type.as_str() {
            "x11" => {
                println!("[+] Running on X11");
                SessionType::X11
            }
            "wayland" => {
                println!("[+] Running on Wayland.");
                SessionType::Wayland
            }
            _ => SessionType::Unknown,
        }
    } else {
        println!("[-] Could not read XDG_SESSION_TYPE");
        SessionType::FAILED
    }
}

pub async fn submit_flag(flag: &str, config: &Config) -> bool {
    println!("[+] Sending Flag");
    let client = Client::new();
    let url = format!("{}api/v4/arena/own", API_URL);
    let submit_flag_request = json!({
        "flag": flag,
    });

    if config.htb["api_token"].len() == 0 {
        println!("[-] No token added");
        return false;
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Bearer {}", config.htb["api_token"])
            .parse()
            .expect("[-] Token Error"),
    );
    headers.insert(
        "Content-Type",
        "application/json"
            .to_string()
            .parse()
            .expect("[-] Failed to parse Content-Type"),
    );

    let response = client
        .post(&url)
        .headers(headers)
        .json(&submit_flag_request)
        .send()
        .await;

    match response {
        Ok(res) => {
            if let Ok(text) = res.text().await {
                //println!("{}", text);
                let res = handle_response(text).await;
                res
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

async fn handle_response(response: String) -> bool {
    if response.contains("pwned") {
        println!("Congrats the flag was submited!!");
        true
    } else if response.contains("Incorrect") {
        println!("[-] Sorry Wrong Flag :(");
        println!("{response}");
        false
    } else {
        println!("[-] Error handling response");
        println!("{response}");
        false
    }
}

pub fn open_or_create_file(filename: &str) -> Result<File> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(true)
        .open(filename)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(file)
}
