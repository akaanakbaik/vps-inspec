use super::{MetricGroup, MetricValue, MetricSeverity};
use chrono::Local;
use std::fs;
use std::process::Command;
use regex::Regex;

pub struct LogsCollector {
    start_time: std::time::Instant,
}

impl LogsCollector {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    pub async fn collect(&self) -> MetricGroup {
        let start = std::time::Instant::now();
        let mut metrics = Vec::new();

        metrics.push(self.get_syslog_errors());
        metrics.push(self.get_auth_log_errors());
        metrics.push(self.get_kernel_errors());
        metrics.push(self.get_nginx_errors());
        metrics.push(self.get_apache_errors());
        metrics.push(self.get_mysql_errors());
        metrics.push(self.get_php_errors());
        metrics.push(self.get_oom_events());
        metrics.push(self.get_hardware_errors());
        metrics.push(self.get_failed_cron_jobs());
        metrics.push(self.get_service_crash_logs());
        metrics.push(self.get_firewall_logs());
        metrics.push(self.get_log_rotation_status());

        let duration_ms = start.elapsed().as_millis() as u64;

        MetricGroup {
            category: "System Logs Analysis".to_string(),
            metrics,
            collected_at: Local::now(),
            duration_ms,
        }
    }

    fn get_syslog_errors(&self) -> MetricValue {
        let syslog_paths = vec!["/var/log/syslog", "/var/log/messages"];
        let mut errors = Vec::new();

        for path in syslog_paths {
            if let Ok(content) = fs::read_to_string(path) {
                let last_lines: Vec<&str> = content.lines()
                    .rev()
                    .take(200)
                    .filter(|l| {
                        l.to_lowercase().contains("error") ||
                        l.to_lowercase().contains("fail") ||
                        l.to_lowercase().contains("critical") ||
                        l.to_lowercase().contains("panic")
                    })
                    .take(10)
                    .collect();

                for line in last_lines {
                    errors.push(line.to_string());
                }
            }
        }

        MetricValue {
            name: "syslog_recent_errors".to_string(),
            value: if errors.is_empty() { "No recent syslog errors".to_string() } else { errors.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: if errors.is_empty() { MetricSeverity::Good } else { MetricSeverity::Warning },
        }
    }

    fn get_auth_log_errors(&self) -> MetricValue {
        let auth_paths = vec!["/var/log/auth.log", "/var/log/secure"];
        let mut auth_events = Vec::new();

        for path in auth_paths {
            if let Ok(content) = fs::read_to_string(path) {
                let failed_logins: Vec<&str> = content.lines()
                    .rev()
                    .take(500)
                    .filter(|l| {
                        l.contains("Failed password") ||
                        l.contains("authentication failure") ||
                        l.contains("Invalid user") ||
                        l.contains("pam_unix")
                    })
                    .take(15)
                    .collect();

                for line in failed_logins {
                    auth_events.push(line.to_string());
                }
            }
        }

        let severity = if auth_events.len() > 50 {
            MetricSeverity::Critical
        } else if auth_events.len() > 20 {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Info
        };

        MetricValue {
            name: "authentication_events".to_string(),
            value: if auth_events.is_empty() { "No recent authentication events".to_string() } else { auth_events.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_kernel_errors(&self) -> MetricValue {
        let dmesg = Command::new("dmesg")
            .arg("-T")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let kernel_errors: Vec<String> = dmesg.lines()
            .rev()
            .take(500)
            .filter(|l| {
                l.to_lowercase().contains("error") ||
                l.to_lowercase().contains("fail") ||
                l.to_lowercase().contains("bug") ||
                l.to_lowercase().contains("oops") ||
                l.to_lowercase().contains("segfault") ||
                l.to_lowercase().contains("call trace")
            })
            .take(15)
            .map(|l| l.to_string())
            .collect();

        let severity = if kernel_errors.is_empty() {
            MetricSeverity::Good
        } else if kernel_errors.iter().any(|e| e.contains("Oops") || e.contains("BUG")) {
            MetricSeverity::Critical
        } else {
            MetricSeverity::Warning
        };

        MetricValue {
            name: "kernel_errors".to_string(),
            value: if kernel_errors.is_empty() { "No kernel errors detected".to_string() } else { kernel_errors.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_nginx_errors(&self) -> MetricValue {
        let error_logs = vec!["/var/log/nginx/error.log", "/var/log/nginx/error.log.1"];
        let mut errors = Vec::new();

        for path in error_logs {
            if let Ok(content) = fs::read_to_string(path) {
                let recent_errors: Vec<&str> = content.lines()
                    .rev()
                    .take(300)
                    .filter(|l| {
                        l.contains("error") ||
                        l.contains("crit") ||
                        l.contains("emerg") ||
                        l.contains("alert")
                    })
                    .take(10)
                    .collect();

                for line in recent_errors {
                    errors.push(line.to_string());
                }
            }
        }

        MetricValue {
            name: "nginx_error_logs".to_string(),
            value: if errors.is_empty() { "No Nginx errors found".to_string() } else { errors.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: if errors.is_empty() { MetricSeverity::Good } else { MetricSeverity::Warning },
        }
    }

    fn get_apache_errors(&self) -> MetricValue {
        let error_logs = vec!["/var/log/apache2/error.log", "/var/log/httpd/error_log"];
        let mut errors = Vec::new();

        for path in error_logs {
            if let Ok(content) = fs::read_to_string(path) {
                let recent_errors: Vec<&str> = content.lines()
                    .rev()
                    .take(300)
                    .filter(|l| {
                        l.contains("error") ||
                        l.contains("crit") ||
                        l.contains("emerg") ||
                        l.contains("alert")
                    })
                    .take(10)
                    .collect();

                for line in recent_errors {
                    errors.push(line.to_string());
                }
            }
        }

        MetricValue {
            name: "apache_error_logs".to_string(),
            value: if errors.is_empty() { "No Apache errors found".to_string() } else { errors.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: if errors.is_empty() { MetricSeverity::Good } else { MetricSeverity::Warning },
        }
    }

    fn get_mysql_errors(&self) -> MetricValue {
        let error_logs = vec![
            "/var/log/mysql/error.log",
            "/var/log/mariadb/error.log",
            "/var/log/mysqld.log",
        ];
        let mut errors = Vec::new();

        for path in error_logs {
            if let Ok(content) = fs::read_to_string(path) {
                let recent_errors: Vec<&str> = content.lines()
                    .rev()
                    .take(300)
                    .filter(|l| {
                        l.to_lowercase().contains("error") ||
                        l.to_lowercase().contains("fatal") ||
                        l.to_lowercase().contains("crash")
                    })
                    .take(10)
                    .collect();

                for line in recent_errors {
                    errors.push(line.to_string());
                }
            }
        }

        MetricValue {
            name: "mysql_mariadb_errors".to_string(),
            value: if errors.is_empty() { "No MySQL/MariaDB errors found".to_string() } else { errors.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: if errors.is_empty() { MetricSeverity::Good } else { MetricSeverity::Critical },
        }
    }

    fn get_php_errors(&self) -> MetricValue {
        let php_error_logs = vec![
            "/var/log/php_errors.log",
            "/var/log/php-fpm.log",
            "/var/log/php7.4-fpm.log",
            "/var/log/php8.1-fpm.log",
            "/var/log/php8.2-fpm.log",
            "/var/log/php8.3-fpm.log",
        ];
        let mut errors = Vec::new();

        for path in php_error_logs {
            if let Ok(content) = fs::read_to_string(path) {
                let recent_errors: Vec<&str> = content.lines()
                    .rev()
                    .take(200)
                    .filter(|l| {
                        l.contains("error") ||
                        l.contains("fatal") ||
                        l.contains("warning") ||
                        l.contains("parse error")
                    })
                    .take(10)
                    .collect();

                for line in recent_errors {
                    errors.push(line.to_string());
                }
            }
        }

        MetricValue {
            name: "php_errors".to_string(),
            value: if errors.is_empty() { "No PHP errors found".to_string() } else { errors.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: if errors.is_empty() { MetricSeverity::Good } else { MetricSeverity::Warning },
        }
    }

    fn get_oom_events(&self) -> MetricValue {
        let dmesg = Command::new("dmesg")
            .arg("-T")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let syslog = fs::read_to_string("/var/log/syslog").unwrap_or_default();
        let combined = format!("{}\n{}", dmesg, syslog);

        let oom_events: Vec<String> = combined.lines()
            .filter(|l| {
                l.to_lowercase().contains("oom") ||
                l.to_lowercase().contains("out of memory") ||
                l.to_lowercase().contains("kill process") ||
                l.contains("Killed process")
            })
            .map(|l| l.to_string())
            .collect();

        let severity = if !oom_events.is_empty() {
            MetricSeverity::Critical
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "oom_killer_events".to_string(),
            value: if oom_events.is_empty() { "No OOM killer events detected".to_string() } else {
                let last_event = oom_events.last().unwrap();
                format!("Last OOM event: {}", last_event)
            },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_hardware_errors(&self) -> MetricValue {
        let dmesg = Command::new("dmesg")
            .arg("-T")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let hardware_errors: Vec<String> = dmesg.lines()
            .filter(|l| {
                l.to_lowercase().contains("hardware error") ||
                l.to_lowercase().contains("mce") ||
                l.to_lowercase().contains("machine check") ||
                l.contains("EDAC") ||
                l.to_lowercase().contains("corrected error") ||
                l.to_lowercase().contains("uncorrected error")
            })
            .take(10)
            .map(|l| l.to_string())
            .collect();

        let severity = if !hardware_errors.is_empty() {
            MetricSeverity::Critical
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "hardware_errors".to_string(),
            value: if hardware_errors.is_empty() { "No hardware errors detected".to_string() } else { hardware_errors.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_failed_cron_jobs(&self) -> MetricValue {
        let cron_logs = vec!["/var/log/syslog", "/var/log/cron"];
        let mut failed_jobs = Vec::new();

        for path in cron_logs {
            if let Ok(content) = fs::read_to_string(path) {
                let cron_failures: Vec<&str> = content.lines()
                    .rev()
                    .take(1000)
                    .filter(|l| {
                        l.contains("CRON") && (
                            l.contains("FAILED") ||
                            l.contains("error") ||
                            l.contains("permission denied") ||
                            l.contains("not found")
                        )
                    })
                    .take(10)
                    .collect();

                for line in cron_failures {
                    failed_jobs.push(line.to_string());
                }
            }
        }

        MetricValue {
            name: "failed_cron_jobs".to_string(),
            value: if failed_jobs.is_empty() { "No failed cron jobs detected".to_string() } else { failed_jobs.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: if !failed_jobs.is_empty() { MetricSeverity::Warning } else { MetricSeverity::Good },
        }
    }

    fn get_service_crash_logs(&self) -> MetricValue {
        let journal = Command::new("journalctl")
            .args(["-p", "3", "-b", "-n", "20", "--no-pager"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let crashes: Vec<String> = journal.lines()
            .filter(|l| {
                l.contains("crashed") ||
                l.contains("failed to start") ||
                l.contains("exit code") ||
                l.contains("segfault") ||
                l.contains("core dumped")
            })
            .take(15)
            .map(|l| l.to_string())
            .collect();

        let severity = if !crashes.is_empty() {
            MetricSeverity::Critical
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "service_crash_logs".to_string(),
            value: if crashes.is_empty() { "No service crashes detected".to_string() } else { crashes.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_firewall_logs(&self) -> MetricValue {
        let firewall_logs = vec!["/var/log/ufw.log", "/var/log/firewalld", "/var/log/kern.log"];
        let mut blocked = Vec::new();

        for path in firewall_logs {
            if let Ok(content) = fs::read_to_string(path) {
                let blocked_attempts: Vec<&str> = content.lines()
                    .rev()
                    .take(500)
                    .filter(|l| {
                        l.contains("BLOCK") ||
                        l.contains("DROP") ||
                        l.contains("REJECT") ||
                        (l.contains("DPT=") && l.contains("SRC="))
                    })
                    .take(10)
                    .collect();

                for line in blocked_attempts {
                    blocked.push(line.to_string());
                }
            }
        }

        let severity = if blocked.len() > 100 {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Info
        };

        MetricValue {
            name: "firewall_blocked_attempts".to_string(),
            value: if blocked.is_empty() { "No recent blocked attempts".to_string() } else { blocked.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_log_rotation_status(&self) -> MetricValue {
        let log_dir = "/var/log/";
        let mut oversized_logs = Vec::new();

        if let Ok(entries) = fs::read_dir(log_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
                        if size_mb > 500.0 {
                            if let Ok(name) = entry.file_name().into_string() {
                                oversized_logs.push(format!("{}: {:.1} MB", name, size_mb));
                            }
                        }
                    }
                }
            }
        }

        let severity = if !oversized_logs.is_empty() {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "oversized_log_files".to_string(),
            value: if oversized_logs.is_empty() { "No oversized log files (>500MB)".to_string() } else { oversized_logs.join("\n") },
            unit: "bytes".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }
}