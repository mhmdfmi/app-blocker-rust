/// Deteksi upaya bypass pemblokiran: rename executable, eksekusi USB, portable apps.
use crate::config::settings::BlockedApp;
use crate::system::process::ProcessInfo;
use tracing::warn;

/// Hasil deteksi bypass
#[derive(Debug, Clone)]
pub struct BypassDetectionResult {
    pub method: BypassMethod,
    pub confidence: u32,
    pub description: String,
}

/// Metode bypass yang terdeteksi
#[derive(Debug, Clone)]
pub enum BypassMethod {
    /// Executable diganti namanya
    RenamedExecutable,
    /// Dieksekusi dari USB/removable drive
    UsbExecution,
    /// Portable app tanpa instalasi
    PortableApp,
    /// Proses anak dari launcher yang diblokir
    ChildOfBlockedProcess,
}

/// Layanan deteksi bypass
pub struct BypassDetector {
    /// Daftar nama asli game yang diblokir (lowercase)
    known_game_exes: Vec<String>,
    /// Prefiks drive removable (D:, E:, F:, dst.)
    removable_drive_prefixes: Vec<String>,
}

impl BypassDetector {
    pub fn new(blacklist: &[BlockedApp]) -> Self {
        let known_game_exes = blacklist
            .iter()
            .flat_map(|app| app.process_names.iter())
            .map(|n| n.to_lowercase().replace(".exe", ""))
            .collect();

        // Drive D: ke Z: potensial removable/USB (A: B: system, C: biasanya system)
        let removable_drive_prefixes = ('d'..='z')
            .map(|c| format!("{c}:\\"))
            .collect();

        Self {
            known_game_exes,
            removable_drive_prefixes,
        }
    }

    /// Analisis proses untuk upaya bypass
    pub fn detect_bypass(&self, proc: &ProcessInfo) -> Option<BypassDetectionResult> {
        // 1. Deteksi eksekusi dari USB/drive removable
        if let Some(result) = self.check_usb_execution(proc) {
            return Some(result);
        }

        // 2. Deteksi executable yang diganti nama
        if let Some(result) = self.check_renamed_executable(proc) {
            return Some(result);
        }

        // 3. Deteksi portable app (exe di luar folder instalasi standar)
        if let Some(result) = self.check_portable_app(proc) {
            return Some(result);
        }

        None
    }

    /// Cek apakah proses berjalan dari drive USB (D: ke Z:)
    fn check_usb_execution(&self, proc: &ProcessInfo) -> Option<BypassDetectionResult> {
        if let Some(exe) = &proc.exe_path {
            let exe_lower = exe.to_lowercase();
            for prefix in &self.removable_drive_prefixes {
                if exe_lower.starts_with(prefix.as_str()) {
                    // Cek apakah nama file cocok dengan game yang dikenal
                    let filename = std::path::Path::new(exe)
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_lowercase())
                        .unwrap_or_default();

                    if self.known_game_exes.iter().any(|g| filename.contains(g.as_str()) || g.contains(filename.as_str())) {
                        warn!(pid = proc.pid, exe, "Deteksi eksekusi USB game!");
                        return Some(BypassDetectionResult {
                            method: BypassMethod::UsbExecution,
                            confidence: 85,
                            description: format!("Game dijalankan dari drive removable: {exe}"),
                        });
                    }
                }
            }
        }
        None
    }

    /// Cek apakah executable telah diganti nama
    fn check_renamed_executable(&self, proc: &ProcessInfo) -> Option<BypassDetectionResult> {
        if let Some(exe) = &proc.exe_path {
            let filename = std::path::Path::new(exe)
                .file_stem()
                .map(|s| s.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            let proc_name_stem = proc.name.to_lowercase().replace(".exe", "");

            // Nama file berbeda dengan nama proses (indikasi rename)
            if !filename.is_empty()
                && !proc_name_stem.is_empty()
                && filename != proc_name_stem
                && proc.cpu_usage > 30.0
            {
                // Cek apakah salah satunya cocok dengan game yang dikenal
                let file_is_game = self.known_game_exes.iter()
                    .any(|g| filename.contains(g.as_str()));
                let proc_is_game = self.known_game_exes.iter()
                    .any(|g| proc_name_stem.contains(g.as_str()));

                if file_is_game || proc_is_game {
                    warn!(
                        pid = proc.pid,
                        proc_name = %proc.name,
                        file_name = %filename,
                        "Deteksi rename executable game!"
                    );
                    return Some(BypassDetectionResult {
                        method: BypassMethod::RenamedExecutable,
                        confidence: 75,
                        description: format!(
                            "Nama proses '{}' berbeda dengan nama file '{}'",
                            proc.name, filename
                        ),
                    });
                }
            }
        }
        None
    }

    /// Cek apakah ini portable app (tidak di folder instalasi standar)
    fn check_portable_app(&self, proc: &ProcessInfo) -> Option<BypassDetectionResult> {
        if let Some(exe) = &proc.exe_path {
            let exe_lower = exe.to_lowercase();
            let proc_name_lower = proc.name.to_lowercase().replace(".exe", "");

            // Apakah nama proses cocok dengan game yang dikenal?
            let is_known_game = self.known_game_exes.iter()
                .any(|g| proc_name_lower.contains(g.as_str()) || g.contains(proc_name_lower.as_str()));

            if is_known_game {
                // Apakah lokasinya bukan di folder instalasi standar?
                let standard_paths = [
                    r"c:\program files",
                    r"c:\program files (x86)",
                    r"c:\riot games",
                ];
                let in_standard_path = standard_paths.iter()
                    .any(|p| exe_lower.starts_with(p));

                if !in_standard_path {
                    warn!(
                        pid = proc.pid,
                        exe,
                        "Deteksi portable app game di lokasi tidak standar!"
                    );
                    return Some(BypassDetectionResult {
                        method: BypassMethod::PortableApp,
                        confidence: 70,
                        description: format!("Game dijalankan dari lokasi tidak standar: {exe}"),
                    });
                }
            }
        }
        None
    }
}
