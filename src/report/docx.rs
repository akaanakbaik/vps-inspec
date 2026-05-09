use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use docx_rs::*;
use chrono::Local;
use crate::collector::CompleteReport;
use crate::translator;
use crate::report::ReportGenerator;

pub async fn create_document(output_dir: &PathBuf, report: &CompleteReport, lang: &str) {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let output_path = output_dir.join(format!("report_{}.docx", timestamp));
    
    let mut doc = Docx::new();
    let generator = ReportGenerator::new(report.clone(), lang.to_string());
    
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text("VPS INSPECTOR PROFESSIONAL REPORT").size(48).bold().center()));
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("Generated: {}", Local::now().format("%Y-%m-%d %H:%M:%S"))).size(20).italic()));
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(" ")));
    
    doc = doc.add_heading(Heading::new("EXECUTIVE SUMMARY", 1));
    let health_icon = generator.get_health_emoji(report.overall_health_score);
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("{} Overall Health Score: {} / 100", health_icon, report.overall_health_score)).bold().size(24)));
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("📊 Total Recommendations: {}", report.recommendation_count))));
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("🔴 Critical Issues: {}", report.critical_issues.len()))));
    
    for issue in &report.critical_issues {
        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("• {}", issue)).color("FF0000")));
    }
    
    doc = doc.add_heading(Heading::new("SYSTEM INFORMATION", 1));
    for metric in &report.system.metrics {
        let icon = generator.get_severity_icon(&metric.severity);
        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("{} {}: {}", icon, metric.name, metric.value))));
    }
    
    doc = doc.add_heading(Heading::new("HARDWARE ANALYSIS", 1));
    let mut cpu_usage = 0.0;
    let mut ram_usage = 0.0;
    
    for metric in &report.hardware.metrics {
        let icon = generator.get_severity_icon(&metric.severity);
        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("{} {}: {} {}", icon, metric.name, metric.value, metric.unit))));
        
        if metric.name == "cpu_usage_percent" {
            cpu_usage = metric.value.parse().unwrap_or(0.0);
        }
        if metric.name == "ram_usage_percent" {
            ram_usage = metric.value.parse().unwrap_or(0.0);
        }
    }
    
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("CPU Gauge: {}", generator.generate_ascii_gauge(cpu_usage, 30)))));
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("RAM Gauge: {}", generator.generate_ascii_gauge(ram_usage, 30)))));
    
    doc = doc.add_heading(Heading::new("STORAGE ANALYSIS", 1));
    for metric in &report.storage.metrics {
        let icon = generator.get_severity_icon(&metric.severity);
        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("{} {}: {}", icon, metric.name, metric.value))));
    }
    
    doc = doc.add_heading(Heading::new("NETWORK CONFIGURATION", 1));
    for metric in &report.network.metrics {
        let icon = generator.get_severity_icon(&metric.severity);
        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("{} {}: {}", icon, metric.name, metric.value))));
    }
    
    doc = doc.add_heading(Heading::new("SECURITY AUDIT", 1));
    for metric in &report.security.metrics {
        let icon = generator.get_severity_icon(&metric.severity);
        let color = match metric.severity {
            crate::collector::MetricSeverity::Critical => "FF0000",
            crate::collector::MetricSeverity::Warning => "FFA500",
            _ => "000000",
        };
        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("{} {}: {}", icon, metric.name, metric.value)).color(color)));
    }
    
    doc = doc.add_heading(Heading::new("PERFORMANCE METRICS", 1));
    for metric in &report.performance.metrics {
        let icon = generator.get_severity_icon(&metric.severity);
        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("{} {}: {} {}", icon, metric.name, metric.value, metric.unit))));
    }
    
    doc = doc.add_heading(Heading::new("SOFTWARE INVENTORY", 1));
    for metric in &report.software.metrics {
        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("{}: {}", metric.name, metric.value))));
    }
    
    doc = doc.add_heading(Heading::new("LOG ANALYSIS", 1));
    for metric in &report.logs.metrics {
        let icon = generator.get_severity_icon(&metric.severity);
        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("{} {}:", icon, metric.name)).bold()));
        let lines: Vec<&str> = metric.value.split('\n').collect();
        for line in lines.iter().take(10) {
            doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(line).size(16)));
        }
    }
    
    doc = doc.add_heading(Heading::new("RECOMMENDATIONS", 1));
    let mut rec_count = 1;
    
    for metric in &report.security.metrics {
        if metric.severity == crate::collector::MetricSeverity::Critical || metric.severity == crate::collector::MetricSeverity::Warning {
            let recommendation = match metric.name.as_str() {
                "ssh_root_login" => "Disable SSH root login: PermitRootLogin no",
                "ssh_password_authentication" => "Disable SSH password authentication: Use SSH keys only",
                "failed_logins_24h" => "Consider installing fail2ban to prevent brute force attacks",
                "pending_security_updates" => "Run security updates immediately: apt upgrade or yum update",
                _ => &format!("Review and address: {}", metric.name),
            };
            doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("{}. {}", rec_count, recommendation)).color("FFA500")));
            rec_count += 1;
        }
    }
    
    for metric in &report.storage.metrics {
        if metric.severity == crate::collector::MetricSeverity::Critical {
            doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("{}. Free up disk space: {}", rec_count, metric.name)).color("FF0000")));
            rec_count += 1;
        }
    }
    
    if rec_count == 1 {
        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text("✅ No critical recommendations. System is healthy!").color("00AA00")));
    }
    
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(" ")));
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text("Report generated by VPS Inspector Professional").italic().size(16)));
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text("https://github.com/akaanakbaik/vps-inspec").italic().size(14)));
    
    let bytes = doc.build().unwrap();
    let mut file = File::create(output_path).unwrap();
    file.write_all(&bytes).unwrap();
}