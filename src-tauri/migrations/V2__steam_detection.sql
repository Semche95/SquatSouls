ALTER TABLE games ADD COLUMN steam_app_id INTEGER;
CREATE TABLE ignored_games (
    app_id INTEGER PRIMARY KEY,
    position INTEGER NOT NULL,
    name TEXT NOT NULL
);
