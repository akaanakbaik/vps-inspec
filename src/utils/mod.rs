use std::process::Command;
use std::time::Duration;
use std::thread;

pub fn run_command_with_timeout(cmd: &str, args: &[&str], _timeout_secs: u64) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
}

pub fn parse_bash_variable(content: &str, var_name: &str) -> Option<String> {
    let pattern = format!(r#"{}\s*=\s*['"]?([^'"]+)['"]?"#, var_name);
    let re = regex::Regex::new(&pattern).ok()?;
    re.captures(content).and_then(|cap| cap.get(1)).map(|m| m.as_str().to_string())
}

pub fn safe_split_line(line: &str, delimiter: char, expected_parts: usize) -> Vec<String> {
    let parts: Vec<String> = line.split(delimiter)
        .map(|s| s.trim().to_string())
        .collect();
    
    if parts.len() >= expected_parts {
        parts
    } else {
        let mut padded = parts;
        while padded.len() < expected_parts {
            padded.push(String::new());
        }
        padded
    }
}

pub fn parse_size_to_bytes(size_str: &str) -> Option<u64> {
    let size_str = size_str.trim().to_uppercase();
    let number_part: String = size_str.chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    
    let suffix: String = size_str.chars()
        .skip_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    
    let number = number_part.parse::<f64>().ok()?;
    
    let multiplier = match suffix.trim() {
        "B" | "" => 1.0,
        "KB" | "K" => 1024.0,
        "MB" | "M" => 1024.0 * 1024.0,
        "GB" | "G" => 1024.0 * 1024.0 * 1024.0,
        "TB" | "T" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        "PB" | "P" => 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    
    Some((number * multiplier) as u64)
}

pub fn calculate_percent(part: f64, total: f64) -> f64 {
    if total == 0.0 {
        0.0
    } else {
        (part / total) * 100.0
    }
}

pub fn average(numbers: &[f64]) -> f64 {
    if numbers.is_empty() {
        0.0
    } else {
        numbers.iter().sum::<f64>() / numbers.len() as f64
    }
}

pub fn median(numbers: &mut [f64]) -> f64 {
    if numbers.is_empty() {
        return 0.0;
    }
    numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = numbers.len() / 2;
    if numbers.len() % 2 == 0 {
        (numbers[mid - 1] + numbers[mid]) / 2.0
    } else {
        numbers[mid]
    }
}

pub fn standard_deviation(numbers: &[f64]) -> f64 {
    let avg = average(numbers);
    let variance = numbers.iter()
        .map(|&x| (x - avg).powi(2))
        .sum::<f64>() / numbers.len() as f64;
    variance.sqrt()
}

pub fn detect_severity_from_threshold(value: f64, warning: f64, critical: f64) -> &'static str {
    if value >= critical {
        "CRITICAL"
    } else if value >= warning {
        "WARNING"
    } else {
        "OK"
    }
}

pub fn exponential_backoff(attempt: u32, base_delay_ms: u64) -> Duration {
    let delay = base_delay_ms * (2u64.pow(attempt));
    Duration::from_millis(delay.min(30000))
}

pub fn retry_async<F, T>(mut f: F, max_attempts: u32, base_delay_ms: u64) -> Option<T>
where
    F: FnMut() -> Option<T>,
{
    for attempt in 0..max_attempts {
        if let Some(result) = f() {
            return Some(result);
        }
        if attempt < max_attempts - 1 {
            thread::sleep(exponential_backoff(attempt, base_delay_ms));
        }
    }
    None
}

pub fn format_error_context(error: &str, context: &str, max_length: usize) -> String {
    let error_preview = if error.len() > max_length {
        let start_len = max_length / 3;
        let end_len = max_length / 3;
        format!("{}...{}", &error[..start_len], &error[error.len() - end_len..])
    } else {
        error.to_string()
    };
    format!("[{}] {}", context, error_preview)
}

pub fn extract_important_log_lines(log: &str, keywords: &[&str], max_lines: usize) -> Vec<String> {
    let mut result = Vec::new();
    for line in log.lines() {
        for keyword in keywords {
            if line.to_lowercase().contains(keyword) {
                result.push(line.to_string());
                if result.len() >= max_lines {
                    return result;
                }
                break;
            }
        }
    }
    result
}

pub fn create_timestamp_filename(prefix: &str, suffix: &str) -> String {
    let now = Local::now();
    format!("{}_{}{}", prefix, now.format("%Y%m%d_%H%M%S"), suffix)
}

use chrono::Local;

pub fn validate_ip_address(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    for part in parts {
        if let Ok(num) = part.parse::<u8>() {
            continue;
        }
        return false;
    }
    true
}

pub fn validate_port_number(port: u16) -> bool {
    port >= 1 && port <= 65535
}

pub fn sanitize_command_arg(arg: &str) -> String {
    arg.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == '.' || *c == '/')
        .collect()
}
