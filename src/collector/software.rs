use super::{MetricGroup, MetricValue, MetricSeverity};
use chrono::Local;
use std::process::Command;

pub struct SoftwareCollector {
    start_time: std::time::Instant,
}

impl SoftwareCollector {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    pub async fn collect(&self) -> MetricGroup {
        let start = std::time::Instant::now();
        let mut metrics = Vec::new();

        metrics.push(self.get_nginx_version());
        metrics.push(self.get_apache_version());
        metrics.push(self.get_php_version());
        metrics.push(self.get_mysql_version());
        metrics.push(self.get_postgresql_version());
        metrics.push(self.get_nodejs_version());
        metrics.push(self.get_python_version());
        metrics.push(self.get_docker_version());
        metrics.push(self.get_redis_version());
        metrics.push(self.get_memcached_version());
        metrics.push(self.get_php_modules());
        metrics.push(self.get_python_packages());
        metrics.push(self.get_docker_containers());
        metrics.push(self.get_composer_packages());
        metrics.push(self.get_git_version());
        metrics.push(self.get_java_version());
        metrics.push(self.get_go_version());
        metrics.push(self.get_rust_version());
        metrics.push(self.get_ruby_version());

        let duration_ms = start.elapsed().as_millis() as u64;

        MetricGroup {
            category: "Software Inventory".to_string(),
            metrics,
            collected_at: Local::now(),
            duration_ms,
        }
    }

    fn get_nginx_version(&self) -> MetricValue {
        let version = Command::new("nginx")
            .arg("-v")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stderr).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "not installed".to_string());

        MetricValue {
            name: "nginx".to_string(),
            value: version,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_apache_version(&self) -> MetricValue {
        let version = Command::new("apache2")
            .arg("-v")
            .output()
            .ok()
            .or_else(|| Command::new("httpd").arg("-v").output().ok())
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.lines().next().unwrap_or("unknown").to_string())
            .unwrap_or_else(|| "not installed".to_string());

        MetricValue {
            name: "apache".to_string(),
            value: version,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_php_version(&self) -> MetricValue {
        let version = Command::new("php")
            .arg("-v")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.lines().next().unwrap_or("unknown").to_string())
            .unwrap_or_else(|| "not installed".to_string());

        MetricValue {
            name: "php".to_string(),
            value: version,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_mysql_version(&self) -> MetricValue {
        let version = Command::new("mysql")
            .args(["--version"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| {
                Command::new("mariadb")
                    .args(["--version"])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "not installed".to_string())
            });

        MetricValue {
            name: "mysql_mariadb".to_string(),
            value: version,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_postgresql_version(&self) -> MetricValue {
        let version = Command::new("psql")
            .args(["--version"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "not installed".to_string());

        MetricValue {
            name: "postgresql".to_string(),
            value: version,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_nodejs_version(&self) -> MetricValue {
        let version = Command::new("node")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "not installed".to_string());

        let npm_version = Command::new("npm")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| format!(", npm: {}", s.trim()))
            .unwrap_or_default();

        MetricValue {
            name: "nodejs".to_string(),
            value: format!("{}{}", version, npm_version),
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_python_version(&self) -> MetricValue {
        let version = Command::new("python3")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stderr).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "not installed".to_string());

        let pip_version = Command::new("pip3")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| {
                let parts: Vec<&str> = s.split_whitespace().collect();
                if parts.len() >= 2 {
                    format!(", pip: {}", parts[1])
                } else {
                    String::new()
                }
            })
            .unwrap_or_default();

        MetricValue {
            name: "python".to_string(),
            value: format!("{}{}", version, pip_version),
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_docker_version(&self) -> MetricValue {
        let version = Command::new("docker")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "not installed".to_string());

        MetricValue {
            name: "docker".to_string(),
            value: version,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_redis_version(&self) -> MetricValue {
        let version = Command::new("redis-server")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stderr).ok())
            .map(|s| s.lines().next().unwrap_or("unknown").to_string())
            .unwrap_or_else(|| "not installed".to_string());

        MetricValue {
            name: "redis".to_string(),
            value: version,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_memcached_version(&self) -> MetricValue {
        let version = Command::new("memcached")
            .arg("-h")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.lines().find(|l| l.contains("memcached")).map(|l| l.to_string()))
            .unwrap_or_else(|| "not installed".to_string());

        MetricValue {
            name: "memcached".to_string(),
            value: version,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_php_modules(&self) -> MetricValue {
        let modules = Command::new("php")
            .args(["-m"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let modules_list: Vec<String> = modules.lines()
            .filter(|l| !l.is_empty() && !l.starts_with('['))
            .take(20)
            .map(|l| l.to_string())
            .collect();

        MetricValue {
            name: "php_modules".to_string(),
            value: format!("{} modules (showing first 20):\n{}", modules_list.len(), modules_list.join("\n")),
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_python_packages(&self) -> MetricValue {
        let packages = Command::new("pip3")
            .args(["list", "--format=freeze"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let package_list: Vec<String> = packages.lines()
            .take(20)
            .map(|l| l.to_string())
            .collect();

        MetricValue {
            name: "python_packages".to_string(),
            value: format!("{} packages (showing first 20):\n{}", packages.lines().count(), package_list.join("\n")),
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_docker_containers(&self) -> MetricValue {
        let containers = Command::new("docker")
            .args(["ps", "-a", "--format", "table {{.Names}}\t{{.Status}}\t{{.Image}}"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let container_list: Vec<String> = containers.lines()
            .skip(1)
            .map(|l| l.to_string())
            .collect();

        MetricValue {
            name: "docker_containers".to_string(),
            value: if container_list.is_empty() { "No containers found".to_string() } else { container_list.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_composer_packages(&self) -> MetricValue {
        let packages = Command::new("composer")
            .args(["show", "--direct", "--format=json"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let package_count = packages.matches("\"name\"").count();

        MetricValue {
            name: "composer_packages".to_string(),
            value: if package_count > 0 { format!("{} packages installed", package_count) } else { "not installed or no packages".to_string() },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_git_version(&self) -> MetricValue {
        let version = Command::new("git")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "not installed".to_string());

        MetricValue {
            name: "git".to_string(),
            value: version,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_java_version(&self) -> MetricValue {
        let version = Command::new("java")
            .arg("-version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stderr).ok())
            .map(|s| s.lines().next().unwrap_or("unknown").to_string())
            .unwrap_or_else(|| "not installed".to_string());

        MetricValue {
            name: "java".to_string(),
            value: version,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_go_version(&self) -> MetricValue {
        let version = Command::new("go")
            .arg("version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "not installed".to_string());

        MetricValue {
            name: "golang".to_string(),
            value: version,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_rust_version(&self) -> MetricValue {
        let version = Command::new("rustc")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "not installed".to_string());

        MetricValue {
            name: "rust".to_string(),
            value: version,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_ruby_version(&self) -> MetricValue {
        let version = Command::new("ruby")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "not installed".to_string());

        MetricValue {
            name: "ruby".to_string(),
            value: version,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }
}