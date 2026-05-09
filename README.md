# 🚀 VPS Inspector Professional

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-linux-lightgrey.svg)](https://www.linux.org)
[![GitHub release](https://img.shields.io/github/v/release/akaanakbaik/vps-inspec)](https://github.com/akaanakbaik/vps-inspec/releases)

**VPS Inspector Professional** adalah tools diagnostik VPS berbasis Rust dengan laporan profesional (DOCX/PDF), analisis multi-kategori, dan integrasi AI.

## ✨ Highlights

- 🔍 Scan menyeluruh: sistem, hardware, storage, network, security, performance, software, logs
- 📄 Output laporan profesional: **DOCX** dan **PDF**
- 🌐 Dukungan bahasa: **English** dan **Indonesia**
- 🤖 AI-assisted diagnostics untuk rekomendasi dan analisis lanjutan
- 🧭 Interactive CLI yang ramah untuk operator/server admin

## 📦 Prasyarat

- Linux (direkomendasikan Ubuntu/Debian)
- Rust toolchain (stable)
- Paket build dasar

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Ubuntu/Debian dependencies
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev
```

## ⚙️ Instalasi

```bash
git clone https://github.com/akaanakbaik/vps-inspec.git
cd vps-inspec
cargo build --release
sudo cp target/release/vps-inspec /usr/local/bin/
```

## ▶️ Menjalankan

```bash
# Jalankan binary yang sudah di-install
vps-inspec

# Atau langsung dari source
cargo run --release
```

> Saat ini alur utama berjalan dalam mode interaktif (pemilihan bahasa, konfirmasi scan, dan format output laporan).

## 🔄 Update

Project menyediakan updater binary terpisah:

```bash
# Build updater
cargo build --release --bin vps-inspec-update

# Jalankan updater
./target/release/vps-inspec-update
```

## 📁 Output

Setiap eksekusi membuat folder laporan seperti:

```text
vps_report_YYYYMMDD_HHMMSS/
├── report.docx
└── report.pdf
```

(tergantung format yang dipilih di prompt interaktif)

## 🧱 Struktur Proyek (ringkas)

```text
src/
├── main.rs          # Entry point interactive CLI
├── preflight/       # Dependency & environment check
├── collector/       # Kolektor metrik multi-kategori
├── report/          # Generator DOCX/PDF
├── translator/      # Dukungan bahasa EN/ID
└── ai/              # Integrasi AI diagnostics
```

## 📝 License

MIT License — lihat file [LICENSE](LICENSE).

## 🙏 Credits

- [NVIDIA NIM](https://build.nvidia.com) (infrastruktur AI untuk analisis/rekomendasi)
- Rust ecosystem & community
- Kontributor proyek ini
