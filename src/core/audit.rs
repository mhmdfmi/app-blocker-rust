/// Audit logger - menulis JSON audit report secara atomic per-event.
/// Setiap event kritis (detect, kill, unlock, auth) menghasilkan baris JSON.
use crate::utils::error::{AppError, AppResult};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing::{error, warn};
use uuid::Uuid;

/// Tipe event audit
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventKind {
    ProcessDetected,
    ProcessKilled,
    ProcessKillFailed,
    OverlayShown,
    OverlayDismissed,
    AuthSuccess,
    AuthFailed,
    AuthLockedOut,
    SystemStarted,
    SystemStopped,
    SafeModeEntered,
    EmergencyUnlock,
    ConfigReloaded,
    DisableFlagDetected,
}

/// Satu baris audit
#[derive(Debug, Clone, Serialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub trace_id: String,
    pub event: AuditEventKind,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    pub username: Option<String>,
    pub computer_name: Option<String>,
    pub score: Option<u32>,
    pub detail: Option<String>,
    pub session_duration_seconds: Option<f64>,
    pub detection_method: Option<String>,
    pub schedule_rule_triggered: Option<bool>,
}

impl AuditEntry {
    /// Buat entry minimal dengan trace_id baru
    pub fn new(event: AuditEventKind) -> Self {
        Self {
            timestamp: Utc::now(),
            trace_id: Uuid::new_v4().to_string(),
            event,
            pid: None,
            process_name: None,
            username: None,
            computer_name: None,
            score: None,
            detail: None,
            session_duration_seconds: None,
            detection_method: None,
            schedule_rule_triggered: None,
        }
    }

    pub fn with_trace(mut self, id: &Uuid) -> Self {
        self.trace_id = id.to_string();
        self
    }

    pub fn with_process(mut self, pid: u32, name: &str) -> Self {
        self.pid = Some(pid);
        self.process_name = Some(name.to_string());
        self
    }

    pub fn with_user(mut self, username: &str, computer: &str) -> Self {
        self.username = Some(username.to_string());
        self.computer_name = Some(computer.to_string());
        self
    }

    pub fn with_score(mut self, score: u32) -> Self {
        self.score = Some(score);
        self
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_detection_method(mut self, method: impl Into<String>) -> Self {
        self.detection_method = Some(method.into());
        self
    }

    pub fn with_schedule(mut self, triggered: bool) -> Self {
        self.schedule_rule_triggered = Some(triggered);
        self
    }

    pub fn with_duration(mut self, seconds: f64) -> Self {
        self.session_duration_seconds = Some(seconds);
        self
    }
}

/// Audit writer thread-safe
pub struct AuditWriter {
    report_dir: PathBuf,
    buffer: Mutex<Vec<AuditEntry>>,
    max_buffer: usize,
}

impl AuditWriter {
    /// Buat writer baru dengan direktori output
    pub fn new(report_dir: &Path) -> AppResult<Self> {
        std::fs::create_dir_all(report_dir)
            .map_err(|e| AppError::io("Buat direktori report", e))?;

        Ok(Self {
            report_dir: report_dir.to_path_buf(),
            buffer: Mutex::new(Vec::new()),
            max_buffer: 50,
        })
    }

    /// Tulis satu audit entry (atomic - langsung ke file)
    pub fn write(&self, entry: AuditEntry) {
        // Serialisasi ke JSON satu baris
        let line = match serde_json::to_string(&entry) {
            Ok(l) => l,
            Err(e) => {
                error!(error = %e, "Gagal serialisasi audit entry");
                return;
            }
        };

        let date_str = entry.timestamp.format("%Y-%m-%d").to_string();
        let file_path = self.report_dir.join(format!("audit_{date_str}.jsonl"));

        // Tulis ke file dengan append (atomic per baris)
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
        {
            Ok(mut f) => {
                if let Err(e) = writeln!(f, "{line}") {
                    error!(error = %e, path = %file_path.display(), "Gagal tulis audit");
                    // Fallback ke buffer memori
                    self.buffer_entry(entry);
                }
            }
            Err(e) => {
                error!(error = %e, path = %file_path.display(), "Gagal buka file audit");
                self.buffer_entry(entry);
            }
        }
    }

    /// Buffer entry di memori jika file tidak bisa ditulis
    fn buffer_entry(&self, entry: AuditEntry) {
        if let Ok(mut buf) = self.buffer.lock() {
            if buf.len() < self.max_buffer {
                buf.push(entry);
            } else {
                warn!("Buffer audit penuh, entry dibuang");
            }
        }
    }

    /// Flush buffer ke file (dipanggil periodik atau saat shutdown)
    pub fn flush(&self) {
        let entries = {
            match self.buffer.lock() {
                Ok(mut buf) => std::mem::take(&mut *buf),
                Err(_) => return,
            }
        };

        for entry in entries {
            // Re-tulis tanpa fallback ke buffer
            let line = match serde_json::to_string(&entry) {
                Ok(l) => l,
                Err(_) => continue,
            };
            let date_str = entry.timestamp.format("%Y-%m-%d").to_string();
            let file_path = self.report_dir.join(format!("audit_{date_str}.jsonl"));
            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&file_path) {
                let _ = writeln!(f, "{line}");
            }
        }
    }

    /// Bersihkan file audit lebih dari N hari
    pub fn cleanup_old_files(&self, retention_days: u32) {
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        if let Ok(entries) = std::fs::read_dir(&self.report_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                    if let Ok(meta) = std::fs::metadata(&path) {
                        if let Ok(modified) = meta.modified() {
                            let modified_utc: DateTime<Utc> = modified.into();
                            if modified_utc < cutoff {
                                let _ = std::fs::remove_file(&path);
                                tracing::info!(path = %path.display(), "File audit lama dihapus");
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Singleton global audit writer (opsional)
use std::sync::OnceLock;
pub static GLOBAL_AUDIT: OnceLock<AuditWriter> = OnceLock::new();

/// Inisialisasi global audit writer
pub fn init_global_audit(report_dir: &Path) -> AppResult<()> {
    let writer = AuditWriter::new(report_dir)?;
    GLOBAL_AUDIT.set(writer)
        .map_err(|_| AppError::System("Audit writer sudah diinisialisasi".to_string()))?;
    Ok(())
}

/// Tulis ke global audit writer (jika sudah diinisialisasi)
pub fn audit(entry: AuditEntry) {
    if let Some(writer) = GLOBAL_AUDIT.get() {
        writer.write(entry);
    }
}
