/// Analisis perilaku proses untuk mendeteksi aktivitas mencurigakan.
/// Faktor: CPU spike, spawn rate tinggi, proses tersembunyi.
use crate::system::process::ProcessInfo;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::debug;

/// Data historis satu proses untuk analisis perilaku
#[derive(Debug)]
struct ProcessHistory {
    first_seen: Instant,
    cpu_samples: Vec<f32>,
    spawn_count: u32,
}

impl ProcessHistory {
    fn new(cpu: f32) -> Self {
        Self {
            first_seen: Instant::now(),
            cpu_samples: vec![cpu],
            spawn_count: 1,
        }
    }

    fn add_sample(&mut self, cpu: f32) {
        self.cpu_samples.push(cpu);
        // Pertahankan maksimal 10 sampel terakhir
        if self.cpu_samples.len() > 10 {
            self.cpu_samples.remove(0);
        }
    }

    fn avg_cpu(&self) -> f32 {
        if self.cpu_samples.is_empty() {
            return 0.0;
        }
        self.cpu_samples.iter().sum::<f32>() / self.cpu_samples.len() as f32
    }

    fn age_seconds(&self) -> f32 {
        self.first_seen.elapsed().as_secs_f32()
    }
}

/// Flag perilaku mencurigakan
#[derive(Debug, Clone, Default)]
pub struct BehaviorFlags {
    pub high_cpu: bool,
    pub rapid_spawn: bool,
    pub hidden_process: bool,
    pub suspicious_path: bool,
}

impl BehaviorFlags {
    /// Hitung skor tambahan dari behavior flags
    pub fn score(&self) -> u32 {
        let mut s = 0u32;
        if self.high_cpu        { s += 15; }
        if self.rapid_spawn     { s += 20; }
        if self.hidden_process  { s += 20; }
        if self.suspicious_path { s += 25; }
        s
    }

    pub fn has_any(&self) -> bool {
        self.high_cpu || self.rapid_spawn || self.hidden_process || self.suspicious_path
    }
}

/// Analyzer perilaku berbasis histori
pub struct BehaviorAnalyzer {
    /// Histori per nama proses
    history: HashMap<String, ProcessHistory>,
    /// Interval pembersihan histori proses yang sudah tidak ada
    cleanup_interval: Duration,
    last_cleanup: Instant,
    /// Path mencurigakan
    suspicious_path_patterns: Vec<String>,
}

impl BehaviorAnalyzer {
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
            cleanup_interval: Duration::from_secs(60),
            last_cleanup: Instant::now(),
            suspicious_path_patterns: vec![
                r"c:\users\public\".to_lowercase(),
                r"c:\windows\temp\".to_lowercase(),
                r"\appdata\local\temp\".to_lowercase(),
                r"c:\temp\".to_lowercase(),
            ],
        }
    }

    /// Analisis proses dan kembalikan flags perilaku
    pub fn analyze(&mut self, proc: &ProcessInfo) -> BehaviorFlags {
        let key = proc.name.to_lowercase();

        // Tambahkan/update histori
        self.history
            .entry(key.clone())
            .and_modify(|h| h.add_sample(proc.cpu_usage))
            .or_insert_with(|| ProcessHistory::new(proc.cpu_usage));

        let mut flags = BehaviorFlags::default();

        if let Some(hist) = self.history.get(&key) {
            // CPU tinggi berkelanjutan (>60% rata-rata)
            if hist.avg_cpu() > 60.0 {
                flags.high_cpu = true;
                debug!(name = %proc.name, cpu = hist.avg_cpu(), "High CPU terdeteksi");
            }

            // Spawn cepat: proses baru tapi sudah pakai banyak CPU
            if hist.age_seconds() < 5.0 && proc.cpu_usage > 40.0 {
                flags.rapid_spawn = true;
                debug!(name = %proc.name, "Rapid spawn terdeteksi");
            }
        }

        // Proses tersembunyi: CPU tinggi tanpa UI window (heuristik sederhana)
        if proc.cpu_usage > 50.0 && proc.name.to_lowercase().ends_with(".exe") {
            // Nama generik yang sering dipakai penyamaran
            let generic_names = ["svchost", "conhost", "rundll32", "dllhost"];
            if !generic_names.iter().any(|n| proc.name.to_lowercase().contains(n)) {
                flags.hidden_process = proc.exe_path.is_none();
            }
        }

        // Path mencurigakan
        if let Some(exe) = &proc.exe_path {
            let exe_lower = exe.to_lowercase();
            for pattern in &self.suspicious_path_patterns {
                if exe_lower.contains(pattern.as_str()) {
                    flags.suspicious_path = true;
                    break;
                }
            }
        }

        // Bersihkan histori lama secara berkala
        if self.last_cleanup.elapsed() >= self.cleanup_interval {
            self.cleanup_old_entries();
        }

        flags
    }

    /// Hapus histori proses yang sudah lebih dari 5 menit tidak terlihat
    fn cleanup_old_entries(&mut self) {
        let cutoff = Duration::from_secs(300);
        self.history.retain(|_, h| h.first_seen.elapsed() < cutoff);
        self.last_cleanup = Instant::now();
    }
}

impl Default for BehaviorAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
