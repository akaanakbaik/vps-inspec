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

    let mut y = 285.0f64;
    let line_height = 6.0f64;
    let left_margin = 12.0f64;
    let max_chars = 95usize;

    let mut write_line = |line: &str| {
        if y < 15.0 {
            return;
        }
        current_layer.use_text(line, 10.0, Mm(left_margin), Mm(y), &font);
        y -= line_height;
    };

    let mut write_wrapped = |text: &str| {
        for paragraph in text.lines() {
            if paragraph.is_empty() {
                write_line("");
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
                    write_line(&buf);
                    buf.clear();
                    buf.push_str(word);
                }
            }
            if !buf.is_empty() {
                write_line(&buf);
            }
        }
    };

    write_line("VPS INSPECTOR PROFESSIONAL REPORT");
    write_line(&format!("Generated: {}", Local::now().format("%Y-%m-%d %H:%M:%S")));
    write_line("");
    write_line(&format!("Overall Health Score: {} / 100", report.overall_health_score));
    write_line(&format!("Total Recommendations: {}", report.recommendation_count));
    write_line("");

    write_line("CRITICAL ISSUES");
    if report.critical_issues.is_empty() {
        write_line("- None detected");
    } else {
        for issue in &report.critical_issues {
            write_wrapped(&format!("- {}", issue));
        }
    }
    write_line("");

    write_line("SYSTEM METRICS");
    for metric in &report.system.metrics {
        write_wrapped(&format!("- {}: {}", metric.name, metric.value));
    }
    write_line("");

    write_line("HARDWARE METRICS");
    for metric in &report.hardware.metrics {
        write_wrapped(&format!("- {}: {} {}", metric.name, metric.value, metric.unit));
    }
    write_line("");

    write_line("AI ANALYSIS");
    if let Some(analysis) = ai_analysis {
        write_wrapped(analysis);
    } else {
        write_line("AI analysis is unavailable.");
    }

    if let Err(e) = doc.save(&mut BufWriter::new(file)) {
        eprintln!("Failed to write PDF report file {}: {}", output_path.display(), e);
    }
    output_path
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
