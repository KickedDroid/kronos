use clap::Parser;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use tokio::fs;
const TARGET_STRING: &str = "27b47455f301788ebf9f85d0d1aa90d5";
mod utils;
use anyhow::Result;
use chrono::{DateTime, Local};
use copypasta::{ClipboardContext, ClipboardProvider};
use std::io::Write;
use std::path::Path;
use utils::{is_valid_hex, open_or_create_file, submit_flag, Args, Config};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup
    let now = SystemTime::now();
    let args = Args::parse();

    if !Path::new("config.toml").is_file() {
        println!("[-] No config.toml file found");
        return Ok(());
    };

    if !args.output.clone().is_dir() {
        println!("{} is not a directory", args.output.to_str().unwrap());
        return Ok(());
    }

    if args.name.clone().to_str().unwrap().contains("\"")
        || args.name.clone().to_str().unwrap().contains("/")
    {
        println!("{} is not a valid file name", args.name.to_str().unwrap());
        return Ok(());
    }
    let mut output_file = open_or_create_file(
        format!(
            "{}/{}_kron_history.md",
            args.output.to_str().unwrap(),
            args.name.to_str().unwrap()
        )
        .as_str(),
    )
    .unwrap();
    // Time
    let time: DateTime<Local> = Local::now();
    println!("[+] Time: {time}");
    //Config
    let toml_string = fs::read_to_string("config.toml")
        .await
        .expect("[-] No config.toml file found");
    let config: Config =
        toml::from_str(toml_string.as_str()).expect("[+] Failed to read config.toml");

    if config.htb["api_token"].len() == 0 {
        println!("[-] No token added");
    }
    let mut auto = true;
    if args.disable_auto {
        auto = false;
    }

    // ClipBoard
    let mut ctx = ClipboardContext::new().expect("[-] Error reading clipboard");
    let mut last_content = String::new();
    println!("[+] Daemon started. Listening for flags");

    let mut flags = 0;
    loop {
        // Bounds
        match now.elapsed() {
            Ok(time) => assert!(time.as_secs() < 86400),
            Err(_) => break,
        }
        if flags == 2 {
            let time: DateTime<Local> = Local::now();
            println!("[+] Finished Time: {}\n", time);
            println!("[+] Congratulations you have succesfully pwned the machine!");

            break;
        }

        // Itnerval (No bennefit from lowering the Duration)
        thread::sleep(Duration::from_secs(1));

        let content = match ctx.get_contents() {
            Ok(content) => {
                if content != last_content {
                    writeln!(output_file, "").unwrap();
                    writeln!(output_file, "\n```\n{content}\n```\n").unwrap();
                    last_content = content.clone();
                    content
                } else {
                    continue;
                }
            }
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

        if valid {
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
                            'n' => println!("Ignoring contents"),
                            _ => println!("y/n only"),
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
