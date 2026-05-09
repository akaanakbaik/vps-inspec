use super::{MetricGroup, MetricValue, MetricSeverity};
use chrono::Local;
use std::process::Command;
use std::fs;
use regex::Regex;

pub struct SecurityCollector {
    start_time: std::time::Instant,
}

impl SecurityCollector {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    pub async fn collect(&self) -> MetricGroup {
        let start = std::time::Instant::now();
        let mut metrics = Vec::new();

        metrics.push(self.get_ssh_port());
        metrics.push(self.get_ssh_root_login());
        metrics.push(self.get_ssh_password_auth());
        metrics.push(self.get_ssh_protocol());
        metrics.push(self.get_failed_logins_24h());
        metrics.push(self.get_users_with_sudo());
        metrics.push(self.get_uid0_users());
        metrics.push(self.get_last_logins());
        metrics.push(self.get_running_services());
        metrics.push(self.get_failed_services());
        metrics.push(self.get_security_updates());
        metrics.push(self.get_last_update_time());
        metrics.push(self.get_fail2ban_status());
        metrics.push(self.get_selinux_status());
        metrics.push(self.get_apparmor_status());
        metrics.push(self.get_cron_jobs());
        metrics.push(self.get_suspicious_processes());
        metrics.push(self.get_open_ports_risk());
        metrics.push(self.get_password_policy());
        metrics.push(self.get_umask_settings());

        let duration_ms = start.elapsed().as_millis() as u64;

        MetricGroup {
            category: "Security Audit".to_string(),
            metrics,
            collected_at: Local::now(),
            duration_ms,
        }
    }

    fn get_ssh_port(&self) -> MetricValue {
        let sshd_config = fs::read_to_string("/etc/ssh/sshd_config").unwrap_or_default();
        let port_re = Regex::new(r"^Port\s+(\d+)").unwrap();
        
        let port = port_re.captures(&sshd_config)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "22".to_string());

        let severity = if port == "22" {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "ssh_port".to_string(),
            value: port,
            unit: "port".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_ssh_root_login(&self) -> MetricValue {
        let sshd_config = fs::read_to_string("/etc/ssh/sshd_config").unwrap_or_default();
        let re = Regex::new(r"^PermitRootLogin\s+(\w+)").unwrap();
        
        let root_login = re.captures(&sshd_config)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str())
            .unwrap_or("yes");

        let severity = if root_login == "yes" || root_login == "prohibit-password" {
            MetricSeverity::Critical
        } else if root_login == "without-password" {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "ssh_root_login".to_string(),
            value: root_login.to_string(),
            unit: "".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_ssh_password_auth(&self) -> MetricValue {
        let sshd_config = fs::read_to_string("/etc/ssh/sshd_config").unwrap_or_default();
        let re = Regex::new(r"^PasswordAuthentication\s+(\w+)").unwrap();
        
        let password_auth = re.captures(&sshd_config)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str())
            .unwrap_or("yes");

        let severity = if password_auth == "yes" {
            MetricSeverity::Critical
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "ssh_password_authentication".to_string(),
            value: password_auth.to_string(),
            unit: "".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_ssh_protocol(&self) -> MetricValue {
        let sshd_config = fs::read_to_string("/etc/ssh/sshd_config").unwrap_or_default();
        let re = Regex::new(r"^Protocol\s+(\d+)").unwrap();
        
        let protocol = re.captures(&sshd_config)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str())
            .unwrap_or("2");

        let severity = if protocol != "2" {
            MetricSeverity::Critical
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "ssh_protocol".to_string(),
            value: protocol.to_string(),
            unit: "version".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_failed_logins_24h(&self) -> MetricValue {
        let auth_logs = vec![
            "/var/log/auth.log",
            "/var/log/secure",
            "/var/log/messages",
        ];

        let mut total_failed = 0;
        
        for log_path in auth_logs {
            if let Ok(content) = fs::read_to_string(log_path) {
                let failed_count = content.lines()
                    .filter(|l| {
                        l.contains("Failed password") || 
                        l.contains("authentication failure") ||
                        l.contains("Invalid user")
                    })
                    .count();
                total_failed += failed_count;
            }
        }

        let severity = if total_failed > 100 {
            MetricSeverity::Critical
        } else if total_failed > 20 {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "failed_logins_24h".to_string(),
            value: total_failed.to_string(),
            unit: "attempts".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_users_with_sudo(&self) -> MetricValue {
        let sudoers = fs::read_to_string("/etc/sudoers").unwrap_or_default();
        let sudo_group = Command::new("getent")
            .arg("group")
            .arg("sudo")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let mut sudo_users = Vec::new();
        
        if sudo_group.contains(":") {
            let parts: Vec<&str> = sudo_group.split(':').collect();
            if parts.len() >= 4 {
                let users = parts[3].trim();
                if !users.is_empty() {
                    sudo_users.extend(users.split(',').map(|u| u.to_string()));
                }
            }
        }

        let wheel_group = Command::new("getent")
            .arg("group")
            .arg("wheel")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();
        
        if wheel_group.contains(":") {
            let parts: Vec<&str> = wheel_group.split(':').collect();
            if parts.len() >= 4 {
                let users = parts[3].trim();
                if !users.is_empty() {
                    sudo_users.extend(users.split(',').map(|u| u.to_string()));
                }
            }
        }

        sudo_users.sort();
        sudo_users.dedup();

        MetricValue {
            name: "sudo_users".to_string(),
            value: if sudo_users.is_empty() { "No sudo users found".to_string() } else { sudo_users.join(", ") },
            unit: "users".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_uid0_users(&self) -> MetricValue {
        let passwd = fs::read_to_string("/etc/passwd").unwrap_or_default();
        
        let uid0_users: Vec<String> = passwd.lines()
            .filter(|l| {
                let parts: Vec<&str> = l.split(':').collect();
                parts.len() >= 3 && parts[2] == "0"
            })
            .map(|l| {
                let parts: Vec<&str> = l.split(':').collect();
                parts[0].to_string()
            })
            .collect();

        let severity = if uid0_users.len() > 1 {
            MetricSeverity::Critical
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "uid0_users".to_string(),
            value: if uid0_users.is_empty() { "none".to_string() } else { uid0_users.join(", ") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_last_logins(&self) -> MetricValue {
        let lastlog = Command::new("lastlog")
            .args(["-u", "1000-60000"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let logins: Vec<String> = lastlog.lines()
            .filter(|l| !l.contains("Never logged in"))
            .take(20)
            .map(|l| l.to_string())
            .collect();

        MetricValue {
            name: "recent_user_logins".to_string(),
            value: if logins.is_empty() { "No recent logins found".to_string() } else { logins.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_running_services(&self) -> MetricValue {
        let services = Command::new("systemctl")
            .args(["list-units", "--type=service", "--state=running", "--no-pager"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let service_list: Vec<String> = services.lines()
            .skip(1)
            .filter(|l| l.contains(".service"))
            .map(|l| {
                let parts: Vec<&str> = l.split_whitespace().collect();
                parts[0].to_string()
            })
            .collect();

        MetricValue {
            name: "running_services".to_string(),
            value: format!("{} services running\n{}", service_list.len(), service_list.join("\n")),
            unit: "services".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_failed_services(&self) -> MetricValue {
        let failed = Command::new("systemctl")
            .args(["--failed", "--no-pager"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let failed_list: Vec<String> = failed.lines()
            .skip(1)
            .filter(|l| l.contains(".service"))
            .map(|l| {
                let parts: Vec<&str> = l.split_whitespace().collect();
                parts[0].to_string()
            })
            .collect();

        let severity = if !failed_list.is_empty() {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "failed_services".to_string(),
            value: if failed_list.is_empty() { "No failed services".to_string() } else { failed_list.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_security_updates(&self) -> MetricValue {
        let updates = Command::new("sh")
            .arg("-c")
            .arg("apt list --upgradable 2>/dev/null | grep -i security | wc -l")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0);

        let severity = if updates > 5 {
            MetricSeverity::Critical
        } else if updates > 0 {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "pending_security_updates".to_string(),
            value: updates.to_string(),
            unit: "packages".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_last_update_time(&self) -> MetricValue {
        let apt_log = fs::read_to_string("/var/log/apt/history.log").unwrap_or_default();
        
        let last_update = apt_log.lines()
            .find(|l| l.starts_with("Start-Date:"))
            .map(|l| l.replace("Start-Date: ", ""))
            .unwrap_or_else(|| "Never".to_string());

        MetricValue {
            name: "last_system_update".to_string(),
            value: last_update,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_fail2ban_status(&self) -> MetricValue {
        let fail2ban = Command::new("fail2ban-client")
            .arg("status")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let is_running = fail2ban.contains("Number of jail");
        let jails = if is_running {
            let re = Regex::new(r"Jail list:\s+(.+)").unwrap();
            re.captures(&fail2ban)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str())
                .unwrap_or("none")
        } else {
            "not running"
        };

        let severity = if is_running && jails != "none" {
            MetricSeverity::Good
        } else if is_running {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Warning
        };

        MetricValue {
            name: "fail2ban_status".to_string(),
            value: format!("Running: {}, Jails: {}", if is_running { "yes" } else { "no" }, jails),
            unit: "".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_selinux_status(&self) -> MetricValue {
        let selinux = Command::new("getenforce")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_else(|_| "Disabled".to_string())
            .trim()
            .to_string();

        let severity = if selinux == "Enforcing" {
            MetricSeverity::Good
        } else if selinux == "Permissive" {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Warning
        };

        MetricValue {
            name: "selinux_status".to_string(),
            value: selinux,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_apparmor_status(&self) -> MetricValue {
        let apparmor = Command::new("aa-status")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let profiles_loaded = apparmor.lines()
            .find(|l| l.contains("profiles are loaded"))
            .map(|l| l.to_string())
            .unwrap_or_else(|| "AppArmor not installed".to_string());

        MetricValue {
            name: "apparmor_status".to_string(),
            value: profiles_loaded,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_cron_jobs(&self) -> MetricValue {
        let cron_dirs = vec!["/etc/crontab", "/etc/cron.d/", "/etc/cron.daily/", "/etc/cron.hourly/", "/etc/cron.weekly/", "/etc/cron.monthly/"];
        let mut jobs = Vec::new();

        for dir in cron_dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            jobs.push(entry.path().display().to_string());
                        }
                    }
                }
            }
        }

        MetricValue {
            name: "cron_jobs".to_string(),
            value: format!("{} cron jobs configured\n{}", jobs.len(), jobs.join("\n")),
            unit: "jobs".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_suspicious_processes(&self) -> MetricValue {
        let ps_output = Command::new("ps")
            .args(["aux"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let suspicious_keywords = vec!["nc -e", "ncat -e", "socat", "reverse", "meterpreter", "msf", "beacon"];
        let mut suspicious = Vec::new();

        for line in ps_output.lines() {
            for keyword in &suspicious_keywords {
                if line.to_lowercase().contains(keyword) {
                    suspicious.push(line.to_string());
                    break;
                }
            }
        }

        let severity = if !suspicious.is_empty() {
            MetricSeverity::Critical
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "suspicious_processes".to_string(),
            value: if suspicious.is_empty() { "No suspicious processes detected".to_string() } else { suspicious.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_open_ports_risk(&self) -> MetricValue {
        let risky_ports = vec![
            (21, "FTP - unencrypted"),
            (23, "Telnet - unencrypted"),
            (3389, "RDP - potential brute-force"),
            (5900, "VNC - weak auth potential"),
            (11211, "Memcached - DDoS risk"),
            (27017, "MongoDB - default port"),
            (6379, "Redis - unauthenticated risk"),
            (9200, "Elasticsearch - no auth risk"),
        ];

        let ss_output = Command::new("ss")
            .args(["-tln"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let mut found_risks = Vec::new();

        for (port, description) in risky_ports {
            if ss_output.contains(&format!(":{} ", port)) {
                found_risks.push(format!("Port {}: {}", port, description));
            }
        }

        let severity = if found_risks.len() > 2 {
            MetricSeverity::Critical
        } else if !found_risks.is_empty() {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "risky_open_ports".to_string(),
            value: if found_risks.is_empty() { "No high-risk ports detected".to_string() } else { found_risks.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_password_policy(&self) -> MetricValue {
        let login_defs = fs::read_to_string("/etc/login.defs").unwrap_or_default();
        
        let pass_max_days = login_defs.lines()
            .find(|l| l.starts_with("PASS_MAX_DAYS"))
            .and_then(|l| l.split_whitespace().nth(1))
            .unwrap_or("99999");

        let pass_min_days = login_defs.lines()
            .find(|l| l.starts_with("PASS_MIN_DAYS"))
            .and_then(|l| l.split_whitespace().nth(1))
            .unwrap_or("0");

        let pass_warn_age = login_defs.lines()
            .find(|l| l.starts_with("PASS_WARN_AGE"))
            .and_then(|l| l.split_whitespace().nth(1))
            .unwrap_or("7");

        MetricValue {
            name: "password_policy".to_string(),
            value: format!("Max days: {} | Min days: {} | Warn age: {}", pass_max_days, pass_min_days, pass_warn_age),
            unit: "days".to_string(),
            timestamp: Local::now(),
            severity: if pass_max_days.parse::<u32>().unwrap_or(0) > 90 { MetricSeverity::Warning } else { MetricSeverity::Good },
        }
    }

    fn get_umask_settings(&self) -> MetricValue {
        let profile_umask = Command::new("sh")
            .arg("-c")
            .arg("grep -h '^umask' /etc/profile /etc/bashrc ~/.bashrc 2>/dev/null | head -1")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let current_umask = Command::new("sh")
            .arg("-c")
            .arg("umask")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        MetricValue {
            name: "umask_settings".to_string(),
            value: format!("Configured: {}\nCurrent: {}", 
                if profile_umask.is_empty() { "default (022)" } else { profile_umask.trim() },
                current_umask.trim()),
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: if current_umask.trim() == "000" { MetricSeverity::Critical } else { MetricSeverity::Info },
        }
    }
}