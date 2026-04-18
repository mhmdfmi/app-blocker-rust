/// Mesin deteksi terpadu - menggabungkan semua detektor.
pub mod behavior;
pub mod bypass;
pub mod game;
pub mod schedule;
pub mod scoring;

use crate::config::settings::AppConfig;
use crate::system::process::ProcessInfo;
use crate::utils::error::AppResult;
use behavior::BehaviorAnalyzer;
use bypass::BypassDetector;
use game::GameDetector;
use schedule::ScheduleService;
use scoring::BehaviorScorer;
use tracing::{debug, info};

/// Hasil deteksi terpadu
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub score: u32,
    pub matched_game: Option<String>,
    pub reasons: Vec<String>,
}

/// Engine deteksi yang menggabungkan semua detektor
pub struct DetectionEngine {
    game_detector: GameDetector,
    behavior_analyzer: BehaviorAnalyzer,
    behavior_scorer: BehaviorScorer,
    bypass_detector: BypassDetector,
    schedule_service: ScheduleService,
    whitelist: Vec<String>,
    score_threshold: u32,
}

impl DetectionEngine {
    /// Buat engine dari konfigurasi
    pub fn new(config: &AppConfig) -> AppResult<Self> {
        let game_detector = GameDetector::new(
            config.blocking.blacklist.clone(),
            vec![".exe".to_string(), ".lnk".to_string()],
        )?;

        let bypass_detector = BypassDetector::new(&config.blocking.blacklist);
        let schedule_service = ScheduleService::new(&config.schedule)?;
        let behavior_scorer = BehaviorScorer::new(config.blocking.score_threshold);

        let whitelist = config
            .blocking
            .whitelist
            .iter()
            .map(|s| s.to_lowercase())
            .collect();

        Ok(Self {
            game_detector,
            behavior_analyzer: BehaviorAnalyzer::new(),
            behavior_scorer,
            bypass_detector,
            schedule_service,
            whitelist,
            score_threshold: config.blocking.score_threshold,
        })
    }

    /// Deteksi apakah proses perlu diblokir.
    pub fn detect(&mut self, proc: &ProcessInfo) -> AppResult<Option<DetectionResult>> {
        // 0. Whitelist - prioritas tertinggi
        let name_lower = proc.name.to_lowercase();
        if self
            .whitelist
            .iter()
            .any(|w| name_lower.contains(w.as_str()))
        {
            debug!(name = %proc.name, "Proses di whitelist, dilewati");
            return Ok(None);
        }

        // 1. Jadwal - di luar jam sekolah, skip
        if !self.schedule_service.is_blocking_active() {
            debug!("Di luar jadwal blokir, monitoring saja");
            return Ok(None);
        }

        let mut total_score = 0u32;
        let mut reasons: Vec<String> = Vec::new();
        let mut matched_game: Option<String> = None;

        // 2. Deteksi berdasarkan nama/path game
        if let Some(game_result) = self.game_detector.detect(proc) {
            total_score += game_result.confidence;
            reasons.push(format!("game_match:{}", game_result.matched_app));
            matched_game = Some(game_result.matched_app);
        }

        // 3. Deteksi bypass
        if let Some(bypass_result) = self.bypass_detector.detect_bypass(proc) {
            total_score += bypass_result.confidence;
            reasons.push(format!("bypass:{:?}", bypass_result.method));
            if matched_game.is_none() {
                matched_game = Some(format!("{:?}", bypass_result.method));
            }
        }

        // 4. Analisis perilaku (hanya jika sudah ada sinyal awal)
        if total_score > 0 || proc.cpu_usage > 50.0 {
            let behavior = self.behavior_analyzer.analyze(proc);
            if behavior.has_any() {
                let bs = behavior.score();
                total_score += bs;
                if behavior.high_cpu {
                    reasons.push("high_cpu".to_string());
                }
                if behavior.rapid_spawn {
                    reasons.push("rapid_spawn".to_string());
                }
                if behavior.hidden_process {
                    reasons.push("hidden_process".to_string());
                }
                if behavior.suspicious_path {
                    reasons.push("suspicious_path".to_string());
                }
            }
        }

        // 5. Keputusan
        if total_score >= self.score_threshold && matched_game.is_some() {
            info!(
                pid  = proc.pid,
                name = %proc.name,
                score= total_score,
                threshold = self.score_threshold,
                game = ?matched_game,
                "BLOKIR"
            );
            return Ok(Some(DetectionResult {
                score: total_score,
                matched_game,
                reasons,
            }));
        }

        debug!(pid = proc.pid, name = %proc.name, score = total_score, "AMAN");
        Ok(None)
    }
}
