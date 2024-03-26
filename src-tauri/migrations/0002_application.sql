--- Application

CREATE TABLE IF NOT EXISTS application_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT
,   event_id INTEGER NOT NULL
,   info_id INTEGER NOT NULL
,   process_id INTEGER
,   title TEXT
,   x INTEGER
,   y INTEGER
,   width INTEGER
,   height INTEGER
,   ref_id INTEGER
);

CREATE TABLE IF NOT EXISTS application_info (
    id INTEGER PRIMARY KEY AUTOINCREMENT
,   path TEXT NOT NULL
,   name TEXT
);
