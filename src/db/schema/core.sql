-- ============================================
-- App Blocker - Core Database Schema
-- Version: 1.2.1
-- Created: 2026-04-25
-- ============================================

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- ============================================
-- USERS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user' CHECK(role IN ('admin', 'user')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Index for faster lookups
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);

-- ============================================
-- CONFIGS TABLE (Key-Value Store)
-- ============================================
CREATE TABLE IF NOT EXISTS configs (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================
-- BLACKLIST TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS blacklist (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_blacklist_enabled ON blacklist(enabled);

-- ============================================
-- BLACKLIST PROCESSES TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS blacklist_processes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    blacklist_id INTEGER NOT NULL,
    process_name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (blacklist_id) REFERENCES blacklist(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_blacklist_processes_blacklist_id ON blacklist_processes(blacklist_id);
CREATE INDEX IF NOT EXISTS idx_blacklist_processes_name ON blacklist_processes(process_name);

-- ============================================
-- BLACKLIST PATHS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS blacklist_paths (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    blacklist_id INTEGER NOT NULL,
    path TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (blacklist_id) REFERENCES blacklist(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_blacklist_paths_blacklist_id ON blacklist_paths(blacklist_id);

-- ============================================
-- WHITELIST TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS whitelist (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    process_name TEXT NOT NULL UNIQUE,
    description TEXT,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_whitelist_enabled ON whitelist(enabled);

-- ============================================
-- SCHEDULE TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS schedule (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
    timezone TEXT NOT NULL DEFAULT 'Asia/Jakarta',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================
-- SCHEDULE RULES TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS schedule_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    schedule_id INTEGER NOT NULL,
    days TEXT NOT NULL,  -- JSON array: ["Monday", "Tuesday"]
    start_time TEXT NOT NULL,  -- HH:MM format
    end_time TEXT NOT NULL,    -- HH:MM format
    action TEXT NOT NULL,      -- "block_games", "block_all", etc.
    enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (schedule_id) REFERENCES schedule(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_schedule_rules_schedule_id ON schedule_rules(schedule_id);
CREATE INDEX IF NOT EXISTS idx_schedule_rules_enabled ON schedule_rules(enabled);

-- ============================================
-- INSERT DEFAULT DATA
-- ============================================

-- Insert default admin user (password: Admin12345! - should be changed!)
-- Password hash for 'Admin12345!' using argon2
INSERT OR IGNORE INTO users (username, password_hash, role) VALUES
('admin', '$argon2id$v=19$m=19456,t=2,p=1$phv4zmAQxu/cwVdRY9wgLg$441jRs24dn+kSxOf4K21qGzrsqb2rbtPsFdR5rvCMug', 'admin');

-- Insert default schedule (only if schedule table is empty)
INSERT OR IGNORE INTO schedule (id, enabled, timezone) VALUES
(1, 1, 'Asia/Jakarta');

-- Insert default schedule rules
INSERT OR IGNORE INTO schedule_rules (schedule_id, days, start_time, end_time, action, enabled) VALUES
(1, '["Monday","Tuesday","Wednesday","Thursday","Friday"]', '07:00', '15:00', 'block_games', 1),
(1, '["Saturday"]', '07:00', '12:00', 'block_games', 1);