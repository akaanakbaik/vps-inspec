use std::sync::OnceLock;
use std::collections::HashMap;

static CURRENT_LANG: OnceLock<String> = OnceLock::new();
static EN_STRINGS: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
static ID_STRINGS: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

fn init_en() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("welcome_message", "╔══════════════════════════════════════════════════════════════════════════════════╗\n║                         WELCOME TO VPS INSPECTOR PROFESSIONAL                         ║\n║              The Most Comprehensive VPS Diagnostic Tool with AI Integration           ║\n╚══════════════════════════════════════════════════════════════════════════════════════╝");
    m.insert("report_description", "This tool will perform a thorough examination of your VPS including:\n  • System hardware and architecture analysis\n  • Storage performance and health metrics\n  • Network configuration and security audit\n  • Running services and vulnerability detection\n  • AI-powered diagnostics and recommendations\n  • Professional report generation (DOCX/PDF)");
    m.insert("show_guide_prompt", "Would you like to see the detailed guide before scanning?");
    m.insert("guide_title", "📖 VPS INSPECTOR PROFESSIONAL - QUICK START GUIDE");
    m.insert("guide_step1", "1. DEPENDENCY CHECK: The tool will automatically verify all required system utilities");
    m.insert("guide_step2", "2. AI INTEGRATION: Uses NVIDIA AI models for intelligent error recovery and diagnostics");
    m.insert("guide_step3", "3. COMPREHENSIVE SCAN: Collects 150+ metrics across 8 categories");
    m.insert("guide_step4", "4. REAL-TIME PROGRESS: Visual progress bars show current scanning stage");
    m.insert("guide_step5", "5. PROFESSIONAL OUTPUT: Generates formatted Word documents and PDFs with charts");
    m.insert("start_scan_prompt", "Ready to start the comprehensive VPS scan?");
    m.insert("scan_cancelled", "Scan cancelled by user. Exiting...");
    m.insert("collecting_system_info", "Collecting system information...");
    m.insert("collecting_hardware_info", "Analyzing CPU and memory configuration...");
    m.insert("collecting_storage_info", "Scanning disk partitions and I/O statistics...");
    m.insert("collecting_network_info", "Mapping network interfaces and active connections...");
    m.insert("collecting_security_info", "Performing security audit and threat analysis...");
    m.insert("collecting_performance_info", "Measuring system performance metrics...");
    m.insert("collecting_software_info", "Detecting installed software and versions...");
    m.insert("analyzing_logs", "Processing system logs for error patterns...");
    m.insert("ai_diagnostics", "Running AI-powered diagnostic analysis...");
    m.insert("generating_report", "Compiling data and generating professional report...");
    m.insert("report_generation_title", "📄 REPORT GENERATION");
    m.insert("output_format_prompt", "Select output format for the professional report");
    m.insert("report_generated", "Report generated successfully!");
    m.insert("writing_document", "Writing document with embedded charts and tables...");
    m.insert("thank_you_message", "Thank you for using VPS Inspector Professional!\nFor support: https://github.com/akaanakbaik/vps-inspec");
    m.insert("error_ai_timeout", "AI service timeout. Falling back to rule-based analysis...");
    m.insert("error_ai_rate_limit", "AI rate limit exceeded. Retrying with exponential backoff...");
    m.insert("error_network_unstable", "Network connectivity unstable. Some features may be limited.");
    m.insert("info_ai_processing", "🧠 AI is analyzing your VPS configuration...");
    m.insert("info_ai_response", "🤖 AI Diagnostic Response:");
    m.insert("metric_cpu_model", "CPU Model");
    m.insert("metric_cpu_cores", "CPU Cores");
    m.insert("metric_cpu_threads", "CPU Threads");
    m.insert("metric_cpu_freq", "CPU Frequency");
    m.insert("metric_ram_total", "Total RAM");
    m.insert("metric_ram_used", "Used RAM");
    m.insert("metric_ram_available", "Available RAM");
    m.insert("metric_disk_total", "Total Disk Space");
    m.insert("metric_disk_used", "Used Disk Space");
    m.insert("metric_disk_free", "Free Disk Space");
    m.insert("metric_uptime", "System Uptime");
    m.insert("metric_load_avg", "Load Average");
    m.insert("security_ssh_root", "SSH Root Login");
    m.insert("security_ssh_password", "SSH Password Auth");
    m.insert("security_firewall", "Firewall Status");
    m.insert("security_failed_logins", "Failed Login Attempts");
    m.insert("recommendation_update_kernel", "Recommendation: Update Linux kernel for security patches");
    m.insert("recommendation_secure_ssh", "Recommendation: Disable SSH password authentication and use keys");
    m.insert("recommendation_enable_firewall", "Recommendation: Enable and configure firewall");
    m.insert("recommendation_cleanup_logs", "Recommendation: Clean up old log files to free space");
    m.insert("status_good", "GOOD");
    m.insert("status_warning", "WARNING");
    m.insert("status_critical", "CRITICAL");
    m.insert("chart_cpu_usage", "CPU Usage Distribution");
    m.insert("chart_memory_usage", "Memory Usage Distribution");
    m.insert("chart_disk_usage", "Disk Usage by Partition");
    m.insert("chart_network_traffic", "Network Traffic Analysis");
    m.insert("table_processes", "Top Processes by Resource Consumption");
    m.insert("table_ports", "Open Ports and Listening Services");
    m.insert("table_users", "System Users and Last Login");
    m.insert("section_summary", "EXECUTIVE SUMMARY");
    m.insert("section_hardware", "HARDWARE ANALYSIS");
    m.insert("section_storage", "STORAGE AND FILESYSTEM");
    m.insert("section_network", "NETWORK CONFIGURATION");
    m.insert("section_security", "SECURITY AUDIT");
    m.insert("section_performance", "PERFORMANCE METRICS");
    m.insert("section_software", "SOFTWARE INVENTORY");
    m.insert("section_recommendations", "AI RECOMMENDATIONS");
    m.insert("severity_low", "Low");
    m.insert("severity_medium", "Medium");
    m.insert("severity_high", "High");
    m.insert("severity_critical", "Critical");
    m
}

fn init_id() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("welcome_message", "╔══════════════════════════════════════════════════════════════════════════════════╗\n║                    SELAMAT DATANG DI VPS INSPECTOR PROFESSIONAL                     ║\n║              Tools Diagnostik VPS Paling Komprehensif dengan AI Integration         ║\n╚══════════════════════════════════════════════════════════════════════════════════════╝");
    m.insert("report_description", "Tools ini akan melakukan pemeriksaan menyeluruh pada VPS Anda meliputi:\n  • Analisis hardware dan arsitektur sistem\n  • Metrik performa dan kesehatan penyimpanan\n  • Konfigurasi jaringan dan audit keamanan\n  • Layanan berjalan dan deteksi kerentanan\n  • Diagnostik bertenaga AI dan rekomendasi\n  • Pembuatan laporan profesional (DOCX/PDF)");
    m.insert("show_guide_prompt", "Apakah Anda ingin melihat panduan detail sebelum memindai?");
    m.insert("guide_title", "📖 VPS INSPECTOR PROFESSIONAL - PANDUAN CEPAT");
    m.insert("guide_step1", "1. PEMERIKSAAN DEPENDENSI: Tools akan memverifikasi semua utilitas sistem yang diperlukan");
    m.insert("guide_step2", "2. INTEGRASI AI: Menggunakan model AI NVIDIA untuk pemulihan error dan diagnostik cerdas");
    m.insert("guide_step3", "3. PEMINDAIAN KOMPREHENSIF: Mengumpulkan 150+ metrik dari 8 kategori");
    m.insert("guide_step4", "4. PROGRES REAL-TIME: Bar progres visual menunjukkan tahap pemindaian saat ini");
    m.insert("guide_step5", "5. OUTPUT PROFESIONAL: Menghasilkan dokumen Word dan PDF terformat dengan grafik");
    m.insert("start_scan_prompt", "Siap memulai pemindaian VPS komprehensif?");
    m.insert("scan_cancelled", "Pemindaian dibatalkan. Keluar...");
    m.insert("collecting_system_info", "Mengumpulkan informasi sistem...");
    m.insert("collecting_hardware_info", "Menganalisis konfigurasi CPU dan memori...");
    m.insert("collecting_storage_info", "Memindai partisi disk dan statistik I/O...");
    m.insert("collecting_network_info", "Memetakan antarmuka jaringan dan koneksi aktif...");
    m.insert("collecting_security_info", "Melakukan audit keamanan dan analisis ancaman...");
    m.insert("collecting_performance_info", "Mengukur metrik performa sistem...");
    m.insert("collecting_software_info", "Mendeteksi software terinstall dan versi...");
    m.insert("analyzing_logs", "Memproses log sistem untuk pola error...");
    m.insert("ai_diagnostics", "Menjalankan analisis diagnostik bertenaga AI...");
    m.insert("generating_report", "Mengompilasi data dan membuat laporan profesional...");
    m.insert("report_generation_title", "📄 PEMBUATAN LAPORAN");
    m.insert("output_format_prompt", "Pilih format output untuk laporan profesional");
    m.insert("report_generated", "Laporan berhasil dibuat!");
    m.insert("writing_document", "Menulis dokumen dengan grafik dan tabel...");
    m.insert("thank_you_message", "Terima kasih telah menggunakan VPS Inspector Professional!\nUntuk dukungan: https://github.com/akaanakbaik/vps-inspec");
    m.insert("error_ai_timeout", "Layanan AI timeout. Menggunakan analisis berbasis aturan...");
    m.insert("error_ai_rate_limit", "Batas rate AI terlampaui. Mencoba ulang dengan backoff eksponensial...");
    m.insert("error_network_unstable", "Koneksi jaringan tidak stabil. Beberapa fitur mungkin terbatas.");
    m.insert("info_ai_processing", "🧠 AI sedang menganalisis konfigurasi VPS Anda...");
    m.insert("info_ai_response", "🤖 Respons Diagnostik AI:");
    m.insert("metric_cpu_model", "Model CPU");
    m.insert("metric_cpu_cores", "Core CPU");
    m.insert("metric_cpu_threads", "Thread CPU");
    m.insert("metric_cpu_freq", "Frekuensi CPU");
    m.insert("metric_ram_total", "Total RAM");
    m.insert("metric_ram_used", "RAM Terpakai");
    m.insert("metric_ram_available", "RAM Tersedia");
    m.insert("metric_disk_total", "Total Ruang Disk");
    m.insert("metric_disk_used", "Ruang Disk Terpakai");
    m.insert("metric_disk_free", "Ruang Disk Tersisa");
    m.insert("metric_uptime", "Uptime Sistem");
    m.insert("metric_load_avg", "Rata-rata Beban");
    m.insert("security_ssh_root", "Login Root SSH");
    m.insert("security_ssh_password", "Auth Password SSH");
    m.insert("security_firewall", "Status Firewall");
    m.insert("security_failed_logins", "Upaya Login Gagal");
    m.insert("recommendation_update_kernel", "Rekomendasi: Update kernel Linux untuk patch keamanan");
    m.insert("recommendation_secure_ssh", "Rekomendasi: Nonaktifkan autentikasi password SSH dan gunakan key");
    m.insert("recommendation_enable_firewall", "Rekomendasi: Aktifkan dan konfigurasi firewall");
    m.insert("recommendation_cleanup_logs", "Rekomendasi: Bersihkan file log lama untuk menghemat ruang");
    m.insert("status_good", "BAIK");
    m.insert("status_warning", "PERINGATAN");
    m.insert("status_critical", "KRITIS");
    m.insert("chart_cpu_usage", "Distribusi Penggunaan CPU");
    m.insert("chart_memory_usage", "Distribusi Penggunaan Memori");
    m.insert("chart_disk_usage", "Penggunaan Disk per Partisi");
    m.insert("chart_network_traffic", "Analisis Lalu Lintas Jaringan");
    m.insert("table_processes", "Proses Teratas berdasarkan Konsumsi Sumber Daya");
    m.insert("table_ports", "Port Terbuka dan Layanan Mendengarkan");
    m.insert("table_users", "Pengguna Sistem dan Login Terakhir");
    m.insert("section_summary", "RINGKASAN EKSEKUTIF");
    m.insert("section_hardware", "ANALISIS HARDWARE");
    m.insert("section_storage", "PENYIMPANAN DAN FILESYSTEM");
    m.insert("section_network", "KONFIGURASI JARINGAN");
    m.insert("section_security", "AUDIT KEAMANAN");
    m.insert("section_performance", "METRIK PERFORMA");
    m.insert("section_software", "INVENTARIS PERANGKAT LUNAK");
    m.insert("section_recommendations", "REKOMENDASI AI");
    m.insert("severity_low", "Rendah");
    m.insert("severity_medium", "Sedang");
    m.insert("severity_high", "Tinggi");
    m.insert("severity_critical", "Kritis");
    m
}

pub fn set_language(lang: &str) {
    let _ = CURRENT_LANG.set(lang.to_string());
    let _ = EN_STRINGS.set(init_en());
    let _ = ID_STRINGS.set(init_id());
}

pub fn t(key: &str) -> String {
    let lang = CURRENT_LANG.get().map(|s| s.as_str()).unwrap_or("EN");
    
    let result = if lang == "ID" {
        ID_STRINGS.get().and_then(|m| m.get(key)).copied()
    } else {
        EN_STRINGS.get().and_then(|m| m.get(key)).copied()
    };
    
    result.unwrap_or_else(|| {
        eprintln!("Missing translation for key: {}", key);
        key
    }).to_string()
}