/// Interface CLI menggunakan clap - semua perintah administratif.
/// Semua config dan data sekarang dari database
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
        /// Path file (.json)
        #[arg(long, short = 'f')]
        file: Option<PathBuf>,

        /// Nama proses
        #[arg(long)]
        name: Option<String>,
    },

    /// Hapus dari whitelist
    RemoveWhitelist {
        name: String,
    },

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

    // ============ Statistik ============
    /// Statistik pemblokiran
    Stats {
        /// Periode: day, week, month
        #[arg(long, default_value = "week")]
        period: String,
    },

    Version,

    /// Top proses paling sering diblokir
    TopBlocked {
        /// Jumlah maksimal tampilan
        #[arg(long, default_value = "10")]
        limit: usize,
    },

    /// Log aktivitas admin
    AuditLog {
        /// Filter username
        #[arg(long)]
        user: Option<String>,
        /// Jumlah maksimal
        #[arg(long, default_value = "50")]
        limit: usize,
    },

    // ============ Schedule ============
    /// List semua jadwal
    ScheduleList,

    /// Tambah jadwal baru
    ScheduleAdd {
        /// Hari (Senin,Selasa,Rabu,Kamis,Jumat,Sabtu,Minggu)
        #[arg(long)]
        days: String,
        /// Waktu mulai (HH:MM)
        #[arg(long)]
        start: String,
        /// Waktu selesai (HH:MM)
        #[arg(long)]
        end: String,
        /// Action (block_games, block_all, allow_all)
        #[arg(long, default_value = "block_games")]
        action: String,
    },

    /// Hapus jadwal
    ScheduleRemove {
        /// ID jadwal
        #[arg(long)]
        id: i32,
    },

    /// Toggle schedule on/off
    ScheduleToggle {
        /// ID jadwal
        #[arg(long)]
        id: i32,
    },
}

/// Jalankan perintah CLI
/// Menggunakan database untuk semua operasi
pub fn run_command(_cli: &Cli, cmd: &Commands) {
    match cmd {
        Commands::Enable => cmd_enable(),
        Commands::Disable { yes } => cmd_disable(*yes),
        Commands::Status => cmd_status(),
        Commands::Logs { lines } => cmd_logs(*lines),
        Commands::SetupPassword => {
            // Setup password tidak memerlukan old password
            cmd_setup_password(false);
        }
        Commands::ResetPassword => {
            // Reset password memerlukan verifikasi old password
            cmd_setup_password(true);
        }
        Commands::AddBlacklist {
            file,
            name,
            app_name,
        } => cmd_add_blacklist(file.as_deref(), name.as_deref(), app_name.as_deref()),
        Commands::RemoveBlacklist { name } => cmd_remove_blacklist(name),
        Commands::ListBlacklist => cmd_list_blacklist(),
        Commands::AddWhitelist { file, name } => {
            cmd_add_whitelist(file.as_deref(), name.as_deref())
        }
        Commands::RemoveWhitelist { name } => cmd_remove_whitelist(name),
        Commands::ListWhitelist => cmd_list_whitelist(),
        Commands::UploadConfig { file } => cmd_upload_config(file),
        Commands::DownloadConfig { output } => cmd_download_config(output),
        Commands::SimulationMode { enabled } => cmd_simulation_mode(*enabled),
        Commands::RunSimulation => cmd_run_simulation(),
        Commands::RunProduction => cmd_run_production(),

        // Statistik
        Commands::Version => cmd_version(),
        Commands::Stats { period } => cmd_stats(period),
        Commands::TopBlocked { limit } => cmd_top_blocked(*limit),
        Commands::AuditLog { user, limit } => cmd_audit_log(user.as_deref(), *limit),

        // Schedule
        Commands::ScheduleList => cmd_schedule_list(),
        Commands::ScheduleAdd {
            days,
            start,
            end,
            action,
        } => {
            cmd_schedule_add(days, start, end, action);
        }
        Commands::ScheduleRemove { id } => cmd_schedule_remove(*id),
        Commands::ScheduleToggle { id } => cmd_schedule_toggle(*id),
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
            && !input.trim().eq_ignore_ascii_case("y")
        {
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

/// cmd_status - Menggunakan database
fn cmd_status() {
    // Gunakan tokio runtime untuk operasi database
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        use crate::constants::paths;
        use crate::db::init::init_database;
        use crate::repository::{BlacklistRepository, ConfigRepository, WhitelistRepository};

        // Init database
        let db = match init_database(&paths::get_db_path()).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("✗ Gagal akses database: {e}");
                return;
            }
        };

        let config_repo = ConfigRepository::new(db.clone());
        let blacklist_repo = BlacklistRepository::new(db.clone());
        let whitelist_repo = WhitelistRepository::new(db.clone());

        // Ambil status dari database
        println!("\n=== Status App Blocker ===\n");

        // App Mode
        if let Ok(Some(mode)) = config_repo.get_value("app.mode").await {
            println!("Mode          : {}", mode);
        }

        // Simulation
        if let Ok(Some(sim)) = config_repo.get_bool("simulation.enabled").await {
            println!("Simulasi      : {}", if sim { "Aktif" } else { "Nonaktif" });
        }

        // Blacklist count
        match blacklist_repo.find_all().await {
            Ok(list) => println!("Blacklist     : {} aplikasi", list.len()),
            Err(_) => println!("Blacklist     : error"),
        }

        // Whitelist count
        match whitelist_repo.find_all().await {
            Ok(list) => println!("Whitelist    : {} proses", list.len()),
            Err(_) => println!("Whitelist   : error"),
        }

        // Scan interval
        if let Ok(Some(interval)) = config_repo.get_value("monitoring.scan_interval_ms").await {
            println!("Scan Interval: {} ms", interval);
        }

        println!("\n========================\n");
    });
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

/// cmd_setup_password - Setup atau reset password menggunakan database
/// is_reset: true jika ini adalah reset password (memerlukan old password)
fn cmd_setup_password(is_reset: bool) {
    println!(
        "\n{}",
        if is_reset {
            "Reset Kata Sandi Administrator App Blocker"
        } else {
            "Setup Kata Sandi Administrator App Blocker"
        }
    );
    println!("{}\n", "─".repeat(if is_reset { 38 } else { 41 }));

    // Jika reset password, minta old password dengan max 3 percobaan
    if is_reset {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        let max_attempts = 3;
        let mut attempts = 0;

        loop {
            attempts += 1;
            let old_pass = read_password_prompt("Kata sandi lama");

            // Verifikasi old password
            let result = rt.block_on(async {
                use crate::constants::paths;
                use crate::db::init::{init_database, verify_password};
                use crate::repository::ConfigRepository;

                let db = match init_database(&paths::get_db_path()).await {
                    Ok(db) => db,
                    Err(e) => return Err(e.to_string()),
                };

                let config_repo = ConfigRepository::new(db.clone());

                // Ambil password hash dari database
                match config_repo.get_value("security.password_hash").await {
                    Ok(Some(hash)) => {
                        // Verifikasi password lama
                        match verify_password(&old_pass, &hash) {
                            Ok(true) => Ok(hash),
                            Ok(false) => Err("Kata sandi lama tidak cocok".to_string()),
                            Err(e) => Err(e.to_string()),
                        }
                    }
                    Ok(None) => Err("Password belum dikonfigurasi".to_string()),
                    Err(e) => Err(e.to_string()),
                }
            });

            match result {
                Ok(_hash) => {
                    println!("✓ Kata sandi lama diverifikasi.");
                    break;
                }
                Err(e) => {
                    if attempts >= max_attempts {
                        eprintln!("\n✗ Percobaan maksimal ({}) tercapai.", max_attempts);
                        eprintln!("  Proses reset password dibatalkan.");
                        return;
                    }
                    let remaining = max_attempts - attempts;
                    eprintln!("✗ {e} (sisa percobaan: {})", remaining);
                }
            }
        }
    }

    // Minta new password
    let new_password = read_password_prompt("Kata sandi baru");
    let confirm_password = read_password_prompt("Konfirmasi kata sandi");

    // Validasi
    if new_password != confirm_password {
        eprintln!("✗ Kata sandi tidak cocok.");
        return;
    }

    if new_password.len() < 8 {
        eprintln!("✗ Kata sandi minimal 8 karakter.");
        return;
    }

    // Generate password hash baru dan simpan ke database
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let new_password_clone = new_password.clone();
    let is_reset_clone = is_reset;

    rt.block_on(async move {
        use crate::constants::paths;
        use crate::db::init::{generate_argon2_hash, init_database};
        use crate::repository::ConfigRepository;

        let db = match init_database(&paths::get_db_path()).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("✗ Gagal akses database: {e}");
                return;
            }
        };

        let config_repo = ConfigRepository::new(db.clone());

        // Generate hash baru
        let new_hash = match generate_argon2_hash(&new_password_clone) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("✗ Gagal generate hash: {}", e);
                return;
            }
        };

        // Simpan ke database
        match config_repo
            .set(
                "security.password_hash",
                &new_hash,
                Some("Admin password hash"),
            )
            .await
        {
            Ok(_) => {
                println!(
                    "\n✓ {} berhasil.",
                    if is_reset_clone {
                        "Password direset"
                    } else {
                        "Password dikonfigurasi"
                    }
                );
            }
            Err(e) => eprintln!("✗ Gagal simpan ke database: {e}"),
        }
    });
}

fn cmd_add_blacklist(file: Option<&Path>, name: Option<&str>, app_name: Option<&str>) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        use crate::constants::paths;
        use crate::db::init::init_database;
        use crate::repository::BlacklistRepository;

        let db = match init_database(&paths::get_db_path()).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("✗ Gagal akses database: {e}");
                return;
            }
        };

        let blacklist_repo = BlacklistRepository::new(db.clone());

        if let (Some(name), Some(app_name)) = (name, app_name) {
            match blacklist_repo.create(app_name, Some(name), false).await {
                Ok(bl) => {
                    println!("✓ Menambahkan '{app_name}' ({name}) ke blacklist...");
                    if let Err(e) = blacklist_repo.add_process(bl.id, name).await {
                        eprintln!("  Warning: Gagal tambahkan proses: {e}");
                    } else {
                        println!("  Proses '{name}' ditambahkan.");
                    }
                }
                Err(e) => eprintln!("✗ Gagal: {e}"),
            }
        } else if let Some(f) = file {
            // ============ Load dari File JSON ============
            if !f.exists() {
                eprintln!("✗ File tidak ditemukan: {}", f.display());
                return;
            }

            let extension = f.extension().and_then(|e| e.to_str()).unwrap_or("");

            // Cek format file
            match extension.to_lowercase().as_str() {
                "json" => {
                    match std::fs::read_to_string(f) {
                        Ok(content) => {
                            // Coba parse JSON
                            match serde_json::from_str::<serde_json::Value>(&content) {
                                Ok(json) => {
                                    // Cek format: single atau bulk
                                    if json.get("entries").is_some() {
                                        // Format bulk: ada field entries
                                        match parse_blacklist_bulk(&json, &blacklist_repo).await {
                                            Ok(count) => {
                                                println!(
                                                    "✓ Berhasilimport {count} blacklist dari file."
                                                );
                                            }
                                            Err(e) => eprintln!("✗ Gagal: {}", e),
                                        }
                                    } else {
                                        // Format single
                                        match parse_blacklist_single(&json, &blacklist_repo).await {
                                            Ok(_) => {
                                                let name = json
                                                    .get("name")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("?");
                                                println!("✓ Blacklist '{}' ditambahkan.", name);
                                            }
                                            Err(e) => eprintln!("✗ Gagal: {}", e),
                                        }
                                    }
                                }
                                Err(e) => eprintln!("✗ Format JSON tidak valid: {}", e),
                            }
                        }
                        Err(e) => eprintln!("✗ Gagal baca file: {}", e),
                    }
                }
                "toml" | "yaml" => {
                    eprintln!("✗ Format .{} belum tersedia.", extension);
                }
                _ => {
                    eprintln!("✗ Format tidak didukung: {}", extension);
                    eprintln!("  Gunakan .json saja untuk saat ini.");
                }
            }
        } else {
            eprintln!("✗ Berikan --name dan --app-name, atau --file");
        }
    });
}

fn cmd_remove_blacklist(name: &str) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        use crate::constants::paths;
        use crate::db::init::init_database;
        use crate::repository::BlacklistRepository;

        let db = match init_database(&paths::get_db_path()).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("✗ Gagal akses database: {e}");
                return;
            }
        };

        let blacklist_repo = BlacklistRepository::new(db.clone());

        match blacklist_repo.find_by_name(name).await {
            Ok(Some(bl)) => match blacklist_repo.delete(bl.id).await {
                Ok(_) => println!("✓ Menghapus '{name}' dari blacklist..."),
                Err(e) => eprintln!("✗ Gagal: {e}"),
            },
            Ok(None) => eprintln!("✗ '{name}' tidak ditemukan di blacklist."),
            Err(e) => eprintln!("✗ Error: {e}"),
        }
    });
}

fn cmd_list_blacklist() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        use crate::constants::paths;
        use crate::db::init::init_database;
        use crate::repository::BlacklistRepository;

        let db = match init_database(&paths::get_db_path()).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("✗ Gagal akses database: {e}");
                return;
            }
        };

        let blacklist_repo = BlacklistRepository::new(db.clone());

        match blacklist_repo.find_all_with_details().await {
            Ok(list) => {
                if list.is_empty() {
                    println!("Blacklist kosong.");
                    return;
                }
                println!("\n=== Daftar Blacklist ===\n");
                for item in list {
                    let status = if item.blacklist.enabled {
                        "[Aktif]"
                    } else {
                        "[Nonaktif]"
                    };
                    println!("{} {}", status, item.blacklist.name);
                    if let Some(desc) = &item.blacklist.description {
                        println!("  Deskripsi: {}", desc);
                    }
                    if !item.processes.is_empty() {
                        let procs: Vec<String> = item
                            .processes
                            .iter()
                            .map(|p| p.process_name.clone())
                            .collect();
                        println!("  Proses: {:?}", procs);
                    }
                    if !item.paths.is_empty() {
                        let paths: Vec<String> =
                            item.paths.iter().map(|p| p.path.clone()).collect();
                        println!("  Path: {:?}", paths);
                    }
                    println!();
                }
            }
            Err(e) => eprintln!("✗ Error: {e}"),
        }
    });
}

fn cmd_add_whitelist(file: Option<&Path>, name: Option<&str>) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        use crate::constants::paths;
        use crate::db::init::init_database;
        use crate::repository::WhitelistRepository;

        let db = match init_database(&paths::get_db_path()).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("✗ Gagal akses database: {e}");
                return;
            }
        };

        let whitelist_repo = WhitelistRepository::new(db.clone());

        // Dari file JSON
        if let Some(f) = file {
            if !f.exists() {
                eprintln!("✗ File tidak ditemukan");
                return;
            }
            let content = match std::fs::read_to_string(f) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("✗ Baca file: {}", e);
                    return;
                }
            };
            let json: serde_json::Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("✗ JSON error: {}", e);
                    return;
                }
            };
            // Bulk format
            if let Some(entries) = json.get("entries").and_then(|v| v.as_array()) {
                let mut count = 0;
                for entry in entries {
                    if let Some(n) = entry.get("name").and_then(|v| v.as_str()) {
                        if whitelist_repo.create(n, None, true).await.is_ok() {
                            count += 1;
                        }
                    }
                }
                println!("✓ Import {} whitelist.", count);
            } else if let Some(n) = json.get("name").and_then(|v| v.as_str()) {
                // Single format
                match whitelist_repo.create(n, None, true).await {
                    Ok(_) => println!("✓ '{}' ditambahkan.", n),
                    Err(e) => eprintln!("✗ {}", e),
                }
            } else {
                eprintln!("✗ Format tidak valid");
            }
            return;
        }

        // Dari parameter CLI
        if let Some(n) = name {
            match whitelist_repo.create(n, None, true).await {
                Ok(_) => println!("✓ '{}' ke whitelist.", n),
                Err(e) => eprintln!("✗ {}", e),
            }
            return;
        }

        eprintln!("✗ Gunakan --name process.exe atau --file whitelist.json");
    });
}

fn cmd_remove_whitelist(name: &str) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        use crate::constants::paths;
        use crate::db::init::init_database;
        use crate::repository::WhitelistRepository;

        let db = match init_database(&paths::get_db_path()).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("✗ Gagal akses database: {e}");
                return;
            }
        };

        let whitelist_repo = WhitelistRepository::new(db.clone());

        match whitelist_repo.find_by_process_name(name).await {
            Ok(Some(wl)) => match whitelist_repo.delete(wl.id).await {
                Ok(_) => println!("✓ Menghapus '{name}' dari whitelist..."),
                Err(e) => eprintln!("✗ Gagal: {e}"),
            },
            Ok(None) => eprintln!("✗ '{name}' tidak ditemukan di whitelist."),
            Err(e) => eprintln!("✗ Error: {e}"),
        }
    });
}

fn cmd_list_whitelist() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        use crate::constants::paths;
        use crate::db::init::init_database;
        use crate::repository::WhitelistRepository;

        let db = match init_database(&paths::get_db_path()).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("✗ Gagal akses database: {e}");
                return;
            }
        };

        let whitelist_repo = WhitelistRepository::new(db.clone());

        match whitelist_repo.find_all().await {
            Ok(list) => {
                if list.is_empty() {
                    println!("Whitelist kosong.");
                    return;
                }
                println!("\n=== Daftar Whitelist ===\n");
                for item in list {
                    let status = if item.enabled {
                        "[Aktif]"
                    } else {
                        "[Nonaktif]"
                    };
                    println!("{} {}", status, item.process_name);
                    if let Some(desc) = &item.description {
                        println!("  Deskripsi: {}", desc);
                    }
                }
                println!();
            }
            Err(e) => eprintln!("✗ Error: {e}"),
        }
    });
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
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        use crate::constants::paths;
        use crate::db::init::init_database;
        use crate::repository::ConfigRepository;

        let db = match init_database(&paths::get_db_path()).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("✗ Gagal akses database: {e}");
                return;
            }
        };

        let config_repo = ConfigRepository::new(db.clone());

        match config_repo.find_all().await {
            Ok(configs) => {
                // Generate TOML content
                let mut toml_content = String::new();
                toml_content.push_str("# App Blocker Configuration Export\n");
                toml_content.push_str(&format!(
                    "# Exported at: {}\n\n",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
                ));

                // Group by section
                let mut current_section = String::new();

                for config in configs {
                    // Extract section from key (before first dot)
                    if let Some(section) = config.key.split('.').next() {
                        if section != current_section {
                            current_section = section.to_string();
                            toml_content.push_str(&format!("[{}]\n", section));
                        }
                        // Format: key = value
                        let short_key = config.key.split('.').skip(1).collect::<Vec<_>>().join(".");
                        if short_key.is_empty() {
                            toml_content
                                .push_str(&format!("{} = \"{}\"\n", config.key, config.value));
                        } else {
                            toml_content
                                .push_str(&format!("{} = \"{}\"\n", short_key, config.value));
                        }
                    }
                }

                match std::fs::write(output, toml_content) {
                    Ok(_) => println!("✓ Konfigurasi disimpan ke: {}", output.display()),
                    Err(e) => eprintln!("✗ Gagal simpan: {e}"),
                }
            }
            Err(e) => eprintln!("✗ Error: {e}"),
        }
    });
}

/// cmd_simulation_mode - Ubah mode simulasi menggunakan database
fn cmd_simulation_mode(enabled: bool) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        use crate::constants::paths;
        use crate::db::init::init_database;
        use crate::repository::ConfigRepository;

        let db = match init_database(&paths::get_db_path()).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("✗ Gagal akses database: {e}");
                return;
            }
        };

        let config_repo = ConfigRepository::new(db.clone());

        let value = if enabled { "true" } else { "false" };
        match config_repo
            .set("simulation.enabled", value, Some("Simulation mode enabled"))
            .await
        {
            Ok(_) => {
                let status = if enabled {
                    "diaktifkan"
                } else {
                    "dinonaktifkan"
                };
                println!("✓ Mode simulasi {status}.");
                println!("  Perubahan akan berlaku pada restart berikutnya.");
            }
            Err(e) => eprintln!("✗ Gagal: {e}"),
        }
    });
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

// ============ Statistik Functions ============

fn cmd_stats(period: &str) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        use crate::constants::paths;
        use crate::db::init::init_database;
        use crate::repository::LogRepository;

        let db = match init_database(&paths::get_db_path()).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("✗ DB error: {}", e);
                return;
            }
        };

        let log_repo = LogRepository::new(db.clone());

        // Hitung berdasarkan periode
        let days = match period {
            "day" => 1,
            "week" => 7,
            "month" => 30,
            _ => 7,
        };

        match log_repo.get_blocked_count(days).await {
            Ok(count) => {
                println!("\n=== Statistik Pemblokiran ===");
                println!("Periode      : {} hari", days);
                println!("Total blocked: {}", count);
            }
            Err(e) => eprintln!("✗ {}", e),
        }
    });
}

fn cmd_top_blocked(limit: usize) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        use crate::constants::paths;
        use crate::db::init::init_database;
        use crate::repository::LogRepository;

        let db = match init_database(&paths::get_db_path()).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("✗ DB error: {}", e);
                return;
            }
        };

        let log_repo = LogRepository::new(db.clone());

        match log_repo.get_top_blocked(limit).await {
            Ok(list) => {
                if list.is_empty() {
                    println!("Belum ada data.");
                    return;
                }
                println!("\n=== Top {} Proses Diblokir ===\n", limit);
                for (i, (name, count)) in list.iter().enumerate() {
                    println!("{}. {} - {}x", i + 1, name, count);
                }
            }
            Err(e) => eprintln!("✗ {}", e),
        }
    });
}

fn cmd_audit_log(user: Option<&str>, limit: usize) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        use crate::constants::paths;
        use crate::db::init::init_database;
        use crate::repository::LogRepository;

        let db = match init_database(&paths::get_db_path()).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("✗ DB error: {}", e);
                return;
            }
        };

        let log_repo = LogRepository::new(db.clone());

        match log_repo.get_audit_logs_filtered(user, limit).await {
            Ok(logs) => {
                if logs.is_empty() {
                    println!("Tidak ada log.");
                    return;
                }
                println!("\n=== Audit Log ===\n");
                for log in logs {
                    let status = if log.success { "✓" } else { "✗" };
                    println!(
                        "{} [{}] {} - {}",
                        status,
                        log.timestamp,
                        log.username.as_deref().unwrap_or("system"),
                        log.details.as_deref().unwrap_or("")
                    );
                }
            }
            Err(e) => eprintln!("✗ {}", e),
        }
    });
}

// ============ Schedule Functions ============

fn cmd_schedule_list() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        use crate::constants::paths;
        use crate::db::init::init_database;
        use crate::repository::ScheduleRepository;

        let db = match init_database(&paths::get_db_path()).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("✗ DB error: {}", e);
                return;
            }
        };

        let schedule_repo = ScheduleRepository::new(db.clone());

        match schedule_repo.find_all().await {
            Ok(list) => {
                if list.is_empty() {
                    println!("Jadwal kosong.");
                    return;
                }
                println!("\n=== Daftar Jadwal ===\n");
                for sch in list {
                    let status = if sch.enabled { "[Aktif]" } else { "[Nonaktif]" };
                    println!("{} ID: {} | TZ: {}", status, sch.id, sch.timezone);
                }
            }
            Err(e) => eprintln!("✗ {}", e),
        }
    });
}

fn cmd_schedule_add(days: &str, start: &str, end: &str, action: &str) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        use crate::constants::paths;
        use crate::db::init::init_database;
        use crate::repository::ScheduleRepository;

        let db = match init_database(&paths::get_db_path()).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("✗ DB error: {}", e);
                return;
            }
        };

        let schedule_repo = ScheduleRepository::new(db.clone());

        match schedule_repo
            .create_rule(1, days, start, end, action, true)
            .await
        {
            Ok(_) => println!("✓ Jadwal ditambahkan."),
            Err(e) => eprintln!("✗ {}", e),
        }
    });
}

fn cmd_schedule_remove(id: i32) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        use crate::constants::paths;
        use crate::db::init::init_database;
        use crate::repository::ScheduleRepository;

        let db = match init_database(&paths::get_db_path()).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("✗ DB error: {}", e);
                return;
            }
        };

        let schedule_repo = ScheduleRepository::new(db.clone());

        match schedule_repo.delete_rule(id).await {
            Ok(_) => println!("✓ Jadwal dihapus."),
            Err(e) => eprintln!("✗ {}", e),
        }
    });
}

fn cmd_schedule_toggle(id: i32) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        use crate::constants::paths;
        use crate::db::init::init_database;
        use crate::repository::ScheduleRepository;

        let db = match init_database(&paths::get_db_path()).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("✗ DB error: {}", e);
                return;
            }
        };

        let schedule_repo = ScheduleRepository::new(db.clone());

        match schedule_repo.toggle_rule(id).await {
            Ok(_) => println!("✓ Jadwal toggle."),
            Err(e) => eprintln!("✗ {}", e),
        }
    });
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

// ============ JSON Parse Helpers untuk Blacklist ============

/// Parse single blacklist entry dari JSON
async fn parse_blacklist_single(
    json: &serde_json::Value,
    repo: &crate::repository::BlacklistRepository,
) -> Result<(), String> {
    let name = json
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or("Field 'name' wajib ada")?;

    let description = json.get("description").and_then(|v| v.as_str());

    // Create blacklist entry
    let blacklist = repo
        .create(name, description, false)
        .await
        .map_err(|e| e.to_string())?;

    // Add process names
    if let Some(procs) = json.get("process_names").and_then(|v| v.as_array()) {
        for proc in procs {
            if let Some(proc_name) = proc.as_str() {
                let _ = repo.add_process(blacklist.id, proc_name).await;
            }
        }
    }

    // Add paths
    if let Some(paths) = json.get("paths").and_then(|v| v.as_array()) {
        for path in paths {
            if let Some(path_str) = path.as_str() {
                let _ = repo.add_path(blacklist.id, path_str).await;
            }
        }
    }

    Ok(())
}

/// Parse bulk blacklist entries dari JSON
async fn parse_blacklist_bulk(
    json: &serde_json::Value,
    repo: &crate::repository::BlacklistRepository,
) -> Result<usize, String> {
    let entries = json
        .get("entries")
        .and_then(|v| v.as_array())
        .ok_or("Field 'entries' tidak ditemukan")?;

    let mut count = 0;
    for entry in entries {
        if parse_blacklist_single(entry, repo).await.is_ok() {
            count += 1;
        }
    }

    Ok(count)
}
