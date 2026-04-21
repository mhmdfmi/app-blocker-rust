/// Deteksi game berdasarkan nama proses, path executable, dan ekstensi.
use crate::config::settings::BlockedApp;
use crate::system::process::ProcessInfo;
use crate::utils::error::AppResult;
use regex::Regex;
use tracing::debug;

/// Hasil deteksi game
#[derive(Debug, Clone)]
pub struct GameDetectionResult {
    pub matched_app: String,
    pub match_reason: MatchReason,
    pub confidence: u32, // 0-100
}

/// Alasan pencocokan
#[derive(Debug, Clone)]
pub enum MatchReason {
    ProcessName,
    ExecutablePath,
    Extension,
    Combination,
}

/// Layanan deteksi game
pub struct GameDetector {
    blacklist: Vec<BlacklistEntry>,
    watched_extensions: Vec<String>,
}

struct BlacklistEntry {
    app: BlockedApp,
    /// Regex precompiled untuk setiap nama proses
    process_regexes: Vec<Regex>,
    /// Regex precompiled untuk path
    path_regexes: Vec<Regex>,
}

impl GameDetector {
    /// Buat detector dari daftar blacklist
    pub fn new(
        blacklist: Vec<BlockedApp>,
        watched_extensions: Vec<String>,
    ) -> AppResult<Self> {
        let mut entries = Vec::new();

        for app in blacklist {
            let mut process_regexes = Vec::new();
            for pname in &app.process_names {
                // Escape regex, case-insensitive matching
                let pattern = format!("(?i)^{}$", regex::escape(pname));
                match Regex::new(&pattern) {
                    Ok(r) => process_regexes.push(r),
                    Err(e) => {
                        tracing::warn!(
                            name = pname,
                            error = %e,
                            "Gagal compile regex nama proses"
                        );
                    }
                }
            }

            let mut path_regexes = Vec::new();
            for path in &app.paths {
                // Konversi glob-like path ke regex
                let pattern = format!(
                    "(?i){}",
                    regex::escape(path).replace(r"\*", ".*")
                );
                match Regex::new(&pattern) {
                    Ok(r) => path_regexes.push(r),
                    Err(e) => {
                        tracing::warn!(
                            path,
                            error = %e,
                            "Gagal compile regex path"
                        );
                    }
                }
            }

            entries.push(BlacklistEntry {
                app,
                process_regexes,
                path_regexes,
            });
        }

        Ok(Self {
            blacklist: entries,
            watched_extensions,
        })
    }

    /// Deteksi apakah proses adalah game terlarang
    pub fn detect(&self, proc: &ProcessInfo) -> Option<GameDetectionResult> {
        for entry in &self.blacklist {
            // 1. Cek nama proses (prioritas tertinggi)
            for regex in &entry.process_regexes {
                if regex.is_match(&proc.name) {
                    debug!(
                        name = %proc.name,
                        app = %entry.app.name,
                        "Cocok berdasarkan nama proses"
                    );
                    return Some(GameDetectionResult {
                        matched_app: entry.app.name.clone(),
                        match_reason: MatchReason::ProcessName,
                        confidence: 95,
                    });
                }
            }

            // 2. Cek path executable
            if let Some(exe_path) = &proc.exe_path {
                for regex in &entry.path_regexes {
                    if regex.is_match(exe_path) {
                        debug!(
                            path = exe_path,
                            app = %entry.app.name,
                            "Cocok berdasarkan path"
                        );
                        return Some(GameDetectionResult {
                            matched_app: entry.app.name.clone(),
                            match_reason: MatchReason::ExecutablePath,
                            confidence: 80,
                        });
                    }
                }
            }
        }

        // 3. Cek ekstensi (confidence lebih rendah)
        if let Some(exe_path) = &proc.exe_path {
            for ext in &self.watched_extensions {
                if exe_path.to_lowercase().ends_with(ext.to_lowercase().as_str()) {
                    // Ekstensi cocok tapi perlu skor lebih untuk blokir
                    debug!(
                        path = exe_path,
                        ext,
                        "Ekstensi cocok (perlu skor tambahan)"
                    );
                }
            }
        }

        None
    }
}
