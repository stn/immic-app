
--- Application
CREATE TABLE IF NOT EXISTS application (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    eventId INTEGER NOT NULL,
    kind TEXT NOT NULL,
    processId INTEGER NOT NULL,
    name TEXT NOT NULL,
    title TEXT,
    x0 INTEGER,
    y0 INTEGER,
    x1 INTEGER,
    y1 INTEGER
);
