CREATE TABLE IF NOT EXISTS funds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    note TEXT,
    group_name TEXT,
    tags TEXT,
    enabled INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_funds_enabled ON funds (enabled);
CREATE INDEX IF NOT EXISTS idx_funds_group_name ON funds (group_name);

CREATE TABLE IF NOT EXISTS fund_quotes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    unit_nav REAL,
    estimated_nav REAL,
    change_rate REAL,
    fetched_at TEXT NOT NULL,
    source TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (fund_id) REFERENCES funds (id)
);

CREATE INDEX IF NOT EXISTS idx_fund_quotes_fund_id_fetched_at
    ON fund_quotes (fund_id, fetched_at DESC);

CREATE TABLE IF NOT EXISTS monitor_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER,
    group_name TEXT,
    rule_type TEXT NOT NULL,
    threshold_config TEXT NOT NULL,
    enabled INTEGER NOT NULL,
    cooldown_minutes INTEGER NOT NULL,
    last_triggered_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (fund_id) REFERENCES funds (id)
);

CREATE INDEX IF NOT EXISTS idx_monitor_rules_enabled ON monitor_rules (enabled);
CREATE INDEX IF NOT EXISTS idx_monitor_rules_fund_id ON monitor_rules (fund_id);

CREATE TABLE IF NOT EXISTS alert_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id INTEGER NOT NULL,
    fund_id INTEGER NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL,
    triggered_at TEXT NOT NULL,
    notification_result TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (rule_id) REFERENCES monitor_rules (id),
    FOREIGN KEY (fund_id) REFERENCES funds (id)
);

CREATE INDEX IF NOT EXISTS idx_alert_events_status_triggered_at
    ON alert_events (status, triggered_at DESC);
CREATE INDEX IF NOT EXISTS idx_alert_events_fund_id_triggered_at
    ON alert_events (fund_id, triggered_at DESC);

CREATE TABLE IF NOT EXISTS job_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_type TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_job_runs_started_at ON job_runs (started_at DESC);

CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
