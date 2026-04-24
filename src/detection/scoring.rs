/// Sistem scoring perilaku proses untuk mendeteksi aktivitas mencurigakan.
use crate::system::process::ProcessInfo;
use tracing::debug;

/// Bobot skor untuk setiap faktor
pub struct ScoringWeights {
    pub cpu_spike: u32,
    pub rapid_spawn: u32,
    pub suspicious_path: u32,
    pub hidden_process: u32,
    pub renamed_exe: u32,
    pub game_name_match: u32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            cpu_spike: 15,
            rapid_spawn: 20,
            suspicious_path: 25,
            hidden_process: 20,
            renamed_exe: 30,
            game_name_match: 95,
        }
    }
}

/// Hasil scoring
#[derive(Debug, Clone)]
pub struct ScoreResult {
    /// Total skor (0-100+)
    pub total: u32,
    /// Faktor-faktor yang berkontribusi
    pub factors: Vec<String>,
}

impl Default for ScoreResult {
    fn default() -> Self {
        Self::new()
    }
}

impl ScoreResult {
    pub fn new() -> Self {
        Self {
            total: 0,
            factors: Vec::new(),
        }
    }

    pub fn add(&mut self, score: u32, reason: &str) {
        self.total += score;
        self.factors.push(format!("{reason}(+{score})"));
    }
}

/// Scoring engine
pub struct BehaviorScorer {
    weights: ScoringWeights,
    threshold: u32,
    /// Path mencurigakan yang sering digunakan untuk bypass
    suspicious_paths: Vec<String>,
}

impl BehaviorScorer {
    /// Buat scorer baru
    pub fn new(threshold: u32) -> Self {
        Self {
            weights: ScoringWeights::default(),
            threshold,
            suspicious_paths: vec![
                r"C:\Users\Public\".to_lowercase(),
                r"C:\Temp\".to_lowercase(),
                r"C:\Windows\Temp\".to_lowercase(),
                r"\AppData\Local\Temp\".to_lowercase(),
            ],
        }
    }

    /// Hitung skor perilaku proses
    pub fn score(&self, proc: &ProcessInfo) -> ScoreResult {
        let mut result = ScoreResult::new();

        // 1. CPU spike tinggi (>80%)
        if proc.cpu_usage > 80.0 {
            result.add(self.weights.cpu_spike, "cpu_spike");
        }

        // 2. Path mencurigakan
        if let Some(exe) = &proc.exe_path {
            let exe_lower = exe.to_lowercase();
            for sus_path in &self.suspicious_paths {
                if exe_lower.contains(sus_path.as_str()) {
                    result.add(self.weights.suspicious_path, "suspicious_path");
                    break;
                }
            }

            // 3. Proses tanpa UI yang makan CPU tinggi (hidden process indicator)
            if proc.cpu_usage > 50.0 && proc.status == "Run" {
                // Jika nama .exe tidak cocok dengan folder (kemungkinan renamed)
                if let Some(filename) = std::path::Path::new(exe).file_name() {
                    let fname = filename.to_string_lossy().to_lowercase();
                    let pname = proc.name.to_lowercase();
                    if !fname.contains(&pname) && !pname.contains(&fname.replace(".exe", "")) {
                        result.add(self.weights.renamed_exe, "renamed_exe");
                    }
                }
            }
        }

        debug!(
            pid = proc.pid,
            name = %proc.name,
            score = result.total,
            factors = ?result.factors,
            "Skor perilaku dihitung"
        );

        result
    }

    /// Apakah skor melebihi threshold untuk diblokir
    pub fn should_block(&self, score: u32) -> bool {
        score >= self.threshold
    }
}
