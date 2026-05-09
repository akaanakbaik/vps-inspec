use super::{MetricGroup, MetricValue, MetricSeverity};
use chrono::Local;
use std::process::Command;
use std::fs;

pub struct StorageCollector {
    start_time: std::time::Instant,
}

impl StorageCollector {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    pub async fn collect(&self) -> MetricGroup {
        let start = std::time::Instant::now();
        let mut metrics = Vec::new();

        metrics.push(self.get_disk_usage());
        metrics.push(self.get_inode_usage());
        metrics.push(self.get_mount_points());
        metrics.push(self.get_disk_types());
        metrics.push(self.get_disk_io_stats());
        metrics.push(self.get_large_files());
        metrics.push(self.get_filesystem_types());
        metrics.push(self.get_disk_schedulers());

        let duration_ms = start.elapsed().as_millis() as u64;

        MetricGroup {
            category: "Storage Analysis".to_string(),
            metrics,
            collected_at: Local::now(),
            duration_ms,
        }
    }

    fn get_disk_usage(&self) -> MetricValue {
        let output = Command::new("df")
            .arg("-BG")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let mut usage_lines = Vec::new();
        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let filesystem = parts[0];
                let size = parts[1].replace("G", "");
                let used = parts[2].replace("G", "");
                let avail = parts[3].replace("G", "");
                let use_percent = parts[4].replace("%", "");
                let mount = parts[5];

                if let (Ok(size_gb), Ok(used_gb), Ok(avail_gb), Ok(use_pct)) = (
                    size.parse::<u64>(),
                    used.parse::<u64>(),
                    avail.parse::<u64>(),
                    use_percent.parse::<u64>(),
                ) {
                    usage_lines.push(format!(
                        "{} @ {}: {}GB/{}GB used ({}%)",
                        filesystem, mount, used_gb, size_gb, use_pct
                    ));
                }
            }
        }

        let severity = if usage_lines.iter().any(|l| l.contains("95%")) {
            MetricSeverity::Critical
        } else if usage_lines.iter().any(|l| l.contains("90%")) {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "disk_usage".to_string(),
            value: usage_lines.join("\n"),
            unit: "GB".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_inode_usage(&self) -> MetricValue {
        let output = Command::new("df")
            .arg("-i")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let mut inode_lines = Vec::new();
        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let filesystem = parts[0];
                let iused = parts[2];
                let use_percent = parts[4].replace("%", "");
                let mount = parts[5];

                inode_lines.push(format!("{} @ {}: {}% inodes used", filesystem, mount, use_percent));
            }
        }

        let severity = if inode_lines.iter().any(|l| l.contains("95%")) {
            MetricSeverity::Critical
        } else if inode_lines.iter().any(|l| l.contains("85%")) {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "inode_usage".to_string(),
            value: inode_lines.join("\n"),
            unit: "inodes".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_mount_points(&self) -> MetricValue {
        let output = Command::new("mount")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let mounts: Vec<String> = output.lines()
            .map(|l| l.to_string())
            .collect();

        MetricValue {
            name: "mount_points".to_string(),
            value: mounts.join("\n"),
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_disk_types(&self) -> MetricValue {
        let lsblk = Command::new("lsblk")
            .args(["-d", "-o", "NAME,ROTA,TRAN", "-J"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let mut disk_info = Vec::new();

        for line in lsblk.lines() {
            if line.contains("\"name\"") {
                let name = line.split(':').nth(1).unwrap_or("\"\"").trim_matches('"');
                if !name.is_empty() && name != "name" {
                    disk_info.push(format!("Disk: {}", name));
                }
            }
            if line.contains("\"rota\"") {
                let rota = line.split(':').nth(1).unwrap_or("0").trim();
                let disk_type = if rota == "0" { "SSD" } else { "HDD" };
                disk_info.push(format!("Type: {}", disk_type));
            }
        }

        MetricValue {
            name: "disk_types".to_string(),
            value: if disk_info.is_empty() { "No disk information available".to_string() } else { disk_info.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_disk_io_stats(&self) -> MetricValue {
        let iostat = Command::new("iostat")
            .args(["-x", "1", "1"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let mut io_lines = Vec::new();
        for line in iostat.lines() {
            if line.contains("Device") || line.contains("tps") || (line.contains("sd") || line.contains("vd") || line.contains("nvme")) {
                io_lines.push(line.to_string());
            }
        }

        MetricValue {
            name: "disk_io_statistics".to_string(),
            value: io_lines.join("\n"),
            unit: "r/s w/s rMB/s wMB/s".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_large_files(&self) -> MetricValue {
        let find_cmd = Command::new("sh")
            .arg("-c")
            .arg("find / -type f -size +100M -exec ls -lh {} \\; 2>/dev/null | head -10")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_else(|| "No large files found (>100MB)".to_string());

        let severity = if find_cmd.lines().count() > 5 {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Info
        };

        MetricValue {
            name: "largest_files".to_string(),
            value: find_cmd,
            unit: "bytes".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_filesystem_types(&self) -> MetricValue {
        let df_output = Command::new("df")
            .arg("-T")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let mut fs_types = Vec::new();
        for line in df_output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let fstype = parts[1];
                if !fs_types.contains(&fstype.to_string()) {
                    fs_types.push(fstype.to_string());
                }
            }
        }

        MetricValue {
            name: "filesystem_types".to_string(),
            value: fs_types.join(", "),
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_disk_schedulers(&self) -> MetricValue {
        let schedulers = Command::new("cat")
            .arg("/sys/block/*/queue/scheduler")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        MetricValue {
            name: "io_schedulers".to_string(),
            value: schedulers,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }
}