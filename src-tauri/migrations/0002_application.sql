--- Application

CREATE TABLE IF NOT EXISTS application_info (
    id INTEGER PRIMARY KEY
,   path TEXT NOT NULL UNIQUE
,   name TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS application_info_path ON application_info (path);

CREATE TABLE IF NOT EXISTS application_log (
    id INTEGER PRIMARY KEY
,   event_id INTEGER NOT NULL UNIQUE
,   info_id INTEGER NOT NULL
,   process_id INTEGER
,   title TEXT
,   x INTEGER
,   y INTEGER
,   width INTEGER
,   height INTEGER
,   ref_id INTEGER
,   FOREIGN KEY(event_id) REFERENCES event_log ON DELETE CASCADE
,   FOREIGN KEY(info_id) REFERENCES application_info
,   FOREIGN KEY(ref_id) REFERENCES application_log
);

CREATE UNIQUE INDEX IF NOT EXISTS application_log_event_id ON application_log (event_id);
CREATE INDEX IF NOT EXISTS application_log_info_id ON application_log (info_id);
CREATE INDEX IF NOT EXISTS application_log_ref_id ON application_log (ref_id);
CREATE INDEX IF NOT EXISTS application_log_process_id ON application_log (process_id);
