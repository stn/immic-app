--- Browser

CREATE TABLE IF NOT EXISTS browser_info (
    id INTEGER PRIMARY KEY
,   url TEXT NOT NULL UNIQUE
,   fav_icon_url TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS browser_info_url ON browser_info (url);

CREATE TABLE IF NOT EXISTS browser_log (
    id INTEGER PRIMARY KEY
,   event_id INTEGER NOT NULL UNIQUE
,   info_id INTEGER NOT NULL
,   title TEXT
,   referrer_id INTEGER
,   tab_id INTEGER
,   opener_tab_id INTEGER
,   window_id INTEGER
,   FOREIGN KEY(event_id) REFERENCES event_log ON DELETE CASCADE
,   FOREIGN KEY(info_id) REFERENCES browser_info
,   FOREIGN KEY(referrer_id) REFERENCES browser_info
);

CREATE UNIQUE INDEX IF NOT EXISTS browser_log_event_id ON browser_log (event_id);
CREATE INDEX IF NOT EXISTS browser_log_info_id ON browser_log (info_id);
CREATE INDEX IF NOT EXISTS browser_log_referrer_id ON browser_log (referrer_id);
