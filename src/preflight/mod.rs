use std::process::Command;
use std::fs;
use std::path::Path;
use colored::*;
use dialoguer::Confirm;

pub fn run_preflight() -> Result<(), String> {
    println!("\n{}", "🔧 PREFLIGHT CHECK".bold().cyan());
    println!("{}", "═".repeat(50));
    
    let required_bins = vec![
        "bash", "cat", "grep", "awk", "sed", "cut", "sort", "uniq",
        "head", "tail", "wc", "df", "free", "ps", "ss", "ip", "uptime",
        "who", "last", "hostname", "uname", "lscpu", "lsblk"
    ];
    
    let mut missing = Vec::new();
    
    for bin in &required_bins {
        let status = Command::new("which")
            .arg(bin)
            .output();
        
        match status {
            Ok(output) if output.status.success() => {
                print!("{} ", "✓".green());
                println!("{}", bin);
            }
            _ => {
                print!("{} ", "✗".red());
                println!("{} {}", bin, "(missing)".red());
                missing.push(*bin);
            }
        }
    }
    
    let pkg_manager = detect_package_manager();
    println!("\n{} {}", "📦 Package manager:".cyan(), pkg_manager);
    
    if !missing.is_empty() {
        println!("\n{}", "⚠️  Missing dependencies detected".yellow().bold());
        println!("Required: {}", missing.join(", "));
        
        let auto_install = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Automatically install missing dependencies?")
            .default(true)
            .interact()
            .unwrap();
        
        if auto_install {
            install_dependencies(&missing, &pkg_manager)?;
        } else {
            return Err(format!("Missing dependencies: {}", missing.join(", ")));
        }
    }
    
    check_rust_version()?;
    check_network_connectivity()?;
    check_disk_space()?;
    check_permissions()?;
    
    println!("\n{}", "✅ All preflight checks passed!".green().bold());
    Ok(())
}

fn detect_package_manager() -> String {
    if Command::new("apt").arg("--version").output().is_ok() {
        "apt (Debian/Ubuntu)".to_string()
    } else if Command::new("yum").arg("--version").output().is_ok() {
        "yum (RHEL/CentOS)".to_string()
    } else if Command::new("dnf").arg("--version").output().is_ok() {
        "dnf (Fedora)".to_string()
    } else if Command::new("pacman").arg("--version").output().is_ok() {
        "pacman (Arch)".to_string()
    } else if Command::new("apk").arg("--version").output().is_ok() {
        "apk (Alpine)".to_string()
    } else {
        "unknown".to_string()
    }
}

fn install_dependencies(missing: &[&str], pkg_manager: &str) -> Result<(), String> {
    println!("{}", "📥 Installing missing dependencies...".yellow());
    
    let install_cmd = if pkg_manager.contains("apt") {
        format!("sudo apt update && sudo apt install -y {}", missing.join(" "))
    } else if pkg_manager.contains("yum") {
        format!("sudo yum install -y {}", missing.join(" "))
    } else if pkg_manager.contains("dnf") {
        format!("sudo dnf install -y {}", missing.join(" "))
    } else if pkg_manager.contains("pacman") {
        format!("sudo pacman -S --noconfirm {}", missing.join(" "))
    } else if pkg_manager.contains("apk") {
        format!("sudo apk add {}", missing.join(" "))
    } else {
        return Err("No supported package manager found".to_string());
    };
    
    let status = Command::new("sh")
        .arg("-c")
        .arg(&install_cmd)
        .status()
        .map_err(|e| format!("Failed to run installer: {}", e))?;
    
    if status.success() {
        println!("{}", "✓ Dependencies installed successfully".green());
        Ok(())
    } else {
        Err("Installation failed".to_string())
    }
}

fn check_rust_version() -> Result<(), String> {
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|_| "Rust not installed. Please install Rust from https://rustup.rs/".to_string())?;
    
    let version_str = String::from_utf8_lossy(&output.stdout);
    if version_str.contains("1.70") || version_str.contains("1.7") || version_str.contains("1.8") || version_str.contains("1.9") {
        println!("{} Rust: {}", "✓".green(), version_str.trim());
        Ok(())
    } else {
        println!("{} Rust: {} {}", "⚠️".yellow(), version_str.trim(), "(consider updating)".yellow());
        
        let update = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Update Rust to latest version?")
            .default(true)
            .interact()
            .unwrap();
        
        if update {
            let status = Command::new("rustup")
                .arg("update")
                .status()
                .map_err(|e| format!("Failed to update Rust: {}", e))?;
            
            if status.success() {
                println!("{} Rust updated successfully", "✓".green());
            }
        }
        Ok(())
    }
}

fn check_network_connectivity() -> Result<(), String> {
    let status = Command::new("ping")
        .arg("-c")
        .arg("1")
        .arg("-W")
        .arg("2")
        .arg("8.8.8.8")
        .status();
    
    match status {
        Ok(s) if s.success() => {
            println!("{} Network connectivity: OK", "✓".green());
            Ok(())
        }
        _ => {
            println!("{} Network connectivity: LIMITED (may affect AI features)", "⚠️".yellow());
            Ok(())
        }
    }
}

fn check_disk_space() -> Result<(), String> {
    let output = Command::new("df")
        .arg("-BG")
        .arg("/")
        .output()
        .map_err(|e| format!("Failed to check disk: {}", e))?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();
    
    if lines.len() >= 2 {
        let parts: Vec<&str> = lines[1].split_whitespace().collect();
        if parts.len() >= 5 {
            let available_str = parts[3].replace("G", "");
            if let Ok(available_gb) = available_str.parse::<u64>() {
                if available_gb < 1 {
                    println!("{} Disk space: LOW ({}/GB available)", "⚠️".yellow(), available_gb);
                } else {
                    println!("{} Disk space: {}GB available", "✓".green(), available_gb);
                }
            }
        }
    }
    Ok(())
}

fn check_permissions() -> Result<(), String> {
    let test_path = Path::new("/tmp/vps_inspector_test");
    let write_test = fs::write(test_path, "test");
    
    match write_test {
        Ok(_) => {
            let _ = fs::remove_file(test_path);
            println!("{} Write permissions: OK", "✓".green());
            Ok(())
        }
        Err(_) => {
            println!("{} Write permissions: LIMITED (may affect report generation)", "⚠️".yellow());
            Ok(())
        }
    }
}