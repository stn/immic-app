--- Events
CREATE TABLE IF NOT EXISTS event (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER,
    date TEXT,
    kind TEXT
);
