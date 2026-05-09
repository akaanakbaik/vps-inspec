use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use chrono::Local;

use crate::collector::CompleteReport;

pub async fn create_document(output_dir: &PathBuf, report: &CompleteReport, _lang: &str) -> PathBuf {
    let output_path = output_dir.join("report.docx");
    let mut file = match File::create(&output_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create DOCX report file {}: {}", output_path.display(), e);
            return output_path;
        }
    };

    let mut content = String::new();
    content.push_str("VPS INSPECTOR PROFESSIONAL REPORT\n");
    content.push_str(&format!("Generated: {}\n\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
    content.push_str(&format!("Overall Health Score: {} / 100\n", report.overall_health_score));
    content.push_str(&format!("Total Recommendations: {}\n", report.recommendation_count));
    content.push_str(&format!("Critical Issues: {}\n\n", report.critical_issues.len()));

    for issue in &report.critical_issues {
        content.push_str(&format!("- {}\n", issue));
    }
    content.push('\n');

    let groups = [
        ("SYSTEM INFORMATION", &report.system),
        ("HARDWARE ANALYSIS", &report.hardware),
        ("STORAGE ANALYSIS", &report.storage),
        ("NETWORK CONFIGURATION", &report.network),
        ("SECURITY AUDIT", &report.security),
        ("PERFORMANCE METRICS", &report.performance),
        ("SOFTWARE INVENTORY", &report.software),
        ("LOG ANALYSIS", &report.logs),
    ];

    for (title, group) in groups {
        content.push_str(&format!("{}\n", title));
        content.push_str(&"-".repeat(title.len()));
        content.push('\n');
        for metric in &group.metrics {
            content.push_str(&format!("- {}: {} {}\n", metric.name, metric.value, metric.unit));
        }
        content.push('\n');
    }

    if let Err(e) = file.write_all(content.as_bytes()) {
        eprintln!("Failed to write DOCX report file {}: {}", output_path.display(), e);
    }
    output_path
}
