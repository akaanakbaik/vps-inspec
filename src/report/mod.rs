use std::path::PathBuf;
use crate::collector::CompleteReport;

pub mod docx;
pub mod pdf;

pub async fn generate_docx_report(output_dir: &PathBuf, lang: &str) {
    let report = crate::collector::CollectorManager::new().collect_all().await;
    docx::create_document(output_dir, &report, lang).await;
}

pub async fn generate_pdf_report(output_dir: &PathBuf, lang: &str) {
    let report = crate::collector::CollectorManager::new().collect_all().await;
    pdf::create_document(output_dir, &report, lang).await;
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
        let max_val = values.iter().fold(0.0, |a, &b| a.max(b));
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

    pub fn generate_radial_gauge(&self, percent: f64, radius: usize) -> String {
        let total_blocks = radius * 4;
        let filled = (percent / 100.0 * total_blocks as f64).round() as usize;
        let gauge = (0..total_blocks)
            .map(|i| if i < filled { '◉' } else { '○' })
            .collect::<String>();
        
        let chunks: Vec<String> = gauge.chars()
            .collect::<Vec<char>>()
            .chunks(radius)
            .map(|c| c.iter().collect())
            .collect();
        
        format!("┌{}┐\n│{}│\n│{}│\n│{}│\n└{}┘\n{:.1}%",
            "─".repeat(radius), chunks[0], chunks[1], chunks[2], "─".repeat(radius), percent)
    }

    pub fn generate_histogram(&self, data: &[(String, f64)], width: usize) -> String {
        let max_val = data.iter().map(|(_, v)| *v).fold(0.0, |a, b| a.max(b));
        if max_val == 0.0 {
            return "No data available".to_string();
        }
        
        let mut result = String::new();
        for (label, value) in data {
            let bar_length = ((value / max_val) * width as f64).round() as usize;
            let bar = "█".repeat(bar_length);
            result.push_str(&format!("{:>15} | {:3.1}% {}\n", label, value, bar));
        }
        result
    }

    pub fn generate_table(&self, headers: &[&str], rows: &[Vec<String>], column_widths: &[usize]) -> String {
        let mut result = String::new();
        
        let separator: String = column_widths.iter()
            .map(|&w| "+".to_string() + &"-".repeat(w))
            .collect::<String>() + "+";
        
        result.push_str(&separator);
        result.push('\n');
        
        result.push('|');
        for (i, header) in headers.iter().enumerate() {
            let width = column_widths[i];
            result.push_str(&format!(" {:width$} |", header, width = width));
        }
        result.push('\n');
        
        result.push_str(&separator);
        result.push('\n');
        
        for row in rows {
            result.push('|');
            for (i, cell) in row.iter().enumerate() {
                let width = column_widths[i];
                let truncated = if cell.len() > width { &cell[..width-3] } else { cell.as_str() };
                result.push_str(&format!(" {:width$} |", truncated, width = width));
            }
            result.push('\n');
        }
        
        result.push_str(&separator);
        result
    }
}