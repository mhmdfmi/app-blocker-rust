-- ============================================
-- App Blocker - Logs Database Schema
-- Version: 1.2.0
-- Created: 2026-04-25
-- ============================================

PRAGMA journal_mode = WAL;  -- Write-Ahead Logging for better performance
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000; -- 64MB cache
PRAGMA temp_store = MEMORY;

-- ============================================
-- LOGS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    process_name TEXT NOT NULL,
    process_path TEXT,
    action TEXT NOT NULL CHECK(action IN ('blocked', 'allowed', 'warning', 'error')),
    reason TEXT,
    score INTEGER,
    device_id TEXT,
    user_id INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for faster queries
CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_logs_action ON logs(action);
CREATE INDEX IF NOT EXISTS idx_logs_process_name ON logs(process_name);
CREATE INDEX IF NOT EXISTS idx_logs_device_id ON logs(device_id);

-- Create index for date range queries (YYYY-MM-DD)
CREATE INDEX IF NOT EXISTS idx_logs_date ON logs(substr(timestamp, 1, 10));

-- ============================================
-- AUDIT LOGS TABLE (for security events)
-- ============================================
CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    event_type TEXT NOT NULL,
    user_id INTEGER,
    username TEXT,
    ip_address TEXT,
    details TEXT,  -- JSON
    success INTEGER NOT NULL DEFAULT 1 CHECK(success IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_logs_event_type ON audit_logs(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);

-- ============================================
-- CREATE VIRTUAL TABLE FOR FULLTEXT SEARCH (Optional)
-- ============================================
-- Note: FTS5 is available in SQLite 3.9+
-- Uncomment if you need full-text search on logs

-- CREATE VIRTUAL TABLE IF NOT EXISTS logs_fts USING fts5(
--     process_name,
--     process_path,
--     reason,
--     content='logs',
--     content_rowid='id'
-- );

-- Triggers to keep FTS in sync (if FTS is enabled)
-- CREATE TRIGGER IF NOT EXISTS logs_ai AFTER INSERT ON logs BEGIN
--     INSERT INTO logs_fts(rowid, process_name, process_path, reason)
--     VALUES (new.id, new.process_name, new.process_path, new.reason);
-- END;

-- CREATE TRIGGER IF NOT EXISTS logs_ad AFTER DELETE ON logs BEGIN
--     INSERT INTO logs_fts(logs_fts, rowid, process_name, process_path, reason)
--     VALUES ('delete', old.id, old.process_name, old.process_path, old.reason);
-- END;

-- CREATE TRIGGER IF NOT EXISTS logs_au AFTER UPDATE ON logs BEGIN
--     INSERT INTO logs_fts(logs_fts, rowid, process_name, process_path, reason)
--     VALUES ('delete', old.id, old.process_name, old.process_path, old.reason);
--     INSERT INTO logs_fts(rowid, process_name, process_path, reason)
--     VALUES (new.id, new.process_name, new.process_path, new.reason);
-- END;