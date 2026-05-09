use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use chrono::Local;
use printpdf::{BuiltinFont, Mm, PdfDocument};

use crate::collector::CompleteReport;

pub async fn create_document(output_dir: &PathBuf, report: &CompleteReport, _lang: &str, ai_analysis: Option<&str>) -> PathBuf {
    let output_path = output_dir.join("report.pdf");
    let file = match File::create(&output_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create PDF report file {}: {}", output_path.display(), e);
            return output_path;
        }
    };

    let (doc, page1, layer1) = PdfDocument::new(
        "VPS Inspector Professional Report",
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );
    let current_layer = doc.get_page(page1).get_layer(layer1);

    let font = match doc.add_builtin_font(BuiltinFont::Helvetica) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to initialize PDF font: {}", e);
            return output_path;
        }
    };

    let max_chars = 95usize;
    let mut lines = Vec::<String>::new();
    lines.push("VPS INSPECTOR PROFESSIONAL REPORT".to_string());
    lines.push(format!("Generated: {}", Local::now().format("%Y-%m-%d %H:%M:%S")));
    lines.push(String::new());
    lines.push(format!("Overall Health Score: {} / 100", report.overall_health_score));
    lines.push(format!("Total Recommendations: {}", report.recommendation_count));
    lines.push(String::new());

    lines.push("CRITICAL ISSUES".to_string());
    if report.critical_issues.is_empty() {
        lines.push("- None detected".to_string());
    } else {
        for issue in &report.critical_issues {
            lines.extend(wrap_text(&format!("- {}", issue), max_chars));
        }
    }
    lines.push(String::new());

    lines.push("SYSTEM METRICS".to_string());
    for metric in &report.system.metrics {
        lines.extend(wrap_text(&format!("- {}: {}", metric.name, metric.value), max_chars));
    }
    lines.push(String::new());

    lines.push("HARDWARE METRICS".to_string());
    for metric in &report.hardware.metrics {
        lines.extend(wrap_text(&format!("- {}: {} {}", metric.name, metric.value, metric.unit), max_chars));
    }
    lines.push(String::new());

    lines.push("AI ANALYSIS".to_string());
    if let Some(analysis) = ai_analysis {
        lines.extend(wrap_text(analysis, max_chars));
    } else {
        lines.push("AI analysis is unavailable.".to_string());
    }

    let mut y = 285.0f32;
    let line_height = 6.0f32;
    let left_margin = 12.0f32;
    for line in lines {
        if y < 15.0 {
            break;
        }
        current_layer.use_text(&line, 10.0, Mm(left_margin), Mm(y), &font);
        y -= line_height;
    }

    if let Err(e) = doc.save(&mut BufWriter::new(file)) {
        eprintln!("Failed to write PDF report file {}: {}", output_path.display(), e);
    }
    output_path
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut result = Vec::new();
    for paragraph in text.lines() {
        if paragraph.is_empty() {
            result.push(String::new());
            continue;
        }
        let mut buf = String::new();
        for word in paragraph.split_whitespace() {
            if buf.is_empty() {
                buf.push_str(word);
            } else if buf.len() + 1 + word.len() <= max_chars {
                buf.push(' ');
                buf.push_str(word);
            } else {
                result.push(buf.clone());
                buf.clear();
                buf.push_str(word);
            }
        }
        if !buf.is_empty() {
            result.push(buf);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;
    use tempfile::TempDir;
    use crate::collector::{CompleteReport, MetricGroup, MetricSeverity, MetricValue};

    fn metric(name: &str, value: &str, unit: &str, severity: MetricSeverity) -> MetricValue {
        MetricValue {
            name: name.to_string(),
            value: value.to_string(),
            unit: unit.to_string(),
            timestamp: Local::now(),
            severity,
        }
    }

    fn group(category: &str) -> MetricGroup {
        MetricGroup {
            category: category.to_string(),
            metrics: vec![metric("sample", "42", "unit", MetricSeverity::Info)],
            collected_at: Local::now(),
            duration_ms: 10,
        }
    }

    fn sample_report() -> CompleteReport {
        CompleteReport {
            system: group("system"),
            hardware: group("hardware"),
            storage: group("storage"),
            network: group("network"),
            security: group("security"),
            performance: group("performance"),
            software: group("software"),
            logs: group("logs"),
            overall_health_score: 88,
            recommendation_count: 2,
            critical_issues: vec!["Disk almost full".to_string()],
        }
    }

    #[tokio::test]
    async fn generated_pdf_has_valid_header_and_eof() {
        let dir = TempDir::new().expect("temp dir");
        let report = sample_report();
        let path = create_document(&dir.path().to_path_buf(), &report, "EN", Some("AI analysis test")).await;
        let bytes = std::fs::read(path).expect("read pdf bytes");

        assert!(bytes.starts_with(b"%PDF-"), "PDF header missing");
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("%%EOF"), "PDF EOF marker missing");
    }
}
