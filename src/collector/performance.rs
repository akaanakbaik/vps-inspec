use super::{MetricGroup, MetricValue, MetricSeverity};
use chrono::Local;
use std::process::Command;
use std::fs;

pub struct PerformanceCollector {
    start_time: std::time::Instant,
}

impl PerformanceCollector {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    pub async fn collect(&self) -> MetricGroup {
        let start = std::time::Instant::now();
        let mut metrics = Vec::new();

        metrics.push(self.get_top_cpu_processes());
        metrics.push(self.get_top_memory_processes());
        metrics.push(self.get_zombie_processes());
        metrics.push(self.get_open_files_limit());
        metrics.push(self.get_user_processes_limit());
        metrics.push(self.get_io_wait_percent());
        metrics.push(self.get_interrupts_per_second());
        metrics.push(self.get_context_switches());
        metrics.push(self.get_load_trend());
        metrics.push(self.get_system_calls_rate());
        metrics.push(self.get_cache_efficiency());
        metrics.push(self.get_memory_fragmentation());

        let duration_ms = start.elapsed().as_millis() as u64;

        MetricGroup {
            category: "Performance Metrics".to_string(),
            metrics,
            collected_at: Local::now(),
            duration_ms,
        }
    }

    fn get_top_cpu_processes(&self) -> MetricValue {
        let top_cpu = Command::new("ps")
            .args(["aux", "--sort=-%cpu"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let top_5: Vec<String> = top_cpu.lines()
            .skip(1)
            .take(5)
            .map(|l| l.to_string())
            .collect();

        MetricValue {
            name: "top_cpu_processes".to_string(),
            value: top_5.join("\n"),
            unit: "% CPU".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_top_memory_processes(&self) -> MetricValue {
        let top_mem = Command::new("ps")
            .args(["aux", "--sort=-%mem"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let top_5: Vec<String> = top_mem.lines()
            .skip(1)
            .take(5)
            .map(|l| l.to_string())
            .collect();

        MetricValue {
            name: "top_memory_processes".to_string(),
            value: top_5.join("\n"),
            unit: "% MEM".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_zombie_processes(&self) -> MetricValue {
        let zombies = Command::new("ps")
            .args(["aux"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let zombie_count = zombies.lines()
            .filter(|l| l.contains("Z") || l.contains("defunct"))
            .count();

        let severity = if zombie_count > 5 {
            MetricSeverity::Critical
        } else if zombie_count > 0 {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "zombie_processes".to_string(),
            value: zombie_count.to_string(),
            unit: "processes".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_open_files_limit(&self) -> MetricValue {
        let soft_limit = Command::new("ulimit")
            .arg("-n")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let hard_limit = Command::new("ulimit")
            .arg("-n")
            .arg("-H")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let current_open = Command::new("sh")
            .arg("-c")
            .arg("lsof 2>/dev/null | wc -l")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        MetricValue {
            name: "open_files_limits".to_string(),
            value: format!("Soft: {} | Hard: {} | Current open: {}", soft_limit, hard_limit, current_open),
            unit: "files".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_user_processes_limit(&self) -> MetricValue {
        let max_user_procs = Command::new("ulimit")
            .arg("-u")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let current_procs = Command::new("ps")
            .args(["-u", "$USER", "--no-headers"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.lines().count().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        MetricValue {
            name: "user_process_limits".to_string(),
            value: format!("Max: {} | Current: {}", max_user_procs, current_procs),
            unit: "processes".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_io_wait_percent(&self) -> MetricValue {
        let iowait = Command::new("iostat")
            .args(["-c", "1", "1"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| {
                let re = regex::Regex::new(r"%iowait:\s+([\d\.]+)").unwrap();
                re.captures(&s).and_then(|cap| cap.get(1))
            })
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "0".to_string());

        let iowait_pct = iowait.parse::<f64>().unwrap_or(0.0);
        let severity = if iowait_pct > 20.0 {
            MetricSeverity::Critical
        } else if iowait_pct > 10.0 {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "io_wait_percent".to_string(),
            value: format!("{:.1}", iowait_pct),
            unit: "%".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_interrupts_per_second(&self) -> MetricValue {
        let first_read = fs::read_to_string("/proc/stat")
            .unwrap_or_default();
        
        std::thread::sleep(std::time::Duration::from_millis(1000));
        
        let second_read = fs::read_to_string("/proc/stat")
            .unwrap_or_default();

        let interrupts1 = first_read.lines()
            .find(|l| l.starts_with("intr"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let interrupts2 = second_read.lines()
            .find(|l| l.starts_with("intr"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let interrupts_ps = interrupts2.saturating_sub(interrupts1);

        MetricValue {
            name: "interrupts_per_second".to_string(),
            value: interrupts_ps.to_string(),
            unit: "interrupts/sec".to_string(),
            timestamp: Local::now(),
            severity: if interrupts_ps > 50000 { MetricSeverity::Warning } else { MetricSeverity::Info },
        }
    }

    fn get_context_switches(&self) -> MetricValue {
        let first_read = fs::read_to_string("/proc/stat")
            .unwrap_or_default();
        
        std::thread::sleep(std::time::Duration::from_millis(1000));
        
        let second_read = fs::read_to_string("/proc/stat")
            .unwrap_or_default();

        let ctxt1 = first_read.lines()
            .find(|l| l.starts_with("ctxt"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let ctxt2 = second_read.lines()
            .find(|l| l.starts_with("ctxt"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let ctxt_ps = ctxt2.saturating_sub(ctxt1);

        MetricValue {
            name: "context_switches_per_second".to_string(),
            value: ctxt_ps.to_string(),
            unit: "switches/sec".to_string(),
            timestamp: Local::now(),
            severity: if ctxt_ps > 100000 { MetricSeverity::Warning } else { MetricSeverity::Info },
        }
    }

    fn get_load_trend(&self) -> MetricValue {
        let load = fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|s| {
                let parts: Vec<&str> = s.split_whitespace().collect();
                if parts.len() >= 3 {
                    Some((parts[0].parse::<f64>().unwrap_or(0.0), parts[1].parse::<f64>().unwrap_or(0.0), parts[2].parse::<f64>().unwrap_or(0.0)))
                } else {
                    None
                }
            })
            .unwrap_or((0.0, 0.0, 0.0));

        let trend = if load.1 > load.0 * 1.2 {
            "INCREASING"
        } else if load.1 < load.0 * 0.8 {
            "DECREASING"
        } else {
            "STABLE"
        };

        MetricValue {
            name: "load_trend_5min_vs_1min".to_string(),
            value: format!("{:.2} → {:.2} → {:.2} ({})", load.0, load.1, load.2, trend),
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: if load.0 > 5.0 { MetricSeverity::Critical } else if load.0 > 2.0 { MetricSeverity::Warning } else { MetricSeverity::Good },
        }
    }

    fn get_system_calls_rate(&self) -> MetricValue {
        let first_read = fs::read_to_string("/proc/stat")
            .unwrap_or_default();
        
        std::thread::sleep(std::time::Duration::from_millis(1000));
        
        let second_read = fs::read_to_string("/proc/stat")
            .unwrap_or_default();

        let syscalls1 = first_read.lines()
            .find(|l| l.starts_with("syscalls"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let syscalls2 = second_read.lines()
            .find(|l| l.starts_with("syscalls"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let syscalls_ps = syscalls2.saturating_sub(syscalls1);

        MetricValue {
            name: "system_calls_per_second".to_string(),
            value: syscalls_ps.to_string(),
            unit: "calls/sec".to_string(),
            timestamp: Local::now(),
            severity: if syscalls_ps > 500000 { MetricSeverity::Warning } else { MetricSeverity::Info },
        }
    }

    fn get_cache_efficiency(&self) -> MetricValue {
        let meminfo = fs::read_to_string("/proc/meminfo").unwrap_or_default();
        
        let cached = meminfo.lines()
            .find(|l| l.starts_with("Cached:"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let total = meminfo.lines()
            .find(|l| l.starts_with("MemTotal:"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(1);

        let cache_percent = (cached as f64 / total as f64) * 100.0;

        MetricValue {
            name: "cache_efficiency".to_string(),
            value: format!("{:.1}", cache_percent),
            unit: "% RAM cached".to_string(),
            timestamp: Local::now(),
            severity: if cache_percent < 10.0 { MetricSeverity::Warning } else { MetricSeverity::Good },
        }
    }

    fn get_memory_fragmentation(&self) -> MetricValue {
        let pagetypeinfo = fs::read_to_string("/proc/pagetypeinfo")
            .unwrap_or_default();
        
        let fragmentation_line = pagetypeinfo.lines()
            .find(|l| l.contains("fragmentation"));

        let value = if let Some(line) = fragmentation_line {
            line.to_string()
        } else {
            "Fragmentation data not available".to_string()
        };

        MetricValue {
            name: "memory_fragmentation".to_string(),
            value,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }
}