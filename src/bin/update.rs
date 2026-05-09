use std::process::Command;
use std::fs;
use std::path::Path;
use reqwest::Client;
use serde_json::Value;

#[tokio::main]
async fn main() {
    println!("🔄 VPS Inspector Professional Updater");
    println!("═══════════════════════════════════════");
    
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: v{}", current_version);
    
    match get_latest_version().await {
        Ok(latest) => {
            println!("Latest version: v{}", latest);
            
            if latest != current_version {
                println!("\n✨ New version available!");
                perform_update().await;
            } else {
                println!("\n✅ Already on the latest version!");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to check version: {}", e);
            println!("\n🔄 Attempting force update from GitHub...");
            perform_update().await;
        }
    }
}

async fn get_latest_version() -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    
    let url = "https://api.github.com/repos/akaanakbaik/vps-inspec/releases/latest";
    let response = client
        .get(url)
        .header("User-Agent", "vps-inspec-updater")
        .send()
        .await?;
    
    let json: Value = response.json().await?;
    let tag = json["tag_name"].as_str().unwrap_or("v1.0.0");
    Ok(tag.trim_start_matches('v').to_string())
}

async fn perform_update() {
    println!("📥 Pulling latest code from repository...");
    
    let status = Command::new("git")
        .arg("pull")
        .status();
    
    match status {
        Ok(s) if s.success() => {
            println!("✓ Code pulled successfully");
            rebuild_project().await;
        }
        _ => {
            println!("⚠️ Git pull failed, attempting fresh clone...");
            fresh_clone().await;
        }
    }
}

async fn rebuild_project() {
    println!("🔨 Rebuilding project...");
    
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .status();
    
    match status {
        Ok(s) if s.success() => {
            println!("✓ Build successful");
            install_binary().await;
        }
        _ => {
            eprintln!("❌ Build failed");
            std::process::exit(1);
        }
    }
}

async fn fresh_clone() {
    let backup_dir = "/tmp/vps-inspec-backup";
    let _ = fs::remove_dir_all(backup_dir);
    
    if Path::new("/usr/local/bin/vps-inspec").exists() {
        let _ = Command::new("sudo")
            .args(["cp", "/usr/local/bin/vps-inspec", backup_dir])
            .status();
    }
    
    let repo_url = "https://github.com/akaanakbaik/vps-inspec.git";
    let clone_dir = "/tmp/vps-inspec-new";
    
    let _ = fs::remove_dir_all(clone_dir);
    
    let status = Command::new("git")
        .args(["clone", repo_url, clone_dir])
        .status();
    
    match status {
        Ok(s) if s.success() => {
            println!("✓ Fresh clone successful");
            
            let build_status = Command::new("cargo")
                .current_dir(clone_dir)
                .args(["build", "--release"])
                .status();
            
            if let Ok(s) = build_status {
                if s.success() {
                    let _ = Command::new("sudo")
                        .args(["cp", &format!("{}/target/release/vps-inspec", clone_dir), "/usr/local/bin/"])
                        .status();
                    println!("✓ Binary installed");
                }
            }
            
            let _ = fs::remove_dir_all(clone_dir);
        }
        _ => {
            eprintln!("❌ Clone failed");
            if Path::new(backup_dir).exists() {
                println!("🔄 Restoring from backup...");
                let _ = Command::new("sudo")
                    .args(["cp", &format!("{}/vps-inspec", backup_dir), "/usr/local/bin/"])
                    .status();
            }
            std::process::exit(1);
        }
    }
}

async fn install_binary() {
    let status = Command::new("sudo")
        .args(["cp", "target/release/vps-inspec", "/usr/local/bin/"])
        .status();
    
    match status {
        Ok(s) if s.success() => {
            println!("✓ Binary installed to /usr/local/bin/vps-inspec");
            println!("\n🎉 Update completed successfully!");
            println!("Run 'vps-inspec' to start using the new version.");
        }
        _ => {
            eprintln!("❌ Failed to install binary");
            std::process::exit(1);
        }
    }
}