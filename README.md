<div align="center">

# 🔍 VPS Inspector Professional

**Diagnostik VPS komprehensif berbasis Rust — laporan DOCX/PDF, analisis AI, bilingual.**

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![License](https://img.shields.io/github/license/akaanakbaik/vps-inspec?color=blue)](LICENSE)
[![Release](https://img.shields.io/github/v/release/akaanakbaik/vps-inspec?logo=github)](https://github.com/akaanakbaik/vps-inspec/releases)
[![Stars](https://img.shields.io/github/stars/akaanakbaik/vps-inspec?style=social)](https://github.com/akaanakbaik/vps-inspec/stargazers)
[![Last Commit](https://img.shields.io/github/last-commit/akaanakbaik/vps-inspec?color=brightgreen)](https://github.com/akaanakbaik/vps-inspec/commits)
[![Platform](https://img.shields.io/badge/Platform-Linux-lightgrey?logo=linux&logoColor=white)](https://www.linux.org)
[![Code Size](https://img.shields.io/github/languages/code-size/akaanakbaik/vps-inspec)](https://github.com/akaanakbaik/vps-inspec)

</div>

---

## ✨ Fitur Utama

| Kategori | Deskripsi |
|---|---|
| 🔍 **Scan Menyeluruh** | Sistem, hardware, storage, network, security, performance, software, logs |
| 📄 **Laporan Profesional** | Export ke **DOCX** dan **PDF** secara otomatis |
| 🌐 **Bilingual** | Antarmuka interaktif dalam **English** dan **Indonesia** |
| 🤖 **AI Diagnostics** | Rekomendasi & analisis lanjutan via NVIDIA NIM |
| ⚡ **Performa Tinggi** | Dibangun dengan Rust — cepat, ringan, zero-overhead |

---

## 🚀 Quick Start

### Opsi 1 — `start.sh` (Direkomendasikan)

> Cara paling mudah. Script mendeteksi otomatis apakah binary sudah ada, dan membangunnya jika belum.

```bash
git clone https://github.com/akaanakbaik/vps-inspec.git
cd vps-inspec
bash start.sh
```

#### Opsi `start.sh`

| Perintah | Deskripsi |
|---|---|
| `bash start.sh` | Jalankan langsung (auto-build jika belum ada binary) |
| `bash start.sh --update` | Pull update terbaru → rebuild → jalankan |
| `bash start.sh --build` | Hanya build ulang tanpa langsung menjalankan |
| `bash start.sh --help` | Tampilkan panduan penggunaan |

---

### Opsi 2 — One-Click Install (install ke sistem)

```bash
curl -fsSL https://raw.githubusercontent.com/akaanakbaik/vps-inspec/main/install.sh | bash
```

Setelah install selesai, jalankan dari mana saja:

```bash
vps-inspec
```

---

### Opsi 3 — Manual (dari source)

**Prasyarat:**

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh && source ~/.cargo/env

# Dependensi sistem (Ubuntu/Debian)
sudo apt update && sudo apt install -y build-essential pkg-config libssl-dev git
```

**Build & jalankan:**

```bash
git clone https://github.com/akaanakbaik/vps-inspec.git
cd vps-inspec
cargo build --release
./target/release/vps-inspec
```

---

## 🔄 Update

```bash
# Via start.sh (termudah)
bash start.sh --update

# Atau manual
cd vps-inspec && git pull && cargo build --release
```

---

## 📁 Output Laporan

Setiap sesi scan menghasilkan folder laporan bertanggal:

```
vps_report_YYYYMMDD_HHMMSS/
├── report.docx
└── report.pdf
```

Format dipilih secara interaktif saat program berjalan.

---

## 🧱 Struktur Proyek

```
vps-inspec/
├── start.sh             # Smart launcher (entry point utama)
├── install.sh           # One-click system installer
├── src/
│   ├── main.rs          # Entry point CLI interaktif
│   ├── bin/
│   │   └── update.rs    # Updater binary
│   ├── preflight/       # Pengecekan dependensi & environment
│   ├── collector/       # Kolektor metrik multi-kategori
│   ├── report/          # Generator DOCX/PDF
│   ├── translator/      # Dukungan bahasa EN/ID
│   ├── ai/              # Integrasi AI diagnostics
│   └── utils/           # Utilitas umum
└── Cargo.toml
```

---

## 📝 Lisensi

Dirilis di bawah **MIT License** — lihat file [LICENSE](LICENSE).

---

## 🙏 Credits

- [NVIDIA NIM](https://build.nvidia.com) — infrastruktur AI untuk analisis & rekomendasi
- [Rust](https://www.rust-lang.org) ecosystem & community
- Seluruh kontributor proyek ini

---

<div align="center">
  <sub>Made with ❤️ by <a href="https://github.com/akaanakbaik">akaanakbaik</a></sub>
</div>
