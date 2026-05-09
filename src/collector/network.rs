use super::{MetricGroup, MetricValue, MetricSeverity};
use chrono::Local;
use std::process::Command;
use std::fs;
use regex::Regex;

pub struct NetworkCollector {
    start_time: std::time::Instant,
}

impl NetworkCollector {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    pub async fn collect(&self) -> MetricGroup {
        let start = std::time::Instant::now();
        let mut metrics = Vec::new();

        metrics.push(self.get_interfaces());
        metrics.push(self.get_ip_addresses());
        metrics.push(self.get_mac_addresses());
        metrics.push(self.get_interface_speeds());
        metrics.push(self.get_dns_servers());
        metrics.push(self.get_listening_ports());
        metrics.push(self.get_active_connections());
        metrics.push(self.get_connection_count());
        metrics.push(self.get_gateway_rtt());
        metrics.push(self.get_packet_loss());
        metrics.push(self.get_bandwidth_stats());
        metrics.push(self.get_firewall_status());
        metrics.push(self.get_network_namespaces());
        metrics.push(self.get_ipv6_status());
        metrics.push(self.get_tuning_parameters());
        metrics.push(self.get_network_errors());

        let duration_ms = start.elapsed().as_millis() as u64;

        MetricGroup {
            category: "Network Configuration".to_string(),
            metrics,
            collected_at: Local::now(),
            duration_ms,
        }
    }

    fn get_interfaces(&self) -> MetricValue {
        let output = Command::new("ip")
            .args(["-br", "link"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let interfaces: Vec<String> = output.lines()
            .map(|l| l.to_string())
            .collect();

        MetricValue {
            name: "network_interfaces".to_string(),
            value: interfaces.join("\n"),
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_ip_addresses(&self) -> MetricValue {
        let output = Command::new("ip")
            .args(["-br", "addr"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let mut public_ips = Vec::new();
        let mut private_ips = Vec::new();

        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let ips: Vec<&str> = parts[2].split('/').collect();
                let ip = ips[0];
                if ip.starts_with("10.") || ip.starts_with("172.") || ip.starts_with("192.168.") {
                    private_ips.push(ip);
                } else if !ip.starts_with("127.") && ip != "::1" {
                    public_ips.push(ip);
                }
            }
        }

        let public = if public_ips.is_empty() {
            "none".to_string()
        } else {
            public_ips.join(", ")
        };
        let private = if private_ips.is_empty() {
            "none".to_string()
        } else {
            private_ips.join(", ")
        };
        let value = format!("Public IPs: {}\nPrivate IPs: {}", public, private);

        MetricValue {
            name: "ip_addresses".to_string(),
            value,
            unit: "IPv4/IPv6".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_mac_addresses(&self) -> MetricValue {
        let output = Command::new("ip")
            .args(["link"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let re = Regex::new(r"\d+: (\w+):.*link/ether ([0-9a-f:]{17})").unwrap();
        let mut macs = Vec::new();

        for cap in re.captures_iter(&output) {
            macs.push(format!("{}: {}", &cap[1], &cap[2]));
        }

        MetricValue {
            name: "mac_addresses".to_string(),
            value: if macs.is_empty() { "No MAC addresses found".to_string() } else { macs.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_interface_speeds(&self) -> MetricValue {
        let mut speeds = Vec::new();

        if let Ok(entries) = fs::read_dir("/sys/class/net/") {
            for entry in entries.flatten() {
                let iface = entry.file_name();
                let iface_str = iface.to_string_lossy();
                let speed_path = format!("/sys/class/net/{}/speed", iface_str);

                if let Ok(speed) = fs::read_to_string(&speed_path) {
                    let speed_val = speed.trim();
                    if let Ok(speed_mbps) = speed_val.parse::<u32>() {
                        let speed_gbps = speed_mbps as f64 / 1000.0;
                        speeds.push(format!("{}: {:.1} Gbps ({} Mbps)", iface_str, speed_gbps, speed_mbps));
                    }
                } else {
                    speeds.push(format!("{}: speed unknown", iface_str));
                }
            }
        }

        MetricValue {
            name: "interface_speeds".to_string(),
            value: if speeds.is_empty() { "No interface speed data".to_string() } else { speeds.join("\n") },
            unit: "bps".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_dns_servers(&self) -> MetricValue {
        let resolv = fs::read_to_string("/etc/resolv.conf")
            .unwrap_or_default();
        
        let dns_servers: Vec<String> = resolv.lines()
            .filter(|l| l.starts_with("nameserver"))
            .map(|l| l.replace("nameserver", "").trim().to_string())
            .collect();

        let search_domains: Vec<String> = resolv.lines()
            .filter(|l| l.starts_with("search"))
            .map(|l| l.replace("search", "").trim().to_string())
            .collect();

        let dns = if dns_servers.is_empty() {
            "default".to_string()
        } else {
            dns_servers.join(", ")
        };
        let search = if search_domains.is_empty() {
            "none".to_string()
        } else {
            search_domains.join(", ")
        };
        let value = format!("DNS Servers: {}\nSearch Domains: {}", dns, search);

        MetricValue {
            name: "dns_configuration".to_string(),
            value,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_listening_ports(&self) -> MetricValue {
        let output = Command::new("ss")
            .args(["-tlnp"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let mut ports = Vec::new();
        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let local = parts[4];
                let service = if parts.len() >= 6 { parts[5] } else { "-" };
                ports.push(format!("{} -> {}", local, service));
            }
        }

        let severity = if ports.iter().any(|p| p.contains(":22")) {
            MetricSeverity::Good
        } else if ports.is_empty() {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Info
        };

        MetricValue {
            name: "listening_ports".to_string(),
            value: if ports.is_empty() { "No listening ports found".to_string() } else { ports.join("\n") },
            unit: "port/service".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_active_connections(&self) -> MetricValue {
        let output = Command::new("ss")
            .args(["-tn"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let connections: Vec<String> = output.lines()
            .skip(1)
            .map(|l| l.to_string())
            .collect();

        MetricValue {
            name: "active_connections".to_string(),
            value: format!("{} active connections\n{}", connections.len(), connections.join("\n")),
            unit: "connections".to_string(),
            timestamp: Local::now(),
            severity: if connections.len() > 1000 { MetricSeverity::Warning } else { MetricSeverity::Info },
        }
    }

    fn get_connection_count(&self) -> MetricValue {
        let established = Command::new("ss")
            .args(["-tn", "state", "established"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.lines().count().saturating_sub(1))
            .unwrap_or(0);

        let time_wait = Command::new("ss")
            .args(["-tn", "state", "time-wait"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.lines().count().saturating_sub(1))
            .unwrap_or(0);

        let close_wait = Command::new("ss")
            .args(["-tn", "state", "close-wait"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.lines().count().saturating_sub(1))
            .unwrap_or(0);

        MetricValue {
            name: "connection_states".to_string(),
            value: format!("Established: {}\nTime-Wait: {}\nClose-Wait: {}", established, time_wait, close_wait),
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: if close_wait > 100 { MetricSeverity::Warning } else { MetricSeverity::Info },
        }
    }

    fn get_gateway_rtt(&self) -> MetricValue {
        let default_gw = Command::new("ip")
            .args(["route", "show", "default"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.split_whitespace().nth(2).map(|v| v.to_string()));

        let rtt = if let Some(gateway) = default_gw {
            let ping = Command::new("ping")
                .args(["-c", "3", "-W", "2", &gateway])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok());
            
            if let Some(ping_out) = ping {
                let re = Regex::new(r"rtt min/avg/max/mdev = ([\d\.]+)/([\d\.]+)/([\d\.]+)/([\d\.]+)").unwrap();
                if let Some(cap) = re.captures(&ping_out) {
                    format!("min={}ms, avg={}ms, max={}ms, mdev={}ms", &cap[1], &cap[2], &cap[3], &cap[4])
                } else {
                    "Unable to calculate RTT".to_string()
                }
            } else {
                "Ping failed".to_string()
            }
        } else {
            "No default gateway found".to_string()
        };

        MetricValue {
            name: "gateway_rtt".to_string(),
            value: rtt,
            unit: "ms".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_packet_loss(&self) -> MetricValue {
        let loss = Command::new("ping")
            .args(["-c", "5", "-W", "2", "8.8.8.8"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| {
                let re = Regex::new(r"(\d+)% packet loss").unwrap();
                re.captures(&s)
                    .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            })
            .unwrap_or_else(|| "100".to_string());

        let loss_pct = loss.parse::<u8>().unwrap_or(100);
        let severity = if loss_pct > 20 {
            MetricSeverity::Critical
        } else if loss_pct > 5 {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "packet_loss".to_string(),
            value: format!("{}%", loss),
            unit: "%".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_bandwidth_stats(&self) -> MetricValue {
        let rx_bytes = Command::new("cat")
            .arg("/sys/class/net/eth0/statistics/rx_bytes")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);

        let tx_bytes = Command::new("cat")
            .arg("/sys/class/net/eth0/statistics/tx_bytes")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);

        let rx_gb = rx_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
        let tx_gb = tx_bytes as f64 / 1024.0 / 1024.0 / 1024.0;

        MetricValue {
            name: "bandwidth_usage".to_string(),
            value: format!("RX: {:.2} GB, TX: {:.2} GB (since boot)", rx_gb, tx_gb),
            unit: "bytes".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_firewall_status(&self) -> MetricValue {
        let ufw_status = Command::new("ufw")
            .arg("status")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| {
                if s.contains("active") {
                    "UFW: ACTIVE".to_string()
                } else {
                    "UFW: INACTIVE".to_string()
                }
            });

        let iptables_rules = Command::new("iptables")
            .args(["-L", "-n"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| {
                let chain_count = s.lines()
                    .filter(|l| l.starts_with("Chain"))
                    .count();
                format!("iptables: {} chains configured", chain_count)
            });

        let status = match (ufw_status, iptables_rules) {
            (Some(ufw), Some(ip)) => format!("{}\n{}", ufw, ip),
            (Some(ufw), None) => ufw,
            (None, Some(ip)) => ip,
            (None, None) => "Firewall status unknown".to_string(),
        };

        let severity = if status.contains("ACTIVE") || status.contains("chains configured") {
            MetricSeverity::Good
        } else {
            MetricSeverity::Warning
        };

        MetricValue {
            name: "firewall_status".to_string(),
            value: status,
            unit: "".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn get_network_namespaces(&self) -> MetricValue {
        let namespaces = Command::new("ip")
            .args(["netns", "list"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let ns_list: Vec<String> = namespaces.lines()
            .map(|l| l.to_string())
            .collect();

        MetricValue {
            name: "network_namespaces".to_string(),
            value: if ns_list.is_empty() { "No custom network namespaces".to_string() } else { ns_list.join("\n") },
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_ipv6_status(&self) -> MetricValue {
        let ipv6_enabled = fs::read_to_string("/proc/sys/net/ipv6/conf/all/disable_ipv6")
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok())
            .map(|v| v == 0)
            .unwrap_or(false);

        let has_ipv6_addr = Command::new("ip")
            .args(["-6", "addr"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.lines().count() > 1)
            .unwrap_or(false);

        let status = if ipv6_enabled && has_ipv6_addr {
            "IPv6: ENABLED and configured"
        } else if ipv6_enabled {
            "IPv6: ENABLED but no addresses"
        } else {
            "IPv6: DISABLED"
        };

        MetricValue {
            name: "ipv6_status".to_string(),
            value: status.to_string(),
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_tuning_parameters(&self) -> MetricValue {
        let tcp_window = fs::read_to_string("/proc/sys/net/core/rmem_default")
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .map(|v| format!("{} bytes", v))
            .unwrap_or_else(|| "unknown".to_string());

        let tcp_congestion = fs::read_to_string("/proc/sys/net/ipv4/tcp_congestion_control")
            .unwrap_or_default()
            .trim()
            .to_string();

        MetricValue {
            name: "network_tuning".to_string(),
            value: format!("TCP Receive Buffer: {}\nTCP Congestion Control: {}", tcp_window, tcp_congestion),
            unit: "".to_string(),
            timestamp: Local::now(),
            severity: MetricSeverity::Info,
        }
    }

    fn get_network_errors(&self) -> MetricValue {
        let net_dev = fs::read_to_string("/proc/net/dev")
            .unwrap_or_default();
        
        let mut errors = Vec::new();
        for line in net_dev.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let iface = parts[0].replace(":", "");
                let rx_err = parts[3].parse::<u64>().unwrap_or(0);
                let tx_err = parts[11].parse::<u64>().unwrap_or(0);
                if rx_err > 0 || tx_err > 0 {
                    errors.push(format!("{}: RX errors={}, TX errors={}", iface, rx_err, tx_err));
                }
            }
        }

        let severity = if errors.len() > 0 {
            MetricSeverity::Warning
        } else {
            MetricSeverity::Good
        };

        MetricValue {
            name: "network_errors".to_string(),
            value: if errors.is_empty() { "No network errors detected".to_string() } else { errors.join("\n") },
            unit: "packets".to_string(),
            timestamp: Local::now(),
            severity,
        }
    }
}
