--- File
CREATE TABLE IF NOT EXISTS file (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id INTEGER NOT NULL,
    kind TEXT,
    path TEXT,
    file_type TEXT
);
