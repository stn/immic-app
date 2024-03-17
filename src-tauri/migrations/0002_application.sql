
--- Application
CREATE TABLE IF NOT EXISTS application (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    eventId INTEGER,
    kind TEXT,
    processId INTEGER,
    name TEXT,
    title TEXT,
    x INTEGER,
    y INTEGER,
    width INTEGER,
    height INTEGER
);
