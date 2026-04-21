/// Definisi struct konfigurasi utama aplikasi.
/// Semua field memiliki default value yang aman.
use serde::{Deserialize, Serialize};
// use std::collections::HashMap; // Untuk konfigurasi yang lebih kompleks di masa depan

/// Konfigurasi utama aplikasi
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    /// Metadata aplikasi
    pub app: AppMeta,
    /// Konfigurasi monitoring proses
    pub monitoring: MonitoringConfig,
    /// Konfigurasi blokir proses
    pub blocking: BlockingConfig,
    /// Konfigurasi jadwal
    pub schedule: ScheduleConfig,
    /// Konfigurasi overlay UI
    pub overlay: OverlayConfig,
    /// Konfigurasi logging
    pub logging: LoggingConfig,
    /// Konfigurasi keamanan
    pub security: SecurityConfig,
    /// Konfigurasi watchdog
    pub watchdog: WatchdogConfig,
    /// Mode simulasi
    pub simulation: SimulationConfig,
}

/// Metadata aplikasi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMeta {
    pub mode: AppMode,
    pub startup_delay_seconds: u64,
    pub max_cpu_usage_percent: u32,
    pub max_memory_mb: u64,
}

/// Mode operasi aplikasi
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    Production,
    Development,
    Simulation,
}

impl std::fmt::Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppMode::Production => write!(f, "production"),
            AppMode::Development => write!(f, "development"),
            AppMode::Simulation => write!(f, "simulation"),
        }
    }
}

/// Konfigurasi monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Interval scan dalam milliseconds
    pub scan_interval_ms: u64,
    /// Delay validasi sebelum kill (min, max) ms
    pub validation_delay_ms: (u64, u64),
    /// Aktifkan adaptive scan interval
    pub adaptive_interval: bool,
}

/// Konfigurasi blokir
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockingConfig {
    /// Daftar proses yang diblokir
    pub blacklist: Vec<BlockedApp>,
    /// Daftar proses yang diizinkan (prioritas lebih tinggi)
    pub whitelist: Vec<String>,
    /// Rate limit kill per menit
    pub kill_rate_limit_per_minute: u32,
    /// Grace period sebelum kill (detik)
    pub grace_period_seconds: (u64, u64),
    /// Aktifkan behavior scoring
    pub behavior_scoring_enabled: bool,
    /// Threshold skor untuk blokir
    pub score_threshold: u32,
}

/// Definisi aplikasi yang diblokir
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedApp {
    pub name: String,
    pub process_names: Vec<String>,
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub description: String,
}

/// Konfigurasi jadwal blokir
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub enabled: bool,
    pub timezone: String,
    pub rules: Vec<ScheduleRule>,
}

/// Satu aturan jadwal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleRule {
    pub days: Vec<String>,
    pub start: String,
    pub end: String,
    pub action: String,
}

/// Konfigurasi overlay UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub focus_interval_ms: u64,
    pub failsafe_timeout_minutes: u64,
    pub max_unlock_attempts: u32,
    pub lockout_duration_seconds: u64,
    pub show_process_info: bool,
    pub show_timestamp: bool,
    pub show_pc_name: bool,
    pub show_attempt_counter: bool,
}

/// Konfigurasi logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub path: String,
    pub level: String,
    pub rotation_days: u32,
    pub structured: bool,
}

/// Konfigurasi keamanan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub max_auth_attempts: u32,
    pub backoff_base_seconds: u64,
    pub lockout_duration_seconds: u64,
    pub memory_zero_on_drop: bool,
    pub anti_debugging: bool,
    pub check_disable_flag_interval_ms: u64,
}

/// Konfigurasi watchdog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchdogConfig {
    pub heartbeat_interval_ms: u64,
    pub max_missed_heartbeats: u32,
    pub max_restart_attempts: u32,
    pub deadlock_timeout_ms: u64,
}

/// Konfigurasi mode simulasi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub enabled: bool,
    pub simulate_process_kill: bool,
    pub simulate_overlay: bool,
    pub log_only: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppMeta {
                mode: AppMode::Production,
                startup_delay_seconds: 15,
                max_cpu_usage_percent: 20,
                max_memory_mb: 200,
            },
            monitoring: MonitoringConfig {
                scan_interval_ms: 2000,
                validation_delay_ms: (300, 1000),
                adaptive_interval: true,
            },
            blocking: BlockingConfig {
                blacklist: default_blacklist(),
                whitelist: Vec::new(),
                kill_rate_limit_per_minute: 3,
                grace_period_seconds: (1, 2),
                behavior_scoring_enabled: true,
                score_threshold: 70,
            },
            schedule: ScheduleConfig {
                enabled: true,
                timezone: "Asia/Jakarta".to_string(),
                rules: default_schedule_rules(),
            },
            overlay: OverlayConfig {
                focus_interval_ms: 500,
                failsafe_timeout_minutes: 30,
                max_unlock_attempts: 5,
                lockout_duration_seconds: 60,
                show_process_info: true,
                show_timestamp: true,
                show_pc_name: true,
                show_attempt_counter: true,
            },
            logging: LoggingConfig {
                path: r"C:\AppBlocker\logs".to_string(),
                level: "info".to_string(),
                rotation_days: 7,
                structured: true,
            },
            security: SecurityConfig {
                max_auth_attempts: 5,
                backoff_base_seconds: 5,
                lockout_duration_seconds: 300,
                memory_zero_on_drop: true,
                anti_debugging: true,
                check_disable_flag_interval_ms: 2000,
            },
            watchdog: WatchdogConfig {
                heartbeat_interval_ms: 1000,
                max_missed_heartbeats: 5,
                max_restart_attempts: 3,
                deadlock_timeout_ms: 5000,
            },
            simulation: SimulationConfig {
                enabled: false,
                simulate_process_kill: false,
                simulate_overlay: false,
                log_only: false,
            },
        }
    }
}

/// Daftar default game yang diblokir
fn default_blacklist() -> Vec<BlockedApp> {
    vec![
        BlockedApp {
            name: "Roblox".to_string(),
            process_names: vec!["RobloxPlayerBeta.exe".to_string()],
            paths: vec![
                r"C:\Program Files (x86)\Roblox\".to_string(),
                r"C:\Users\*\AppData\Local\Roblox\".to_string(),
            ],
            description: "Platform game Roblox".to_string(),
        },
        BlockedApp {
            name: "Valorant".to_string(),
            process_names: vec!["VALORANT-Win64-Shipping.exe".to_string()],
            paths: vec![r"C:\Riot Games\VALORANT\".to_string()],
            description: "Game Valorant".to_string(),
        },
        BlockedApp {
            name: "Steam".to_string(),
            process_names: vec!["steam.exe".to_string()],
            paths: vec![r"C:\Program Files (x86)\Steam\".to_string()],
            description: "Platform Steam".to_string(),
        },
        BlockedApp {
            name: "Epic Games".to_string(),
            process_names: vec!["EpicGamesLauncher.exe".to_string()],
            paths: vec![r"C:\Program Files (x86)\Epic Games\".to_string()],
            description: "Epic Games Launcher".to_string(),
        },
    ]
}

/// Aturan jadwal default (jam sekolah)
fn default_schedule_rules() -> Vec<ScheduleRule> {
    vec![
        ScheduleRule {
            days: vec![
                "Monday".to_string(),
                "Tuesday".to_string(),
                "Wednesday".to_string(),
                "Thursday".to_string(),
                "Friday".to_string(),
            ],
            start: "07:00".to_string(),
            end: "15:00".to_string(),
            action: "block_games".to_string(),
        },
        ScheduleRule {
            days: vec!["Saturday".to_string()],
            start: "07:00".to_string(),
            end: "12:00".to_string(),
            action: "block_games".to_string(),
        },
    ]
}
