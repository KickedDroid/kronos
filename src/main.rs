use clap::Parser;
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use reqwest::header::HeaderMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use tokio::fs;
const TARGET_PHRASE: &str = "27b47455f301788ebf9f85d0d1aa90d5";

use reqwest::Client;
use serde_json::json;

const API_URL: &str = "https://labs.hackthebox.com/";
#[derive(Deserialize)]
pub struct Config {
    pub htb: HashMap<String, String>,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    disable_auto: bool,
}

#[tokio::main]
async fn main() {
    let now = SystemTime::now();

    let args = Args::parse();

    let toml_string = fs::read_to_string("config.toml")
        .await
        .expect("No config.toml file found");
    let mut config: Config =
        toml::from_str(toml_string.as_str()).expect("Failed to read config.toml");

    let mut auto = true;
    if args.disable_auto {
        auto = false;
    }

    let mut ctx = ClipboardContext::new().expect("Error reading clipboard");
    let mut last_content = String::new();
    match now.elapsed() {
        Ok(time) => println!("Time: {}", time.as_secs() / 60 / 60),
        Err(e) => eprintln!("SHit {}", e),
    }
    println!("Daemon started. Listening for flags");

    loop {
        thread::sleep(Duration::from_secs(1));

        let content = match ctx.get_contents() {
            Ok(content) => content,
            Err(_) => continue,
        };
        if content.contains(" ") {
            continue;
        }

        if content.len() == TARGET_PHRASE.len() && content != last_content {
            last_content = content.clone();
            if is_valid_hex(&content) && auto {
                println!("Sending Flag");
                let result = submit_flag(&content, &config).await;
                if result {
                    println!("Congrats the flag was submited!!");
                    match now.elapsed() {
                        Ok(time) => println!("Time: {}", time.as_secs() / 60 / 60),
                        Err(e) => eprintln!("SHit {}", e),
                    }
                } else {
                    println!("Sorry Wrong Flag :( or something happened, try submitting the flag and reporting to me about what happened.")
                }
            }
            if is_valid_hex(&content) && !auto {
                use std::io::{stdin, stdout, Write};
                let mut s = String::new();
                print!("Please confirm you want to send flag: y/n\n");
                let _ = stdout().flush();
                stdin()
                    .read_line(&mut s)
                    .expect("Please Enter a valid string");
                if let Some('y') = s.chars().next_back() {
                    println!("Sending Flag");
                    let result = submit_flag(&content, &config).await;
                    if result {
                        println!("Congrats the flag was submited!!");
                        match now.elapsed() {
                            Ok(time) => println!("Time: {}", time.as_secs() / 60 / 60),
                            Err(e) => eprintln!("SHit {}", e),
                        }
                    } else {
                        println!("Sorry Wrong Flag :( or something happened, try submitting the flag and reporting to me about what happened.")
                    }
                } else if s.contains("n") {
                    println!("Skipping contents, not sending flag.")
                }
            }
        }
    }
}

fn is_valid_hex(s: &str) -> bool {
    for c in s.chars() {
        match c {
            '0'..='9' | 'a'..='f' | 'A'..='F' => continue,
            _ => return false,
        }
    }
    true
}

async fn submit_flag(flag: &str, config: &Config) -> bool {
    let client = Client::new();
    let url = format!("{}api/v4/arena/own", API_URL);
    let submit_flag_request = json!({
        "flag": flag,
    });

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Bearer {}", config.htb["api_token"])
            .parse()
            .expect("Failed to read config.toml"),
    );
    headers.insert(
        "Content-Type",
        "application/json"
            .to_string()
            .parse()
            .expect("Failed to parse Content-Type"),
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
                println!("{}", text);
                text.contains("pwned.")
            } else {
                false
            }
        }
        Err(_) => false,
    }
}
