--- Browser

CREATE TABLE IF NOT EXISTS browser_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT
,   event_id INTEGER NOT NULL
,   info_id INTEGER NOT NULL
,   title TEXT
,   referrer TEXT
,   tab_id INTEGER
,   opener_tab_id INTEGER
,   window_id INTEGER
);

CREATE TABLE IF NOT EXISTS browser_info (
    id INTEGER PRIMARY KEY AUTOINCREMENT
,   url TEXT NOT NULL
,   fav_icon_url TEXT
);
