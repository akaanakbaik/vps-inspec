use super::{MetricGroup, MetricValue, MetricSeverity};
use chrono::Local;
use std::process::Command;
use std::fs;

pub struct HardwareCollector {
    start_time: std::time::Instant,
}

impl HardwareCollector {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    pub async fn collect(&self) -> MetricGroup {
        let start = std::time::Instant::now();
        let mut metrics = Vec::new();

        metrics.push(self.get_cpu_model());
        metrics.push(self.get_cpu_cores());
        metrics.push(self.get_cpu_threads());
        metrics.push(self.get_cpu_frequency());
        metrics.push(self.get_cpu_cache());
        metrics.push(self.get_cpu_usage());
        metrics.push(self.get_cpu_flags());
        metrics.push(self.get_ram_total());
        metrics.push(self.get_ram_used());
        metrics.push(self.get_ram_available());
        metrics.push(self.get_ram_usage_percent());
        metrics.push(self.get_swap_total());
        metrics.push(self.get_swap_used());
        metrics.push(self.get_swap_percent());
        metrics.push(self.get_swappiness());
        metrics.push(self.get_cpu_temperature());
        metrics.push(self.get_cpu_vendor());
        metrics.push(self.get_hypervisor_cpu_flags());

        let duration_ms = start.elapsed().as_millis() as u64;

        MetricGroup {
            category: "Hardware Analysis".to_string(),
            metrics,
            collected_at: Local::now(),
            duration_ms,
        }
    }

    fn get_cpu_model(&self) -> MetricValue {
        let model = fs::read_to_string("/proc/cpuinfo")
            .unwrap_or_default()
            .lines()
            .find(|line| line.starts_with("model name"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| {
                Command::new("lscpu")
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .and_then(|s| {
                        s.lines()
                            .find(|l| l.contains("Model name"))
                            .and_then(|l| l.split(':').nth(1).map(|v| v.trim().to_string()))
                    })
                    .unwrap_or_else(|| "Unknown".to_string())
            });

        MetricValue {
            name: "cpu_model".to_string(),
            value: model,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_cpu_cores(&self) -> MetricValue {
        let cores = Command::new("nproc")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or_else(|| num_cpus::get() as u32);

        let physical = fs::read_to_string("/proc/cpuinfo")
            .unwrap_or_default()
            .lines()
            .filter(|line| line.starts_with("physical id"))
            .collect::<Vec<_>>()
            .len();

        let physical_cores = if physical > 0 {
            physical
        } else {
            cores as usize
        };

        MetricValue {
            name: "cpu_cores".to_string(),
            value: format!("{} physical, {} logical", physical_cores, cores),
            unit: "cores".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Good,
        }
    }

    fn get_cpu_threads(&self) -> MetricValue {
        let threads = Command::new("lscpu")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| {
                s.lines()
                    .find(|l| l.contains("Thread(s) per core"))
                    .and_then(|l| l.split(':').nth(1).map(|v| v.trim().to_string()))
            })
            .unwrap_or_else(|| "1".to_string());

        MetricValue {
            name: "cpu_threads_per_core".to_string(),
            value: threads,
            unit: "threads/core".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_cpu_frequency(&self) -> MetricValue {
        let base_freq = fs::read_to_string("/proc/cpuinfo")
            .unwrap_or_default()
            .lines()
            .find(|line| line.contains("cpu MHz"))
            .and_then(|line| line.split(':').nth(1))
            .and_then(|s| s.trim().parse::<f64>().ok())
            .unwrap_or(0.0);

        let current_freq = Command::new("cat")
            .arg("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .map(|f| f as f64 / 1000.0)
            .unwrap_or(base_freq);

        MetricValue {
            name: "cpu_frequency".to_string(),
            value: format!("Base: {:.0} MHz, Current: {:.0} MHz", base_freq, current_freq),
            unit: "MHz".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_cpu_cache(&self) -> MetricValue {
        let l1d = fs::read_to_string("/sys/devices/system/cpu/cpu0/cache/index0/size").unwrap_or_default().trim().to_string();
        let l1i = fs::read_to_string("/sys/devices/system/cpu/cpu0/cache/index1/size").unwrap_or_default().trim().to_string();
        let l2 = fs::read_to_string("/sys/devices/system/cpu/cpu0/cache/index2/size").unwrap_or_default().trim().to_string();
        let l3 = fs::read_to_string("/sys/devices/system/cpu/cpu0/cache/index3/size").unwrap_or_default().trim().to_string();

        MetricValue {
            name: "cpu_cache".to_string(),
            value: format!("L1d: {}, L1i: {}, L2: {}, L3: {}", l1d, l1i, l2, l3),
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_cpu_usage(&self) -> MetricValue {
        let usage = Command::new("top")
            .args(["-bn1"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.lines().find(|l| l.contains("%Cpu(s)")).map(|l| l.to_string()))
            .and_then(|line| {
                let re = regex::Regex::new(r"id,\s*([\d\.]+)").ok()?;
                re.captures(&line)
                    .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            })
            .and_then(|m| m.parse::<f64>().ok())
            .map(|idle| 100.0 - idle)
            .unwrap_or(0.0);

        let severity = if usage > 90.0 {
            MetricSeverity::Critical
        } else if usage > 75.0 {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "cpu_usage_percent".to_string(),
            value: format!("{:.1}", usage),
            unit: "%".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_cpu_flags(&self) -> MetricValue {
        let flags = fs::read_to_string("/proc/cpuinfo")
            .unwrap_or_default()
            .lines()
            .find(|line| line.starts_with("flags"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let has_aes = flags.contains("aes");
        let has_vmx = flags.contains("vmx") || flags.contains("svm");
        let has_avx = flags.contains("avx") || flags.contains("avx2");

        let status = format!("AES-NI: {}, VMX/SVM: {}, AVX: {}", 
            if has_aes { "✓" } else { "✗" },
            if has_vmx { "✓" } else { "✗" },
            if has_avx { "✓" } else { "✗" });

        MetricValue {
            name: "cpu_flags".to_string(),
            value: status,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: if !has_aes { MetricSeverity::Warning } else { MetricSeverity::Info },
        }
    }

    fn get_ram_total(&self) -> MetricValue {
        let total = Command::new("free")
            .arg("-b")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| {
                let lines: Vec<&str> = s.lines().collect();
                if lines.len() >= 2 {
                    let parts: Vec<&str> = lines[1].split_whitespace().collect();
                    if parts.len() >= 2 {
                        parts[1].parse::<u64>().ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or(0);

        let total_gb = total as f64 / 1024.0 / 1024.0 / 1024.0;

        MetricValue {
            name: "ram_total".to_string(),
            value: format!("{:.2}", total_gb),
            unit: "GB".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Good,
        }
    }

    fn get_ram_used(&self) -> MetricValue {
        let used = Command::new("free")
            .arg("-b")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| {
                let lines: Vec<&str> = s.lines().collect();
                if lines.len() >= 2 {
                    let parts: Vec<&str> = lines[1].split_whitespace().collect();
                    if parts.len() >= 3 {
                        parts[2].parse::<u64>().ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or(0);

        let used_gb = used as f64 / 1024.0 / 1024.0 / 1024.0;

        MetricValue {
            name: "ram_used".to_string(),
            value: format!("{:.2}", used_gb),
            unit: "GB".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_ram_available(&self) -> MetricValue {
        let avail = Command::new("free")
            .arg("-b")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| {
                let lines: Vec<&str> = s.lines().collect();
                if lines.len() >= 2 {
                    let parts: Vec<&str> = lines[1].split_whitespace().collect();
                    if parts.len() >= 7 {
                        parts[6].parse::<u64>().ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or(0);

        let avail_gb = avail as f64 / 1024.0 / 1024.0 / 1024.0;

        MetricValue {
            name: "ram_available".to_string(),
            value: format!("{:.2}", avail_gb),
            unit: "GB".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Good,
        }
    }

    fn get_ram_usage_percent(&self) -> MetricValue {
        let total_bytes = Command::new("free")
            .arg("-b")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| {
                let lines: Vec<&str> = s.lines().collect();
                if lines.len() >= 2 {
                    let parts: Vec<&str> = lines[1].split_whitespace().collect();
                    if parts.len() >= 2 {
                        parts[1].parse::<u64>().ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or(1);

        let used_bytes = Command::new("free")
            .arg("-b")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| {
                let lines: Vec<&str> = s.lines().collect();
                if lines.len() >= 2 {
                    let parts: Vec<&str> = lines[1].split_whitespace().collect();
                    if parts.len() >= 3 {
                        parts[2].parse::<u64>().ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or(0);

        let percent = (used_bytes as f64 / total_bytes as f64) * 100.0;

        let severity = if percent > 90.0 {
            MetricSeverity::Critical
        } else if percent > 80.0 {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "ram_usage_percent".to_string(),
            value: format!("{:.1}", percent),
            unit: "%".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_swap_total(&self) -> MetricValue {
        let total = Command::new("free")
            .arg("-b")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| {
                let lines: Vec<&str> = s.lines().collect();
                if lines.len() >= 3 {
                    let parts: Vec<&str> = lines[2].split_whitespace().collect();
                    if parts.len() >= 2 {
                        parts[1].parse::<u64>().ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or(0);

        let total_gb = total as f64 / 1024.0 / 1024.0 / 1024.0;

        MetricValue {
            name: "swap_total".to_string(),
            value: format!("{:.2}", total_gb),
            unit: "GB".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_swap_used(&self) -> MetricValue {
        let used = Command::new("free")
            .arg("-b")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| {
                let lines: Vec<&str> = s.lines().collect();
                if lines.len() >= 3 {
                    let parts: Vec<&str> = lines[2].split_whitespace().collect();
                    if parts.len() >= 3 {
                        parts[2].parse::<u64>().ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or(0);

        let used_gb = used as f64 / 1024.0 / 1024.0 / 1024.0;

        MetricValue {
            name: "swap_used".to_string(),
            value: format!("{:.2}", used_gb),
            unit: "GB".to_string(),
            timestamp: Local::now(),
            severity: if used_gb > 0.5 { MetricSeverity::Warning } else { MetricSeverity::Good },
        }
    }

    fn get_swap_percent(&self) -> MetricValue {
        let total = Command::new("free")
            .arg("-b")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| {
                let lines: Vec<&str> = s.lines().collect();
                if lines.len() >= 3 {
                    let parts: Vec<&str> = lines[2].split_whitespace().collect();
                    if parts.len() >= 2 {
                        parts[1].parse::<u64>().ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or(1);

        let used = Command::new("free")
            .arg("-b")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| {
                let lines: Vec<&str> = s.lines().collect();
                if lines.len() >= 3 {
                    let parts: Vec<&str> = lines[2].split_whitespace().collect();
                    if parts.len() >= 3 {
                        parts[2].parse::<u64>().ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or(0);

        let percent = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };

        MetricValue {
            name: "swap_usage_percent".to_string(),
            value: format!("{:.1}", percent),
            unit: "%".to_string(),
            timestamp: Local::now(),
            severity: if percent > 50.0 { MetricSeverity::Warning } else { MetricSeverity::Good },
        }
    }

    fn get_swappiness(&self) -> MetricValue {
        let swappiness = fs::read_to_string("/proc/sys/vm/swappiness")
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(60);

        let severity = if swappiness > 80 {
            MetricSeverity::Warning
        } else if swappiness < 10 {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "swappiness".to_string(),
            value: swappiness.to_string(),
            unit: "0-100".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_cpu_temperature(&self) -> MetricValue {
        let temp = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .map(|t| t as f64 / 1000.0)
            .unwrap_or(0.0);

        let severity = if temp > 85.0 {
            MetricSeverity::Critical
        } else if temp > 70.0 {
            MetricSeverity::Warning
        } else if temp > 0.0 {
            MetricSeverity::Good
        } else {
            MetricSeverity::Info
        };

        MetricValue {
            name: "cpu_temperature".to_string(),
            value: if temp > 0.0 { format!("{:.1}", temp) } else { "N/A".to_string() },
            unit: "°C".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_cpu_vendor(&self) -> MetricValue {
        let vendor = fs::read_to_string("/proc/cpuinfo")
            .unwrap_or_default()
            .lines()
            .find(|line| line.starts_with("vendor_id"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        MetricValue {
            name: "cpu_vendor".to_string(),
            value: vendor,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_hypervisor_cpu_flags(&self) -> MetricValue {
        let flags = Command::new("cat")
            .arg("/proc/cpuinfo")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| {
                let has_hypervisor = s.lines().any(|l| l.contains("hypervisor"));
                if has_hypervisor { "Running under hypervisor".to_string() } else { "Bare-metal".to_string() }
            })
            .unwrap_or_else(|| "Unknown".to_string());

        MetricValue {
            name: "hypervisor_status".to_string(),
            value: flags,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }
}
