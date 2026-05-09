use std::process;
use std::sync::Arc;
use tokio::sync::Mutex;
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use chrono::Local;

mod preflight;
mod collector;
mod report;
mod utils;
mod translator;
mod ai;

#[tokio::main]
async fn main() {
    println!("{}", "
╔══════════════════════════════════════════════════════════════════════════════════╗
║                                                                                  ║
║   ██╗   ██╗██████╗ ███████╗    ██╗███╗   ██╗███████╗██████╗ ███████╗ ██████╗████████╗
║   ██║   ██║██╔══██╗██╔════╝    ██║████╗  ██║██╔════╝██╔══██╗██╔════╝██╔════╝╚══██╔══╝
║   ██║   ██║██████╔╝███████╗    ██║██╔██╗ ██║█████╗  ██████╔╝███████╗██║        ██║   
║   ╚██╗ ██╔╝██╔═══╝ ╚════██║    ██║██║╚██╗██║██╔══╝  ██╔══██╗╚════██║██║        ██║   
║    ╚████╔╝ ██║     ███████║    ██║██║ ╚████║███████╗██║  ██║███████║╚██████╗   ██║   
║     ╚═══╝  ╚═╝     ╚══════╝    ╚═╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝╚══════╝ ╚═════╝   ╚═╝   
║                                                                                  ║
║                    PROFESSIONAL VPS INSPECTOR WITH AI DIAGNOSTICS                ║
║                                    v1.0.0                                        ║
╚══════════════════════════════════════════════════════════════════════════════════╝
");
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap());
    pb.set_message("Initializing system components...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    pb.set_message("Checking dependencies...");
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    let preflight_result = preflight::run_preflight();
    
    if let Err(e) = preflight_result {
        pb.finish_with_message(format!("{}", "Preflight check failed!".red()));
        eprintln!("{} {}", "✗".red(), e);
        process::exit(1);
    }
    
    pb.finish_with_message("✓ System ready".green().to_string());
    println!();
    
    let lang_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("🌐 Select language / Pilih bahasa")
        .default(0)
        .items(&["🇬🇧 English", "🇮🇩 Indonesia"])
        .interact()
        .unwrap();
    
    let lang = if lang_selection == 0 { "EN" } else { "ID" };
    
    translator::set_language(lang);
    
    println!("\n{}", translator::t("welcome_message"));
    println!("{}", translator::t("report_description"));
    
    let show_guide = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(translator::t("show_guide_prompt"))
        .default(false)
        .interact()
        .unwrap();
    
    if show_guide {
        println!("\n{}", "═".repeat(70).cyan());
        println!("{}", translator::t("guide_title").bold().yellow());
        println!("{}", translator::t("guide_step1"));
        println!("{}", translator::t("guide_step2"));
        println!("{}", translator::t("guide_step3"));
        println!("{}", translator::t("guide_step4"));
        println!("{}", translator::t("guide_step5"));
        println!("{}", "═".repeat(70).cyan());
        println!();
    }
    
    let start_scan = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(translator::t("start_scan_prompt"))
        .default(true)
        .interact()
        .unwrap();
    
    if !start_scan {
        println!("{}", translator::t("scan_cancelled").yellow());
        process::exit(0);
    }
    
    println!("\n{}", "🚀 Starting comprehensive VPS scan...".green().bold());
    println!("{}", format!("📅 {}", Local::now().format("%Y-%m-%d %H:%M:%S")).dimmed());
    println!("{}", "═".repeat(70));
    
    let scan_pb = ProgressBar::new(100);
    scan_pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("█▓▒░"));
    
    scan_pb.set_message(translator::t("collecting_system_info"));
    
    let collector_manager = collector::CollectorManager::new();
    
    scan_pb.inc(10);
    scan_pb.set_message(translator::t("collecting_hardware_info"));
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    scan_pb.inc(10);
    scan_pb.set_message(translator::t("collecting_storage_info"));
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    scan_pb.inc(10);
    scan_pb.set_message(translator::t("collecting_network_info"));
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    scan_pb.inc(10);
    scan_pb.set_message(translator::t("collecting_security_info"));
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    scan_pb.inc(10);
    scan_pb.set_message(translator::t("collecting_performance_info"));
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    scan_pb.inc(10);
    scan_pb.set_message(translator::t("collecting_software_info"));
    tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
    
    scan_pb.inc(10);
    scan_pb.set_message(translator::t("analyzing_logs"));
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    scan_pb.inc(10);
    scan_pb.set_message(translator::t("ai_diagnostics"));
    tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
    
    scan_pb.inc(10);
    scan_pb.set_message(translator::t("generating_report"));
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    scan_pb.inc(10);
    scan_pb.finish_with_message("✓ Scan completed!".green().to_string());
    
    println!("\n{}", "═".repeat(70));
    println!("{}", translator::t("report_generation_title").bold().green());
    
    let output_format = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(translator::t("output_format_prompt"))
        .default(0)
        .items(&["📄 Microsoft Word (.docx)", "📑 PDF Document (.pdf)", "📊 Both formats"])
        .interact()
        .unwrap();
    
    let output_dir = std::path::PathBuf::from(format!("vps_report_{}", Local::now().format("%Y%m%d_%H%M%S")));
    std::fs::create_dir_all(&output_dir).unwrap();
    
    let final_pb = ProgressBar::new(100);
    final_pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{bar:40.cyan/blue}] {msg}")
        .unwrap());
    
    final_pb.set_message(translator::t("writing_document"));
    
    match output_format {
        0 => {
            report::generate_docx_report(&output_dir, lang).await;
            final_pb.inc(100);
            final_pb.finish_with_message(format!("✓ {} report.docx", translator::t("report_generated")));
            println!("{} {}", "📁".cyan(), output_dir.join("report.docx").display());
        }
        1 => {
            report::generate_pdf_report(&output_dir, lang).await;
            final_pb.inc(100);
            final_pb.finish_with_message(format!("✓ {} report.pdf", translator::t("report_generated")));
            println!("{} {}", "📁".cyan(), output_dir.join("report.pdf").display());
        }
        2 => {
            report::generate_docx_report(&output_dir, lang).await;
            report::generate_pdf_report(&output_dir, lang).await;
            final_pb.inc(100);
            final_pb.finish_with_message(format!("✓ {} both formats", translator::t("report_generated")));
            println!("{} 📄 {}", "📁".cyan(), output_dir.join("report.docx").display());
            println!("{} 📑 {}", "📁".cyan(), output_dir.join("report.pdf").display());
        }
        _ => {}
    }
    
    println!("\n{}", "╔════════════════════════════════════════════════════════════════════════════╗".green());
    println!("{}", "║                                    DONE                                    ║".green());
    println!("{}", "╚════════════════════════════════════════════════════════════════════════════╝".green());
    println!("\n{}", translator::t("thank_you_message").dimmed());
}