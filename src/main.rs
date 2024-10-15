use arboard::{Clipboard, Get};
use clap::Parser;
use reqwest::header::HeaderMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use tokio::fs;
const TARGET_STRING: &str = "27b47455f301788ebf9f85d0d1aa90d5";

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
    let _session_type = session_type();
    //assert!(session_type == SessionType::Wayland);

    let toml_string = fs::read_to_string("config.toml")
        .await
        .expect("[-] No config.toml file found");
    let mut config: Config =
        toml::from_str(toml_string.as_str()).expect("[+] Failed to read config.toml");

    let mut auto = true;
    if args.disable_auto {
        auto = false;
    }

    let mut ctx = Clipboard::new().expect("[-] Error reading clipboard");
    let mut last_content = String::new();
    match now.elapsed() {
        Ok(time) => println!("[+] Time: {}", time.as_secs() / 60 / 60),
        Err(e) => eprintln!("SHit {}", e),
    }
    println!("[+] Daemon started. Listening for flags");

    loop {
        // Bounds
        match now.elapsed() {
            Ok(time) => assert!(time.as_secs() < 86400),
            Err(_) => break,
        }
        // Itnerval (No bennefit from lowering the Duration)
        thread::sleep(Duration::from_secs(1));

        let content = match ctx.get_text() {
            Ok(content) => content,
            Err(_) => continue,
        };
        if content.contains(" ") {
            continue;
        }
        if content.len() != TARGET_STRING.len() {
            continue;
        }
        assert!(content.len() == TARGET_STRING.len());

        let valid = is_valid_hex(&content);

        if valid && content != last_content {
            last_content = content.clone();

            match auto {
                true => {
                    println!("[+] Sending Flag");
                    let result = submit_flag(&content, &config).await;
                    if result {
                        println!("Congrats the flag was submited!!");
                        match now.elapsed() {
                            Ok(time) => println!("Time: {}", time.as_secs() / 60 / 60),
                            Err(e) => eprintln!("SHit {}", e),
                        }
                    } else {
                        println!("[-] Sorry Wrong Flag :( or something happened, try submitting the flag and reporting to me about what happened.")
                    }
                }
                false => {
                    use std::io::{stdin, stdout, Write};
                    let mut s = String::new();
                    print!("Please confirm you want to send flag: y/n\n");
                    let _ = stdout().flush();
                    stdin()
                        .read_line(&mut s)
                        .expect("Please Enter a valid string");
                    if let Some(char) = s.chars().next() {
                        match char {
                            'y' => {
                                println!("[+] Sending Flag");
                                let result = submit_flag(&content, &config).await;
                                if result {
                                    println!("Congrats the flag was submited!!");
                                    match now.elapsed() {
                                        Ok(time) => {
                                            println!("[+] Time: {}", time.as_secs() / 60 / 60)
                                        }
                                        Err(e) => eprintln!("SHit {}", e),
                                    }
                                } else {
                                    println!("[-] Sorry Wrong Flag :( or something happened, try submitting the flag and reporting to me about what happened.")
                                }
                            }
                            'n' => println!("Ignoring conntents"),
                            _ => println!("y/n only"),
                        }
                    }
                }
            }
        }
    }
}
#[derive(PartialEq)]
enum SessionType {
    X11,
    Wayland,
    Unknown,
    FAILED,
}

fn session_type() -> SessionType {
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
                println!("{}", text);
                text.contains("pwned")
            } else {
                false
            }
        }
        Err(_) => false,
    }
}
