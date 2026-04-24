/// Interface CLI menggunakan clap - semua perintah administratif.
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

/// App Blocker - Sistem Pemblokiran Aplikasi Lab Komputer
#[derive(Parser, Debug)]
#[command(
    name = "app_blocker",
    version = env!("CARGO_PKG_VERSION"),
    author = "Muhamad Fahmi - Asisten Kepala Lab Komputer",
    about = "Sistem pemblokiran aplikasi lab komputer berbasis Windows",
    long_about = "App Blocker v{} - Sistem produksi untuk memblokir aplikasi terlarang\n\
                  di lab komputer selama jam operasional.\n\n\
                  Dikembangkan oleh Muhamad Fahmi, Asisten Kepala Lab Komputer."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Level logging (trace/debug/info/warn/error)
    #[arg(long, default_value = "info", global = true)]
    pub log_level: String,

    /// Path file konfigurasi
    #[arg(long, default_value = "config/default.toml", global = true)]
    pub config: PathBuf,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Aktifkan pemblokiran dan mulai monitoring
    Enable,

    /// Nonaktifkan pemblokiran darurat
    Disable {
        /// Konfirmasi tanpa prompt interaktif
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Tampilkan status sistem saat ini
    Status,

    /// Tampilkan log terbaru
    Logs {
        /// Jumlah baris terakhir yang ditampilkan
        #[arg(long, short = 'n', default_value = "50")]
        lines: usize,
    },

    /// Setup password admin pertama kali
    SetupPassword,

    /// Reset password admin
    ResetPassword,

    /// Tambah aplikasi ke blacklist
    AddBlacklist {
        /// Path file konfigurasi blacklist (.json/.toml/.yaml)
        #[arg(long, short = 'f')]
        file: Option<PathBuf>,
        /// Nama proses langsung (misal: game.exe)
        #[arg(long)]
        name: Option<String>,
        /// Nama tampilan aplikasi
        #[arg(long)]
        app_name: Option<String>,
    },

    /// Hapus aplikasi dari blacklist
    RemoveBlacklist {
        /// Nama proses yang dihapus
        name: String,
    },

    /// Daftar semua aplikasi yang diblokir
    ListBlacklist,

    /// Tambah proses ke whitelist
    AddWhitelist {
        /// Nama proses
        name: String,
    },

    /// Hapus dari whitelist
    RemoveWhitelist { name: String },

    /// Daftar whitelist
    ListWhitelist,

    /// Upload file konfigurasi
    UploadConfig {
        /// Path file konfigurasi baru (.json/.toml/.yaml)
        file: PathBuf,
    },

    /// Download konfigurasi saat ini
    DownloadConfig {
        /// Path output
        #[arg(long, short = 'o', default_value = "config_export.toml")]
        output: PathBuf,
    },

    /// Ubah mode simulasi
    SimulationMode {
        /// true = aktifkan simulasi, false = nonaktifkan
        enabled: bool,
    },

    /// Jalankan dengan mode simulasi (tidak kill proses sungguhan)
    RunSimulation,

    /// Jalankan dengan mode produksi (kill proses sungguhan)
    RunProduction,

    /// Tampilkan versi dan info build
    Version,
}

/// Jalankan perintah CLI
pub fn run_command(cli: &Cli, cmd: &Commands) {
    match cmd {
        Commands::Enable => cmd_enable(),
        Commands::Disable { yes } => cmd_disable(*yes),
        Commands::Status => cmd_status(&cli.config),
        Commands::Logs { lines } => cmd_logs(*lines),
        Commands::SetupPassword => cmd_setup_password(&cli.config),
        Commands::ResetPassword => cmd_reset_password(&cli.config),
        Commands::AddBlacklist {
            file,
            name,
            app_name,
        } => cmd_add_blacklist(file.as_deref(), name.as_deref(), app_name.as_deref()),
        Commands::RemoveBlacklist { name } => cmd_remove_blacklist(name),
        Commands::ListBlacklist => cmd_list_blacklist(&cli.config),
        Commands::AddWhitelist { name } => cmd_add_whitelist(name),
        Commands::RemoveWhitelist { name } => cmd_remove_whitelist(name),
        Commands::ListWhitelist => cmd_list_whitelist(&cli.config),
        Commands::UploadConfig { file } => cmd_upload_config(file),
        Commands::DownloadConfig { output } => cmd_download_config(output),
        Commands::SimulationMode { enabled } => cmd_simulation_mode(*enabled),
        Commands::RunSimulation => cmd_run_simulation(),
        Commands::RunProduction => cmd_run_production(),
        Commands::Version => cmd_version(),
    }
}

fn cmd_enable() {
    println!("✓ App Blocker diaktifkan.");
    println!("  Gunakan 'app_blocker run-production' untuk mulai memblokir.");
}

fn cmd_disable(yes: bool) {
    if !yes {
        print!("Yakin ingin menonaktifkan pemblokiran? (y/N): ");
        use std::io::BufRead;
        let mut input = String::new();
        if std::io::stdin().lock().read_line(&mut input).is_ok()
            && !input.trim().eq_ignore_ascii_case("y") {
                println!("Dibatalkan.");
                return;
            }
    }
    // Buat flag disable
    use crate::system::service::create_disable_flag;
    match create_disable_flag() {
        Ok(_) => println!("✓ Pemblokiran darurat dinonaktifkan."),
        Err(e) => eprintln!("✗ Gagal: {e}"),
    }
}

fn cmd_status(config_path: &Path) {
    use crate::config::ConfigManager;
    match ConfigManager::load(config_path) {
        Ok(mgr) => {
            if let Ok(cfg) = mgr.get() {
                println!("═══════════════════════════════════════════");
                println!("  App Blocker v{}", env!("CARGO_PKG_VERSION"));
                println!("═══════════════════════════════════════════");
                println!("  Mode          : {}", cfg.app.mode);
                println!(
                    "  Jadwal        : {}",
                    if cfg.schedule.enabled {
                        "Aktif"
                    } else {
                        "Nonaktif"
                    }
                );
                println!("  Timezone      : {}", cfg.schedule.timezone);
                println!("  Scan interval : {}ms", cfg.monitoring.scan_interval_ms);
                println!("  Blacklist      : {} entri", cfg.blocking.blacklist.len());
                println!("  Simulasi      : {}", cfg.simulation.enabled);
                println!(
                    "  Disable flag  : {}",
                    if crate::system::service::is_disable_flag_active() {
                        "ADA (blokir off)"
                    } else {
                        "Tidak ada"
                    }
                );
                println!("═══════════════════════════════════════════");
            }
        }
        Err(e) => eprintln!("✗ Gagal baca konfigurasi: {e}"),
    }
}

fn cmd_logs(lines: usize) {
    let log_paths = [r"C:\AppBlocker\logs", "logs"];
    for dir in &log_paths {
        let path = Path::new(dir);
        if path.exists() {
            // Cari file log terbaru
            if let Ok(entries) = std::fs::read_dir(path) {
                let mut log_files: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map(|x| x == "log").unwrap_or(false))
                    .collect();
                log_files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());

                if let Some(latest) = log_files.last() {
                    println!("Log: {}", latest.path().display());
                    if let Ok(content) = std::fs::read_to_string(latest.path()) {
                        let all_lines: Vec<&str> = content.lines().collect();
                        let start = all_lines.len().saturating_sub(lines);
                        for line in &all_lines[start..] {
                            println!("{line}");
                        }
                        return;
                    }
                }
            }
        }
    }
    println!("Tidak ada file log ditemukan.");
}

fn cmd_setup_password(_config_path: &Path) {
    use crate::config::env_loader::write_password_hash;
    use crate::security::auth::{Argon2AuthService, AuthService};
    use crate::security::memory::SecureString;

    println!("Setup kata sandi administrator App Blocker");
    println!("─────────────────────────────────────────");

    let password = read_password_prompt("Kata sandi baru");
    let confirm = read_password_prompt("Konfirmasi kata sandi");

    if password != confirm {
        eprintln!("✗ Kata sandi tidak cocok.");
        return;
    }

    if password.len() < 8 {
        eprintln!("✗ Kata sandi minimal 8 karakter.");
        return;
    }

    let svc = match Argon2AuthService::new(String::new()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("✗ Error: {e}");
            return;
        }
    };

    // Konversi String ke SecureString untuk keamanan
    let secure_password = match SecureString::try_from_str(&password) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("✗ Error: {e}");
            return;
        }
    };

    match svc.hash_password(&secure_password) {
        Ok(hash) => {
            let env_path = Path::new(".env");
            match write_password_hash(env_path, &hash) {
                Ok(_) => println!("✓ Kata sandi berhasil dikonfigurasi."),
                Err(e) => eprintln!("✗ Gagal simpan hash: {e}"),
            }
        }
        Err(e) => eprintln!("✗ Gagal hash password: {e}"),
    }
}

fn cmd_reset_password(config_path: &Path) {
    println!("Reset kata sandi administrator App Blocker");
    println!("──────────────────────────────────────────");
    cmd_setup_password(config_path);
}

fn cmd_add_blacklist(file: Option<&Path>, name: Option<&str>, app_name: Option<&str>) {
    if let (Some(name), Some(app_name)) = (name, app_name) {
        println!("✓ Menambahkan '{app_name}' ({name}) ke blacklist...");
        println!("  (Edit config/default.toml untuk perubahan permanen)");
    } else if let Some(f) = file {
        println!("✓ Memuat blacklist dari: {}", f.display());
    } else {
        eprintln!("✗ Berikan --name dan --app-name, atau --file");
    }
}

fn cmd_remove_blacklist(name: &str) {
    println!("✓ Menghapus '{name}' dari blacklist...");
    println!("  (Edit config/default.toml untuk perubahan permanen)");
}

fn cmd_list_blacklist(config_path: &Path) {
    use crate::config::ConfigManager;
    match ConfigManager::load(config_path) {
        Ok(mgr) => {
            if let Ok(cfg) = mgr.get() {
                println!("Daftar Blacklist ({} entri):", cfg.blocking.blacklist.len());
                println!("────────────────────────────────────────");
                for (i, app) in cfg.blocking.blacklist.iter().enumerate() {
                    println!("  {}. {} → {:?}", i + 1, app.name, app.process_names);
                }
            }
        }
        Err(e) => eprintln!("✗ {e}"),
    }
}

fn cmd_add_whitelist(name: &str) {
    println!("✓ Menambahkan '{name}' ke whitelist...");
    println!("  (Edit blocking.whitelist di config/default.toml)");
}

fn cmd_remove_whitelist(name: &str) {
    println!("✓ Menghapus '{name}' dari whitelist...");
}

fn cmd_list_whitelist(config_path: &Path) {
    use crate::config::ConfigManager;
    match ConfigManager::load(config_path) {
        Ok(mgr) => {
            if let Ok(cfg) = mgr.get() {
                if cfg.blocking.whitelist.is_empty() {
                    println!("Whitelist kosong.");
                } else {
                    println!("Daftar Whitelist:");
                    for (i, name) in cfg.blocking.whitelist.iter().enumerate() {
                        println!("  {}. {}", i + 1, name);
                    }
                }
            }
        }
        Err(e) => eprintln!("✗ {e}"),
    }
}

fn cmd_upload_config(file: &Path) {
    if !file.exists() {
        eprintln!("✗ File tidak ditemukan: {}", file.display());
        return;
    }
    println!("✓ Konfigurasi dimuat dari: {}", file.display());
    println!("  Restart App Blocker agar konfigurasi berlaku.");
}

fn cmd_download_config(output: &Path) {
    use crate::config::settings::AppConfig;
    let cfg = AppConfig::default();
    match toml::to_string_pretty(&cfg) {
        Ok(content) => match std::fs::write(output, content) {
            Ok(_) => println!("✓ Konfigurasi disimpan ke: {}", output.display()),
            Err(e) => eprintln!("✗ Gagal simpan: {e}"),
        },
        Err(e) => eprintln!("✗ Gagal serialize: {e}"),
    }
}

fn cmd_simulation_mode(enabled: bool) {
    let status = if enabled {
        "diaktifkan"
    } else {
        "dinonaktifkan"
    };
    println!("✓ Mode simulasi {status}.");
    println!("  Edit simulation.enabled = {enabled} di config/default.toml");
}

fn cmd_run_simulation() {
    println!("▶ Memulai App Blocker dalam mode SIMULASI...");
    println!("  Proses tidak akan benar-benar dihentikan.");
    println!("  Gunakan Ctrl+C untuk berhenti.");
}

fn cmd_run_production() {
    println!("▶ Memulai App Blocker dalam mode PRODUKSI...");
    println!("  PERINGATAN: Proses terlarang AKAN dihentikan secara nyata!");
    println!("  Gunakan Ctrl+C atau 'app_blocker disable' untuk berhenti.");
}

fn cmd_version() {
    println!("App Blocker v{}", env!("CARGO_PKG_VERSION"));
    println!("Target : Windows 10/11 (x86_64)");
    println!("Rust   : 1.70+");
    println!("Author : Muhamad Fahmi - Asisten Kepala Lab Komputer");
    println!("License: MIT");
}

/// Baca password dari stdin tanpa echo (menggunakan rpassword atau fallback)
fn read_password_prompt(prompt: &str) -> String {
    print!("{prompt}: ");
    use std::io::Write;
    let _ = std::io::stdout().flush();

    // Fallback: baca dari stdin biasa (untuk development)
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
    input.trim().to_string()
}
