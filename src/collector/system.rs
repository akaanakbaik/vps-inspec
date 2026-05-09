use super::{MetricGroup, MetricValue, MetricSeverity};
use chrono::Local;
use std::process::Command;
use std::fs;
use regex::Regex;

pub struct SystemCollector {
    start_time: std::time::Instant,
}

impl SystemCollector {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    pub async fn collect(&self) -> MetricGroup {
        let start = std::time::Instant::now();
        let mut metrics = Vec::new();

        metrics.push(self.get_hostname());
        metrics.push(self.get_os_info());
        metrics.push(self.get_kernel_version());
        metrics.push(self.get_architecture());
        metrics.push(self.get_virtualization());
        metrics.push(self.get_uptime());
        metrics.push(self.get_last_boot());
        metrics.push(self.get_load_average());
        metrics.push(self.get_active_users());
        metrics.push(self.get_system_time());
        metrics.push(self.get_timezone());
        metrics.push(self.get_environment());
        metrics.push(self.get_system_type());
        metrics.push(self.get_product_name());
        metrics.push(self.get_serial_number().unwrap_or_else(|| "N/A".to_string()));

        let duration_ms = start.elapsed().as_millis() as u64;

        MetricGroup {
            category: "System Information".to_string(),
            metrics,
            collected_at: Local::now(),
            duration_ms,
        }
    }

    fn get_hostname(&self) -> MetricValue {
        let hostname = Command::new("hostname")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string();

        MetricValue {
            name: "hostname".to_string(),
            value: hostname,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_os_info(&self) -> MetricValue {
        let os_release = fs::read_to_string("/etc/os-release").unwrap_or_default();
        let mut name = "Unknown".to_string();
        let mut version = "".to_string();

        for line in os_release.lines() {
            if line.starts_with("PRETTY_NAME=") {
                let pretty = line.trim_start_matches("PRETTY_NAME=").trim_matches('"');
                name = pretty.to_string();
            }
            if line.starts_with("VERSION_ID=") {
                version = line.trim_start_matches("VERSION_ID=").trim_matches('"').to_string();
            }
        }

        let value = if version.is_empty() {
            name
        } else {
            format!("{} {}", name, version)
        };

        MetricValue {
            name: "operating_system".to_string(),
            value,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_kernel_version(&self) -> MetricValue {
        let kernel = Command::new("uname")
            .arg("-r")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string();

        MetricValue {
            name: "kernel_version".to_string(),
            value: kernel,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_architecture(&self) -> MetricValue {
        let arch = Command::new("uname")
            .arg("-m")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string();

        MetricValue {
            name: "architecture".to_string(),
            value: arch,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_virtualization(&self) -> MetricValue {
        let virt = Command::new("systemd-detect-virt")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                let product = fs::read_to_string("/sys/devices/virtual/dmi/id/product_name").unwrap_or_default();
                if product.to_lowercase().contains("kvm") {
                    "kvm".to_string()
                } else if product.to_lowercase().contains("qemu") {
                    "qemu".to_string()
                } else if product.to_lowercase().contains("virtualbox") {
                    "virtualbox".to_string()
                } else if product.to_lowercase().contains("vmware") {
                    "vmware".to_string()
                } else if fs::metadata("/.dockerenv").is_ok() {
                    "docker".to_string()
                } else {
                    "bare-metal".to_string()
                }
            });

        MetricValue {
            name: "virtualization".to_string(),
            value: virt,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_uptime(&self) -> MetricValue {
        let uptime_secs = fs::read_to_string("/proc/uptime")
            .ok()
            .and_then(|s| s.split_whitespace().next().and_then(|t| t.parse::<f64>().ok()))
            .unwrap_or(0.0);

        let days = (uptime_secs / 86400.0) as u64;
        let hours = ((uptime_secs % 86400.0) / 3600.0) as u64;
        let minutes = ((uptime_secs % 3600.0) / 60.0) as u64;

        let uptime_str = if days > 0 {
            format!("{} days, {} hours, {} minutes", days, hours, minutes)
        } else if hours > 0 {
            format!("{} hours, {} minutes", hours, minutes)
        } else {
            format!("{} minutes", minutes)
        };

        MetricValue {
            name: "uptime".to_string(),
            value: uptime_str,
            unit: "seconds".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Good,
        }
    }

    fn get_last_boot(&self) -> MetricValue {
        let output = Command::new("who")
            .arg("-b")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok());

        let boot_time = output
            .and_then(|s| s.split_whitespace().skip(2).collect::<Vec<_>>().join(" "))
            .unwrap_or_else(|| "unknown".to_string());

        MetricValue {
            name: "last_boot".to_string(),
            value: boot_time,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_load_average(&self) -> MetricValue {
        let load = fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|s| {
                let parts: Vec<&str> = s.split_whitespace().collect();
                if parts.len() >= 3 {
                    Some(format!("{} {} {}", parts[0], parts[1], parts[2]))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "0.00 0.00 0.00".to_string());

        let values: Vec<f64> = load.split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();

        let severity = if values.get(0).map_or(false, |&v| v > 5.0) {
            MetricSeverity::Critical
        } else if values.get(0).map_or(false, |&v| v > 2.0) {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "load_average".to_string(),
            value: load,
            unit: "1m 5m 15m".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_active_users(&self) -> MetricValue {
        let count = Command::new("who")
            .arg("-q")
            .output()
            .ok()
            .and_then(|o| {
                let out = String::from_utf8(o.stdout).ok()?;
                let lines: Vec<&str> = out.lines().collect();
                if lines.len() >= 2 {
                    lines[1].split('=').nth(1).map(|s| s.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "0".to_string());

        let users_list = Command::new("who")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let value = format!("{} logged in users\n{}", count, users_list);

        MetricValue {
            name: "active_users".to_string(),
            value,
            unit: "users".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_system_time(&self) -> MetricValue {
        let now = Local::now();
        let time_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

        MetricValue {
            name: "system_time".to_string(),
            value: time_str,
            unit: "UTC".to_string(),
            timestamp: now,
            severity: MetricSeverity::Info,
        }
    }

    fn get_timezone(&self) -> MetricValue {
        let tz = Command::new("timedatectl")
            .arg("show")
            .arg("--property=Timezone")
            .output()
            .ok()
            .and_then(|o| {
                let out = String::from_utf8(o.stdout).ok()?;
                out.split('=').nth(1).map(|s| s.trim().to_string())
            })
            .unwrap_or_else(|| "UTC".to_string());

        MetricValue {
            name: "timezone".to_string(),
            value: tz,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_environment(&self) -> MetricValue {
        let env_vars = vec!["ENV", "ENVIRONMENT", "NODE_ENV", "APP_ENV", "RAILS_ENV"];
        let mut detected = "production".to_string();

        for var in env_vars {
            if let Ok(value) = std::env::var(var) {
                detected = value.to_lowercase();
                break;
            }
        }

        let hostname = self.get_hostname().value;
        if hostname.contains("dev") || hostname.contains("development") {
            detected = "development".to_string();
        } else if hostname.contains("staging") {
            detected = "staging".to_string();
        } else if hostname.contains("prod") {
            detected = "production".to_string();
        }

        MetricValue {
            name: "environment".to_string(),
            value: detected,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_system_type(&self) -> MetricValue {
        let is_container = fs::metadata("/.dockerenv").is_ok()
            || fs::metadata("/run/.containerenv").is_ok();
        let is_wsl = fs::read_to_string("/proc/version")
            .unwrap_or_default()
            .to_lowercase()
            .contains("microsoft");

        let sys_type = if is_wsl {
            "WSL (Windows Subsystem for Linux)"
        } else if is_container {
            "Container (Docker/Podman)"
        } else {
            "Bare-metal/VM"
        };

        MetricValue {
            name: "system_type".to_string(),
            value: sys_type.to_string(),
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_product_name(&self) -> MetricValue {
        let product = fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
            .unwrap_or_default()
            .trim()
            .to_string();

        MetricValue {
            name: "product_name".to_string(),
            value: if product.is_empty() { "Unknown".to_string() } else { product },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_serial_number(&self) -> Option<String> {
        fs::read_to_string("/sys/devices/virtual/dmi/id/product_serial")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != "None" && s != "To be filled by O.E.M.")
    }
}