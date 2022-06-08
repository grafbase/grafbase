CREATE TABLE IF NOT EXISTS records (
    id TEXT PRIMARY KEY,
    type TEXT,
    document JSON,
    created_at TEXT,
    updated_at TEXT
);
