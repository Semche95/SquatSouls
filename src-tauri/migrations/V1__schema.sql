CREATE TABLE IF NOT EXISTS exercises (
    id TEXT PRIMARY KEY,
    position INTEGER NOT NULL,
    name TEXT NOT NULL,
    unit TEXT NOT NULL,
    min INTEGER NOT NULL,
    max INTEGER NOT NULL,
    pause INTEGER
);
CREATE TABLE IF NOT EXISTS games (
    id TEXT PRIMARY KEY,
    position INTEGER NOT NULL,
    name TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS events (
    game TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    id TEXT NOT NULL,
    position INTEGER NOT NULL,
    name TEXT NOT NULL,
    draws INTEGER NOT NULL,
    max_per_session INTEGER,
    jokers INTEGER NOT NULL,
    PRIMARY KEY (game, id)
);
CREATE TABLE IF NOT EXISTS sets (
    id INTEGER PRIMARY KEY,
    at INTEGER NOT NULL,
    session INTEGER NOT NULL,
    game TEXT NOT NULL,
    event TEXT NOT NULL,
    exercise TEXT NOT NULL,
    unit TEXT NOT NULL,
    amount INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS sets_at ON sets(at);
CREATE INDEX IF NOT EXISTS sets_session ON sets(session);
CREATE TABLE IF NOT EXISTS session_ends (
    session INTEGER PRIMARY KEY,
    ended INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
