--- Screenshot

CREATE TABLE IF NOT EXISTS screenshot (
    id INTEGER PRIMARY KEY
,   event_id INTEGER NOT NULL UNIQUE
,   monitor_id INTEGER
,   FOREIGN KEY(event_id) REFERENCES event_log ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS screenshot_event_id ON screenshot (event_id);
