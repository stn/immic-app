--- Browser

CREATE TABLE IF NOT EXISTS browser_origin (
    id INTEGER PRIMARY KEY
,   origin TEXT NOT NULL UNIQUE
,   fav_icon_url TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS browser_origin_origin ON browser2_origin (origin);

CREATE TABLE IF NOT EXISTS browser_url (
    id INTEGER PRIMARY KEY
,   url TEXT NOT NULL UNIQUE
,   origin_id INTEGER NOT NULL
,   last_update INTEGER
,   FOREIGN KEY(origin_id) REFERENCES browser_origin
);

CREATE UNIQUE INDEX IF NOT EXISTS browser_url_url ON browser2_url (url);
CREATE INDEX IF NOT EXISTS browser_url_origin_id ON browser2_url (origin_id);

CREATE TABLE IF NOT EXISTS browser_log (
    id INTEGER PRIMARY KEY
,   event_id INTEGER NOT NULL UNIQUE
,   origin_id INTEGER NOT NULL
,   url_id INTEGER NOT NULL
,   url_query TEXT
,   title TEXT
,   referrer_id INTEGER
,   tab_id INTEGER
,   opener_tab_id INTEGER
,   window_id INTEGER
,   FOREIGN KEY(event_id) REFERENCES event_log
,   FOREIGN KEY(origin_id) REFERENCES browser_origin
,   FOREIGN KEY(url_id) REFERENCES browser_url
,   FOREIGN KEY(referrer_id) REFERENCES browser_url
);

CREATE UNIQUE INDEX IF NOT EXISTS browser_log_event_id ON browser2_log (event_id);
CREATE INDEX IF NOT EXISTS browser_log_origin_id ON browser2_log (origin_id);
CREATE INDEX IF NOT EXISTS browser_log_url_id ON browser2_log (url_id);
CREATE INDEX IF NOT EXISTS browser_log_referrer_id ON browser2_log (referrer_id);
