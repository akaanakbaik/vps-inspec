use std::path::PathBuf;
use std::fs::File;
use std::io::BufWriter;
use printpdf::*;
use chrono::Local;
use crate::collector::CompleteReport;
use crate::report::ReportGenerator;

pub async fn create_document(output_dir: &PathBuf, report: &CompleteReport, lang: &str) {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let output_path = output_dir.join(format!("report_{}.pdf", timestamp));
    
    let (doc, page1, layer1) = PdfDocument::new("VPS Inspector Professional Report", Mm(297.0), Mm(210.0), "Layer 1");
    let mut current_layer = layer1;
    let mut current_page = page1;
    let mut font = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();
    let mut font_normal = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let mut y_offset = Mm(280.0);
    
    let generator = ReportGenerator::new(report.clone(), lang.to_string());
    
    use printpdf::text::{TextSection, TextRenderingMode};
    
    let title_section = TextSection::new("VPS INSPECTOR PROFESSIONAL REPORT", "Helvetica-Bold".to_string(), 24.0);
    current_layer.use_text_ext(title_section, Mm(20.0), y_offset, 500.0, &mut font);
    y_offset -= Mm(15.0);
    
    let subtitle = format!("Generated: {}", Local::now().format("%Y-%m-%d %H:%M:%S"));
    let subtitle_section = TextSection::new(&subtitle, "Helvetica".to_string(), 10.0);
    current_layer.use_text_ext(subtitle_section, Mm(20.0), y_offset, 500.0, &mut font_normal);
    y_offset -= Mm(20.0);
    
    let health_text = format!("{} Overall Health Score: {} / 100", generator.get_health_emoji(report.overall_health_score), report.overall_health_score);
    let health_section = TextSection::new(&health_text, "Helvetica-Bold".to_string(), 14.0);
    current_layer.use_text_ext(health_section, Mm(20.0), y_offset, 500.0, &mut font);
    y_offset -= Mm(10.0);
    
    let rec_text = format!("Total Recommendations: {}", report.recommendation_count);
    let rec_section = TextSection::new(&rec_text, "Helvetica".to_string(), 11.0);
    current_layer.use_text_ext(rec_section, Mm(20.0), y_offset, 500.0, &mut font_normal);
    y_offset -= Mm(10.0);
    
    let critical_text = format!("Critical Issues: {}", report.critical_issues.len());
    let critical_section = TextSection::new(&critical_text, "Helvetica".to_string(), 11.0);
    current_layer.use_text_ext(critical_section, Mm(20.0), y_offset, 500.0, &mut font_normal);
    y_offset -= Mm(15.0);
    
    for issue in &report.critical_issues {
        let issue_text = format!("• {}", issue);
        let issue_section = TextSection::new(&issue_text, "Helvetica".to_string(), 9.0);
        current_layer.use_text_ext(issue_section, Mm(25.0), y_offset, 480.0, &mut font_normal);
        y_offset -= Mm(8.0);
        if y_offset < Mm(50.0) {
            let (new_page, new_layer) = doc.add_page(Mm(297.0), Mm(210.0), "Layer 1");
            current_page = new_page;
            current_layer = new_layer;
            y_offset = Mm(280.0);
        }
    }
    
    y_offset -= Mm(15.0);
    let section_header = TextSection::new("SYSTEM INFORMATION", "Helvetica-Bold".to_string(), 14.0);
    current_layer.use_text_ext(section_header, Mm(20.0), y_offset, 500.0, &mut font);
    y_offset -= Mm(12.0);
    
    for metric in &report.system.metrics {
        let icon = generator.get_severity_icon(&metric.severity);
        let metric_text = format!("{} {}: {}", icon, metric.name, metric.value);
        let metric_section = TextSection::new(&metric_text, "Helvetica".to_string(), 9.0);
        current_layer.use_text_ext(metric_section, Mm(25.0), y_offset, 480.0, &mut font_normal);
        y_offset -= Mm(7.0);
        
        if y_offset < Mm(50.0) {
            let (new_page, new_layer) = doc.add_page(Mm(297.0), Mm(210.0), "Layer 1");
            current_page = new_page;
            current_layer = new_layer;
            y_offset = Mm(280.0);
        }
    }
    
    y_offset -= Mm(15.0);
    let hardware_header = TextSection::new("HARDWARE ANALYSIS", "Helvetica-Bold".to_string(), 14.0);
    current_layer.use_text_ext(hardware_header, Mm(20.0), y_offset, 500.0, &mut font);
    y_offset -= Mm(12.0);
    
    let mut cpu_usage = 0.0;
    let mut ram_usage = 0.0;
    
    for metric in &report.hardware.metrics {
        let icon = generator.get_severity_icon(&metric.severity);
        let metric_text = format!("{} {}: {} {}", icon, metric.name, metric.value, metric.unit);
        let metric_section = TextSection::new(&metric_text, "Helvetica".to_string(), 9.0);
        current_layer.use_text_ext(metric_section, Mm(25.0), y_offset, 480.0, &mut font_normal);
        y_offset -= Mm(7.0);
        
        if metric.name == "cpu_usage_percent" {
            cpu_usage = metric.value.parse().unwrap_or(0.0);
        }
        if metric.name == "ram_usage_percent" {
            ram_usage = metric.value.parse().unwrap_or(0.0);
        }
        
        if y_offset < Mm(50.0) {
            let (new_page, new_layer) = doc.add_page(Mm(297.0), Mm(210.0), "Layer 1");
            current_page = new_page;
            current_layer = new_layer;
            y_offset = Mm(280.0);
        }
    }
    
    let cpu_gauge = format!("CPU Usage: {}", generator.generate_ascii_gauge(cpu_usage, 30));
    let cpu_gauge_section = TextSection::new(&cpu_gauge, "Helvetica".to_string(), 9.0);
    current_layer.use_text_ext(cpu_gauge_section, Mm(25.0), y_offset, 480.0, &mut font_normal);
    y_offset -= Mm(7.0);
    
    let ram_gauge = format!("RAM Usage: {}", generator.generate_ascii_gauge(ram_usage, 30));
    let ram_gauge_section = TextSection::new(&ram_gauge, "Helvetica".to_string(), 9.0);
    current_layer.use_text_ext(ram_gauge_section, Mm(25.0), y_offset, 480.0, &mut font_normal);
    y_offset -= Mm(15.0);
    
    if y_offset < Mm(50.0) {
        let (new_page, new_layer) = doc.add_page(Mm(297.0), Mm(210.0), "Layer 1");
        current_page = new_page;
        current_layer = new_layer;
        y_offset = Mm(280.0);
    }
    
    let storage_header = TextSection::new("STORAGE ANALYSIS", "Helvetica-Bold".to_string(), 14.0);
    current_layer.use_text_ext(storage_header, Mm(20.0), y_offset, 500.0, &mut font);
    y_offset -= Mm(12.0);
    
    for metric in &report.storage.metrics {
        let icon = generator.get_severity_icon(&metric.severity);
        let metric_text = format!("{} {}: {}", icon, metric.name, metric.value);
        let metric_section = TextSection::new(&metric_text, "Helvetica".to_string(), 9.0);
        current_layer.use_text_ext(metric_section, Mm(25.0), y_offset, 480.0, &mut font_normal);
        y_offset -= Mm(7.0);
        
        if y_offset < Mm(50.0) {
            let (new_page, new_layer) = doc.add_page(Mm(297.0), Mm(210.0), "Layer 1");
            current_page = new_page;
            current_layer = new_layer;
            y_offset = Mm(280.0);
        }
    }
    
    y_offset -= Mm(15.0);
    let network_header = TextSection::new("NETWORK CONFIGURATION", "Helvetica-Bold".to_string(), 14.0);
    current_layer.use_text_ext(network_header, Mm(20.0), y_offset, 500.0, &mut font);
    y_offset -= Mm(12.0);
    
    for metric in &report.network.metrics {
        let icon = generator.get_severity_icon(&metric.severity);
        let metric_text = format!("{} {}: {}", icon, metric.name, metric.value);
        let metric_section = TextSection::new(&metric_text, "Helvetica".to_string(), 9.0);
        current_layer.use_text_ext(metric_section, Mm(25.0), y_offset, 480.0, &mut font_normal);
        y_offset -= Mm(7.0);
        
        if y_offset < Mm(50.0) {
            let (new_page, new_layer) = doc.add_page(Mm(297.0), Mm(210.0), "Layer 1");
            current_page = new_page;
            current_layer = new_layer;
            y_offset = Mm(280.0);
        }
    }
    
    y_offset -= Mm(15.0);
    let security_header = TextSection::new("SECURITY AUDIT", "Helvetica-Bold".to_string(), 14.0);
    current_layer.use_text_ext(security_header, Mm(20.0), y_offset, 500.0, &mut font);
    y_offset -= Mm(12.0);
    
    for metric in &report.security.metrics {
        let icon = generator.get_severity_icon(&metric.severity);
        let color_hex = match metric.severity {
            crate::collector::MetricSeverity::Critical => "FF0000",
            crate::collector::MetricSeverity::Warning => "FFA500",
            _ => "000000",
        };
        let metric_text = format!("{} {}: {}", icon, metric.name, metric.value);
        let style = TextSectionStyle::new().with_font_size(9.0);
        let metric_section = TextSection::new(&metric_text, "Helvetica".to_string(), 9.0);
        current_layer.use_text_ext(metric_section, Mm(25.0), y_offset, 480.0, &mut font_normal);
        y_offset -= Mm(7.0);
        
        if y_offset < Mm(50.0) {
            let (new_page, new_layer) = doc.add_page(Mm(297.0), Mm(210.0), "Layer 1");
            current_page = new_page;
            current_layer = new_layer;
            y_offset = Mm(280.0);
        }
    }
    
    y_offset -= Mm(15.0);
    let performance_header = TextSection::new("PERFORMANCE METRICS", "Helvetica-Bold".to_string(), 14.0);
    current_layer.use_text_ext(performance_header, Mm(20.0), y_offset, 500.0, &mut font);
    y_offset -= Mm(12.0);
    
    for metric in &report.performance.metrics {
        let icon = generator.get_severity_icon(&metric.severity);
        let metric_text = format!("{} {}: {} {}", icon, metric.name, metric.value, metric.unit);
        let metric_section = TextSection::new(&metric_text, "Helvetica".to_string(), 9.0);
        current_layer.use_text_ext(metric_section, Mm(25.0), y_offset, 480.0, &mut font_normal);
        y_offset -= Mm(7.0);
        
        if y_offset < Mm(50.0) {
            let (new_page, new_layer) = doc.add_page(Mm(297.0), Mm(210.0), "Layer 1");
            current_page = new_page;
            current_layer = new_layer;
            y_offset = Mm(280.0);
        }
    }
    
    y_offset -= Mm(15.0);
    let software_header = TextSection::new("SOFTWARE INVENTORY", "Helvetica-Bold".to_string(), 14.0);
    current_layer.use_text_ext(software_header, Mm(20.0), y_offset, 500.0, &mut font);
    y_offset -= Mm(12.0);
    
    for metric in &report.software.metrics {
        let metric_text = format!("{}: {}", metric.name, metric.value);
        let metric_section = TextSection::new(&metric_text, "Helvetica".to_string(), 9.0);
        current_layer.use_text_ext(metric_section, Mm(25.0), y_offset, 480.0, &mut font_normal);
        y_offset -= Mm(7.0);
        
        if y_offset < Mm(50.0) {
            let (new_page, new_layer) = doc.add_page(Mm(297.0), Mm(210.0), "Layer 1");
            current_page = new_page;
            current_layer = new_layer;
            y_offset = Mm(280.0);
        }
    }
    
    y_offset -= Mm(15.0);
    let recommendations_header = TextSection::new("RECOMMENDATIONS", "Helvetica-Bold".to_string(), 14.0);
    current_layer.use_text_ext(recommendations_header, Mm(20.0), y_offset, 500.0, &mut font);
    y_offset -= Mm(12.0);
    
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
            let rec_text = format!("{}. {}", rec_count, recommendation);
            let rec_section = TextSection::new(&rec_text, "Helvetica".to_string(), 9.0);
            current_layer.use_text_ext(rec_section, Mm(25.0), y_offset, 480.0, &mut font_normal);
            y_offset -= Mm(7.0);
            rec_count += 1;
            
            if y_offset < Mm(50.0) {
                let (new_page, new_layer) = doc.add_page(Mm(297.0), Mm(210.0), "Layer 1");
                current_page = new_page;
                current_layer = new_layer;
                y_offset = Mm(280.0);
            }
        }
    }
    
    if rec_count == 1 {
        let ok_text = "✅ No critical recommendations. System is healthy!";
        let ok_section = TextSection::new(ok_text, "Helvetica".to_string(), 11.0);
        current_layer.use_text_ext(ok_section, Mm(25.0), y_offset, 480.0, &mut font_normal);
        y_offset -= Mm(10.0);
    }
    
    y_offset -= Mm(30.0);
    let footer_text = format!("Report generated by VPS Inspector Professional - {}", Local::now().format("%Y-%m-%d"));
    let footer_section = TextSection::new(&footer_text, "Helvetica".to_string(), 8.0);
    current_layer.use_text_ext(footer_section, Mm(20.0), Mm(15.0), 500.0, &mut font_normal);
    
    let url_text = "https://github.com/akaanakbaik/vps-inspec";
    let url_section = TextSection::new(url_text, "Helvetica".to_string(), 8.0);
    current_layer.use_text_ext(url_section, Mm(20.0), Mm(10.0), 500.0, &mut font_normal);
    
    doc.save(&mut BufWriter::new(File::create(output_path).unwrap())).unwrap();
}