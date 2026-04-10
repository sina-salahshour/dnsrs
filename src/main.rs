use anyhow::{Result, anyhow};
use clap::CommandFactory;
use clap::{Parser, Subcommand};
use clap_complete::{Shell, generate};
use inquire::{Select, Text};
use serde::{Deserialize, Serialize};
use std::io;
use std::{fs, path::PathBuf};

use libc;
use std::os::unix::process::CommandExt;
use std::process::Command;

fn elevate_if_needed() {
    if unsafe { libc::geteuid() } != 0 {
        println!("🔐 Elevating privileges...");

        let err = Command::new("sudo")
            .arg("-E") // preserve env (important for config paths, etc.)
            .arg(std::env::current_exe().unwrap())
            .args(std::env::args().skip(1))
            .exec(); // replaces current process

        // If exec() returns, it failed
        eprintln!("Failed to elevate privileges: {}", err);
        std::process::exit(1);
    }
}

fn generate_completions(shell: Shell) {
    let mut cmd = Cli::command();

    generate(shell, &mut cmd, "dnsrs", &mut io::stdout());
}

#[derive(Parser)]
#[command(name = "dnsrs")]
#[command(about = "DNS manager for Linux", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        name: Option<String>,
        ips: Vec<String>,
    },
    Set {
        name: Option<String>,
    },
    Current,
    Undo,
    Reset,
    List,

    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[derive(Serialize, Deserialize, Default)]
struct Profiles {
    profiles: Vec<Profile>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Profile {
    name: String,
    ips: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
struct History {
    stack: Vec<Vec<String>>,
    original: Option<Vec<String>>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Current | Commands::List | Commands::Completions { shell: _ } => {}
        _ => {
            elevate_if_needed();
        }
    };
    match cli.command {
        Commands::Completions { shell } => {
            generate_completions(shell);
            Ok(())
        }

        Commands::Add { name, ips } => add_profile(name, ips),
        Commands::Set { name } => set_profile(name),
        Commands::Current => show_current(),
        Commands::Undo => undo(),
        Commands::Reset => reset(),
        Commands::List => list_profiles(),
    }
}

fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap().join("dnsrs")
}

fn profiles_path() -> PathBuf {
    config_dir().join("profiles.json")
}

fn history_path() -> PathBuf {
    config_dir().join("history.json")
}

fn ensure_files() -> Result<()> {
    fs::create_dir_all(config_dir())?;

    if !profiles_path().exists() {
        fs::write(
            profiles_path(),
            serde_json::to_string_pretty(&Profiles::default())?,
        )?;
    }

    if !history_path().exists() {
        fs::write(
            history_path(),
            serde_json::to_string_pretty(&History::default())?,
        )?;
    }

    Ok(())
}

fn load_profiles() -> Result<Profiles> {
    ensure_files()?;
    let data = fs::read_to_string(profiles_path())?;
    Ok(serde_json::from_str(&data)?)
}

fn save_profiles(p: &Profiles) -> Result<()> {
    fs::write(profiles_path(), serde_json::to_string_pretty(p)?)?;
    Ok(())
}

fn load_history() -> Result<History> {
    ensure_files()?;
    let data = fs::read_to_string(history_path())?;
    Ok(serde_json::from_str(&data)?)
}

fn save_history(h: &History) -> Result<()> {
    fs::write(history_path(), serde_json::to_string_pretty(h)?)?;
    Ok(())
}

fn add_profile(name: Option<String>, mut ips: Vec<String>) -> Result<()> {
    let name = match name {
        Some(n) => n,
        None => Text::new("Profile name:").prompt()?,
    };

    if ips.is_empty() {
        loop {
            let ip = Text::new("Enter DNS IP (leave empty to finish):").prompt()?;
            if ip.trim().is_empty() {
                break;
            }
            ips.push(ip);
        }
    }

    if ips.is_empty() {
        return Err(anyhow!("No IPs provided"));
    }

    let mut profiles = load_profiles()?;
    profiles.profiles.push(Profile { name, ips });
    save_profiles(&profiles)?;

    println!("✅ Profile added");
    Ok(())
}

fn set_profile(name: Option<String>) -> Result<()> {
    let profiles = load_profiles()?;

    if profiles.profiles.is_empty() {
        return Err(anyhow!("No profiles found"));
    }

    let profile = match name {
        Some(n) => profiles
            .profiles
            .into_iter()
            .find(|p| p.name == n)
            .ok_or(anyhow!("Profile not found"))?,
        None => {
            let names: Vec<_> = profiles.profiles.iter().map(|p| p.name.clone()).collect();
            let selected = Select::new("Select profile:", names).prompt()?;
            profiles
                .profiles
                .into_iter()
                .find(|p| p.name == selected)
                .unwrap()
        }
    };

    let current_dns = get_current_dns()?;
    let mut history = load_history()?;

    if history.original.is_none() {
        history.original = Some(current_dns.clone());
    }

    history.stack.push(current_dns);
    save_history(&history)?;

    set_dns(&profile.ips)?;

    println!("✅ DNS set to profile '{}'", profile.name);
    Ok(())
}

fn get_current_dns() -> Result<Vec<String>> {
    let content = fs::read_to_string("/etc/resolv.conf")?;
    let dns: Vec<String> = content
        .lines()
        .filter(|l| l.starts_with("nameserver"))
        .map(|l| l.split_whitespace().nth(1).unwrap().to_string())
        .collect();

    Ok(dns)
}

fn set_dns(ips: &Vec<String>) -> Result<()> {
    let mut content = String::new();
    for ip in ips {
        content.push_str(&format!("nameserver {}\n", ip));
    }

    fs::write("/etc/resolv.conf", content)?;
    Ok(())
}

fn show_current() -> Result<()> {
    let dns = get_current_dns()?;
    println!("Current DNS:");
    for d in dns {
        println!(" - {}", d);
    }
    Ok(())
}

fn undo() -> Result<()> {
    let mut history = load_history()?;

    let last = history.stack.pop().ok_or(anyhow!("No history"))?;
    set_dns(&last)?;
    save_history(&history)?;

    println!("↩️ Reverted DNS");
    Ok(())
}

fn reset() -> Result<()> {
    let mut history = load_history()?;

    let original = history
        .original
        .clone()
        .ok_or(anyhow!("No original DNS stored"))?;
    set_dns(&original)?;

    history.stack.clear();
    save_history(&history)?;

    println!("🔄 Reset to original DNS");
    Ok(())
}

fn list_profiles() -> Result<()> {
    let profiles = load_profiles()?;

    if profiles.profiles.is_empty() {
        println!("No profiles.");
        return Ok(());
    }

    for p in profiles.profiles {
        println!("{}:", p.name);
        for ip in p.ips {
            println!("  - {}", ip);
        }
    }

    Ok(())
}
