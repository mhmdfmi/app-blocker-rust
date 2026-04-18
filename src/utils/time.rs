/// Utilitas waktu dengan dukungan timezone Asia/Jakarta (WIB, UTC+7).
use chrono::{DateTime, Datelike, Local, NaiveTime, Utc, Weekday};
// use chrono::Timelike;  // Untuk format durasi jika ingin lebih detail (jam, menit, detik)

/// Format timestamp standar untuk logging
pub const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.3f";

/// Offset WIB dari UTC dalam jam
pub const WIB_OFFSET_HOURS: i32 = 7;

/// Mendapatkan timestamp UTC saat ini
pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

/// Mendapatkan timestamp lokal saat ini
pub fn now_local() -> DateTime<Local> {
    Local::now()
}

/// Format datetime ke string standar
pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format(TIMESTAMP_FORMAT).to_string()
}

/// Format datetime lokal ke string standar
pub fn format_local(dt: &DateTime<Local>) -> String {
    dt.format(TIMESTAMP_FORMAT).to_string()
}

pub fn chrono_to_local(utc_dt: &DateTime<Utc>) -> DateTime<Local> {
    utc_dt.with_timezone(&Local)
}

/// Informasi jadwal waktu blokir
#[derive(Debug, Clone)]
pub struct ScheduleWindow {
    /// Hari-hari aktif (0=Senin, 6=Minggu)
    pub days: Vec<u8>,
    /// Waktu mulai
    pub start: NaiveTime,
    /// Waktu selesai
    pub end: NaiveTime,
}

impl ScheduleWindow {
    /// Buat jadwal baru
    pub fn new(days: Vec<u8>, start_hhmm: (u32, u32), end_hhmm: (u32, u32)) -> Option<Self> {
        let start = NaiveTime::from_hms_opt(start_hhmm.0, start_hhmm.1, 0)?;
        let end = NaiveTime::from_hms_opt(end_hhmm.0, end_hhmm.1, 0)?;
        Some(Self { days, start, end })
    }

    /// Periksa apakah waktu saat ini masuk jadwal blokir
    pub fn is_active_now(&self) -> bool {
        let now = now_local();
        let weekday_num = weekday_to_num(now.weekday());
        if !self.days.contains(&weekday_num) {
            return false;
        }
        let current_time = now.time();
        current_time >= self.start && current_time < self.end
    }
}

/// Konversi Weekday chrono ke angka (0=Senin, 6=Minggu)
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

/// Parse string hari ke angka
pub fn parse_day_name(name: &str) -> Option<u8> {
    match name.to_lowercase().as_str() {
        "monday" | "senin" => Some(0),
        "tuesday" | "selasa" => Some(1),
        "wednesday" | "rabu" => Some(2),
        "thursday" | "kamis" => Some(3),
        "friday" | "jumat" => Some(4),
        "saturday" | "sabtu" => Some(5),
        "sunday" | "minggu" => Some(6),
        _ => None,
    }
}

/// Mendapatkan durasi sejak timestamp dalam detik
pub fn elapsed_seconds(since: &DateTime<Utc>) -> f64 {
    let duration = Utc::now() - *since;
    duration.num_milliseconds() as f64 / 1000.0
}

/// Format durasi menjadi string yang mudah dibaca
pub fn format_duration_seconds(seconds: u64) -> String {
    if seconds < 60 {
        format!("{seconds}d")
    } else if seconds < 3600 {
        format!("{}m {}d", seconds / 60, seconds % 60)
    } else {
        format!("{}j {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}
