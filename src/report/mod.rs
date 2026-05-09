use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use reqwest::Client;
use serde_json::Value;
use crate::collector::CompleteReport;

pub mod docx;
pub mod pdf;

pub async fn generate_docx_report(output_dir: &PathBuf, report: &CompleteReport, lang: &str) {
    let file_path = docx::create_document(output_dir, report, lang).await;
    upload_to_cdn(&file_path).await;
}

pub async fn generate_pdf_report(output_dir: &PathBuf, report: &CompleteReport, lang: &str) {
    let file_path = pdf::create_document(output_dir, report, lang).await;
    upload_to_cdn(&file_path).await;
}

pub async fn upload_to_cdn(file_path: &PathBuf) -> Option<String> {
    println!("📤 Uploading {} to CDN...", file_path.display());
    
    let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
    
    let mut file = File::open(file_path).ok()?;
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content).ok()?;
    
    let part = reqwest::multipart::Part::bytes(file_content)
        .file_name(file_name)
        .mime_str("application/octet-stream").ok()?;
    
    let form = reqwest::multipart::Form::new()
        .part("file", part);
    
    let client = Client::new();
    let response = client.post("https://api.kabox.my.id/api/upload")
        .header("x-expire", "never")
        .multipart(form)
        .send()
        .await
        .ok()?;
    
    if response.status().is_success() {
        let json: Value = response.json().await.ok()?;
        let url = json["url"].as_str().unwrap_or("").to_string();
        println!("✅ File uploaded: {}", url);
        Some(url)
    } else {
        println!("❌ Upload failed: {}", response.status());
        None
    }
}

pub struct ReportGenerator {
    data: CompleteReport,
    language: String,
}

impl ReportGenerator {
    pub fn new(data: CompleteReport, language: String) -> Self {
        Self { data, language }
    }

    pub fn generate_ascii_gauge(&self, percent: f64, width: usize) -> String {
        let filled = (percent / 100.0 * width as f64).round() as usize;
        let empty = width - filled;
        let bar: String = "█".repeat(filled) + &"░".repeat(empty);
        format!("[{:.<width$}] {:.1}%", bar, percent, width = width + 4)
    }

    pub fn generate_sparkline(&self, values: &[f64]) -> String {
        let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let max_val = values.iter().fold(0.0_f64, |a, &b| a.max(b));
        if max_val == 0.0 {
            return "▁▁▁▁▁▁▁▁▁▁".to_string();
        }
        values.iter()
            .map(|&v| {
                let idx = ((v / max_val) * (chars.len() - 1) as f64).round() as usize;
                chars[idx.min(chars.len() - 1)]
            })
            .collect()
    }

    pub fn format_bytes(&self, bytes: u64) -> String {
        const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
        let mut size = bytes as f64;
        let mut unit_idx = 0;
        while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
            size /= 1024.0;
            unit_idx += 1;
        }
        format!("{:.2} {}", size, UNITS[unit_idx])
    }

    pub fn format_duration(&self, seconds: u64) -> String {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        
        if days > 0 {
            format!("{}d {}h {}m {}s", days, hours, minutes, secs)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, secs)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }

    pub fn get_health_emoji(&self, score: u8) -> &'static str {
        match score {
            90..=100 => "🟢",
            70..=89 => "🟡",
            50..=69 => "🟠",
            0..=49 => "🔴",
            _ => "⚪",
        }
    }

    pub fn get_severity_icon(&self, severity: &crate::collector::MetricSeverity) -> &'static str {
        match severity {
            crate::collector::MetricSeverity::Good => "✅",
            crate::collector::MetricSeverity::Info => "ℹ️",
            crate::collector::MetricSeverity::Warning => "⚠️",
            crate::collector::MetricSeverity::Critical => "🔴",
        }
    }
}
