--- Events
CREATE TABLE IF NOT EXISTS event (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    date TEXT NOT NULL,
    kind TEXT NOT NULL
);

--- Application
CREATE TABLE IF NOT EXISTS application (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    eventId INTEGER NOT NULL,
    kind TEXT NOT NULL,
    name TEXT NOT NULL,
    title TEXT,
    top INTEGER,
    left INTEGER,
    width INTEGER,
    height INTEGER
);
