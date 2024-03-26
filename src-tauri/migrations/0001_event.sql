--- Event

CREATE TABLE IF NOT EXISTS event_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT
,   timestamp INTEGER NOT NULL
,   timeframe INTEGER NOT NULL
,   date TEXT NOT NULL
,   kind TEX NOT NULL
,   log_id INTEGER
);
