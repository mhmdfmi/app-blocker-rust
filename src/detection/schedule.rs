/// Sistem jadwal blokir berbasis waktu - timezone-aware (Asia/Jakarta WIB).
use crate::config::settings::ScheduleConfig;
use crate::utils::error::{AppError, AppResult};
use crate::utils::time::parse_day_name;
use chrono::{Local, NaiveTime, Timelike, Datelike, Weekday};
use tracing::{debug, warn};

/// Satu aturan jadwal yang sudah diparsing
#[derive(Debug, Clone)]
struct ParsedRule {
    /// Hari-hari aktif (0=Senin..6=Minggu)
    days: Vec<u8>,
    start: NaiveTime,
    end: NaiveTime,
    action: String,
}

/// Layanan jadwal blokir
pub struct ScheduleService {
    enabled: bool,
    rules: Vec<ParsedRule>,
}

impl ScheduleService {
    /// Buat service dari konfigurasi
    pub fn new(config: &ScheduleConfig) -> AppResult<Self> {
        if !config.enabled {
            return Ok(Self { enabled: false, rules: Vec::new() });
        }

        let mut rules = Vec::new();
        for rule in &config.rules {
            let days: Vec<u8> = rule.days.iter()
                .filter_map(|d| parse_day_name(d))
                .collect();

            if days.is_empty() {
                warn!(days = ?rule.days, "Aturan jadwal tidak memiliki hari valid");
                continue;
            }

            let start = parse_time(&rule.start)
                .map_err(|e| AppError::Config(format!("Waktu mulai tidak valid '{}': {e}", rule.start)))?;
            let end = parse_time(&rule.end)
                .map_err(|e| AppError::Config(format!("Waktu selesai tidak valid '{}': {e}", rule.end)))?;

            if start >= end {
                return Err(AppError::Config(format!(
                    "Waktu mulai '{}' harus sebelum waktu selesai '{}'",
                    rule.start, rule.end
                )));
            }

            rules.push(ParsedRule { days, start, end, action: rule.action.clone() });
        }

        debug!(rule_count = rules.len(), "Jadwal berhasil dimuat");
        Ok(Self { enabled: true, rules })
    }

    /// Apakah pemblokiran aktif pada waktu saat ini?
    pub fn is_blocking_active(&self) -> bool {
        if !self.enabled {
            return true; // Jika jadwal disabled, selalu aktif
        }

        let now = Local::now();
        let weekday_num = weekday_to_num(now.weekday());
        let current_time = now.time();

        for rule in &self.rules {
            if rule.days.contains(&weekday_num)
                && current_time >= rule.start
                && current_time < rule.end
                && rule.action.contains("block")
            {
                debug!(
                    hari = weekday_num,
                    waktu = %current_time,
                    aturan = %rule.action,
                    "Jadwal blokir aktif"
                );
                return true;
            }
        }

        false
    }

    /// Berapa menit tersisa hingga jadwal selesai?
    pub fn minutes_until_end(&self) -> Option<i64> {
        if !self.enabled {
            return None;
        }
        let now = Local::now();
        let weekday_num = weekday_to_num(now.weekday());
        let current_time = now.time();

        for rule in &self.rules {
            if rule.days.contains(&weekday_num)
                && current_time >= rule.start
                && current_time < rule.end
            {
                let end_secs = rule.end.num_seconds_from_midnight() as i64;
                let now_secs = current_time.num_seconds_from_midnight() as i64;
                return Some((end_secs - now_secs) / 60);
            }
        }
        None
    }
}

/// Parse string waktu "HH:MM" ke NaiveTime
fn parse_time(s: &str) -> Result<NaiveTime, String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("Format waktu harus HH:MM, dapat: '{s}'"));
    }
    let h: u32 = parts[0].parse().map_err(|_| format!("Jam tidak valid: '{}'", parts[0]))?;
    let m: u32 = parts[1].parse().map_err(|_| format!("Menit tidak valid: '{}'", parts[1]))?;
    NaiveTime::from_hms_opt(h, m, 0)
        .ok_or_else(|| format!("Waktu tidak valid: {h}:{m}"))
}

/// Konversi chrono Weekday ke angka (0=Senin)
fn weekday_to_num(day: Weekday) -> u8 {
    match day {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    }
}
