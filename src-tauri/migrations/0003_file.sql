--- File

CREATE TABLE IF NOT EXISTS file_info (
    id INTEGER PRIMARY KEY
,   path TEXT NOT NULL UNIQUE
);

CREATE UNIQUE INDEX IF NOT EXISTS file_info_path ON file_info (path);

CREATE TABLE IF NOT EXISTS file_log (
    id INTEGER PRIMARY KEY
,   event_id INTEGER NOT NULL UNIQUE
,   info_id INTEGER NOT NULL
,   kind TEXT
,   FOREIGN KEY(event_id) REFERENCES event_log ON DELETE CASCADE
,   FOREIGN KEY(info_id) REFERENCES file_info
);

CREATE UNIQUE INDEX IF NOT EXISTS file_log_event_id ON file_log (event_id);
CREATE INDEX IF NOT EXISTS file_log_info_id ON file_log (info_id);
CREATE INDEX IF NOT EXISTS file_log_kind ON file_log (kind);
