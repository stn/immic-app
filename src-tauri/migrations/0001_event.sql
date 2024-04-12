--- Event

CREATE TABLE IF NOT EXISTS event_log (
    id INTEGER PRIMARY KEY
,   timestamp INTEGER NOT NULL
,   date TEXT NOT NULL -- local date when the event happened
,   kind TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS event_log_timestamp ON event_log (timestamp);
CREATE INDEX IF NOT EXISTS event_log_date ON event_log (date);
CREATE INDEX IF NOT EXISTS event_log_kind ON event_log (kind);
