
--- Application
CREATE TABLE IF NOT EXISTS application (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id INTEGER,
    process_id INTEGER,
    name TEXT,
    title TEXT,
    x INTEGER,
    y INTEGER,
    width INTEGER,
    height INTEGER,
    ref_id INTEGER
);
