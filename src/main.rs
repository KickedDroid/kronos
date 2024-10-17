use clap::Parser;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use tokio::fs;
const TARGET_STRING: &str = "27b47455f301788ebf9f85d0d1aa90d5";
mod utils;
use anyhow::Result;
use copypasta::{ClipboardContext, ClipboardProvider};
use utils::{is_valid_hex, session_type, submit_flag, Args, Config};
#[tokio::main]
async fn main() -> Result<()> {
    let now = SystemTime::now();

    let args = Args::parse();
    //let _session_type = session_type();
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

    let mut ctx = ClipboardContext::new().expect("[-] Error reading clipboard");
    let mut last_content = String::new();
    match now.elapsed() {
        Ok(time) => println!("[+] Time: {}", time.as_secs() / 60 / 60),
        Err(e) => eprintln!("SHit {}", e),
    }
    println!("[+] Daemon started. Listening for flags");

    let mut flags = 0;
    loop {
        // Bounds
        match now.elapsed() {
            Ok(time) => assert!(time.as_secs() < 86400),
            Err(_) => break,
        }
        if flags == 2 {
            match now.elapsed() {
                Ok(time) => {
                    println!("[+] Time: {}", time.as_secs() / 3600)
                }
                Err(e) => eprintln!("SHit {}", e),
            }

            break;
        }
        // Itnerval (No bennefit from lowering the Duration)
        thread::sleep(Duration::from_secs(1));

        let content = match ctx.get_contents() {
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
                    let res = submit_flag(&content, &config).await;
                    if res {
                        flags += 1;
                    }
                }
                false => {
                    println!("{} ?", content.clone());
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
                                let result = submit_flag(&content, &config).await;
                                if result {
                                    flags += 1;
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
    Ok(())
}
