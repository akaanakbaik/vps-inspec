use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use chrono::Local;

use crate::collector::CompleteReport;

pub async fn create_document(output_dir: &PathBuf, report: &CompleteReport, _lang: &str) -> PathBuf {
    let output_path = output_dir.join("report.pdf");
    let mut file = File::create(&output_path).unwrap();

    let mut content = String::new();
    content.push_str("VPS INSPECTOR PROFESSIONAL REPORT (PDF)\n");
    content.push_str(&format!("Generated: {}\n\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
    content.push_str(&format!("Overall Health Score: {} / 100\n", report.overall_health_score));
    content.push_str(&format!("Total Recommendations: {}\n\n", report.recommendation_count));

    for issue in &report.critical_issues {
        content.push_str(&format!("CRITICAL: {}\n", issue));
    }
    content.push('\n');

    for metric in &report.system.metrics {
        content.push_str(&format!("SYSTEM {}: {}\n", metric.name, metric.value));
    }
    for metric in &report.hardware.metrics {
        content.push_str(&format!("HARDWARE {}: {} {}\n", metric.name, metric.value, metric.unit));
    }

    let _ = file.write_all(content.as_bytes());
    output_path
}
