--- Browser

CREATE TABLE IF NOT EXISTS browser_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT
,   event_id INTEGER NOT NULL
,   tab_id INTERGERT
,   url TEXT
,   title TEXT
,   fav_icon_url TEXT
,   referrer TEXT
,   opener_tab_id INTEGER
,   window_id INTEGER
);
