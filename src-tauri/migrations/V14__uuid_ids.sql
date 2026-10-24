-- Sessions, sets done and sets to do are identified by a UUIDv7 instead of an integer. A session was identified by
-- its start in seconds: it now has a row of its own in `sessions`, which keeps that start, and the tables that
-- refer to it are removed with it. The single row of the session in progress is kept by a unique index on a
-- constant instead of its id fixed to 1. The ids of exercises, games and events, already text, stay as they are.
--
-- The UUIDv7 of an existing row is built from the time it carries, in milliseconds, followed by a 12-bit sequence
-- that keeps the order of rows of the same time, then by random bits.

ALTER TABLE sets RENAME TO old_sets;
ALTER TABLE session_ends RENAME TO old_session_ends;
ALTER TABLE session_events RENAME TO old_session_events;
ALTER TABLE session_jokers RENAME TO old_session_jokers;
ALTER TABLE session_durations RENAME TO old_session_durations;
ALTER TABLE current_session RENAME TO old_current_session;
ALTER TABLE current_set_items RENAME TO old_current_set_items;
ALTER TABLE current_sets RENAME TO old_current_sets;

-- UUIDv7 of every session, by its start.
CREATE TABLE session_ids (
    old INTEGER PRIMARY KEY,
    id TEXT NOT NULL
);
INSERT INTO session_ids (old, id)
SELECT old, printf('%08x-%04x-7%03x-%s%s-%s', (old * 1000) >> 16, (old * 1000) & 65535, random() & 4095,
                   substr('89ab', 1 + (random() & 3), 1), substr(lower(hex(randomblob(2))), 2),
                   lower(hex(randomblob(6))))
FROM (SELECT session AS old FROM old_sets
      UNION SELECT session FROM old_session_ends
      UNION SELECT session FROM old_session_events
      UNION SELECT session FROM old_session_jokers
      UNION SELECT session FROM old_session_durations
      UNION SELECT started_at FROM old_current_session);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    started_at INTEGER NOT NULL
);
CREATE INDEX sessions_started_at ON sessions(started_at);
INSERT INTO sessions (id, started_at) SELECT id, old FROM session_ids ORDER BY old;

CREATE TABLE sets (
    id TEXT PRIMARY KEY,
    at INTEGER NOT NULL,
    session TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    game TEXT NOT NULL,
    event TEXT NOT NULL,
    exercise TEXT NOT NULL,
    unit TEXT NOT NULL,
    amount INTEGER NOT NULL
);
INSERT INTO sets (id, at, session, game, event, exercise, unit, amount)
SELECT printf('%08x-%04x-7%03x-%s%s-%s', (s.at * 1000) >> 16, (s.at * 1000) & 65535,
              (row_number() OVER (PARTITION BY s.at ORDER BY s.id) - 1) % 4096,
              substr('89ab', 1 + (random() & 3), 1), substr(lower(hex(randomblob(2))), 2),
              lower(hex(randomblob(6)))),
       s.at, m.id, s.game, s.event, s.exercise, s.unit, s.amount
FROM old_sets AS s JOIN session_ids AS m ON m.old = s.session
ORDER BY s.id;

CREATE TABLE session_ends (
    session TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
    ended INTEGER NOT NULL
);
INSERT INTO session_ends (session, ended)
SELECT m.id, e.ended FROM old_session_ends AS e JOIN session_ids AS m ON m.old = e.session ORDER BY e.rowid;

CREATE TABLE session_events (
    session TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    event TEXT NOT NULL,
    count INTEGER NOT NULL,
    PRIMARY KEY (session, position)
);
INSERT INTO session_events (session, position, event, count)
SELECT m.id, e.position, e.event, e.count
FROM old_session_events AS e JOIN session_ids AS m ON m.old = e.session ORDER BY e.rowid;

CREATE TABLE session_jokers (
    session TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
    earned INTEGER NOT NULL,
    spent INTEGER NOT NULL
);
INSERT INTO session_jokers (session, earned, spent)
SELECT m.id, j.earned, j.spent FROM old_session_jokers AS j JOIN session_ids AS m ON m.old = j.session ORDER BY j.rowid;

CREATE TABLE session_durations (
    session TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
    counted INTEGER NOT NULL,
    opened INTEGER
);
INSERT INTO session_durations (session, counted, opened)
SELECT m.id, d.counted, d.opened
FROM old_session_durations AS d JOIN session_ids AS m ON m.old = d.session ORDER BY d.rowid;

CREATE TABLE current_session (
    session TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    game TEXT NOT NULL,
    intensity TEXT NOT NULL,
    jokers INTEGER NOT NULL,
    jokers_earned INTEGER NOT NULL,
    jokers_spent INTEGER NOT NULL,
    -- Last moment the app was known to be open: written at each change, every minute and on quitting.
    seen INTEGER NOT NULL,
    session_limit TEXT,
    asked INTEGER NOT NULL,
    asked_time INTEGER NOT NULL,
    limit_full INTEGER NOT NULL,
    asked_known INTEGER NOT NULL
);
CREATE UNIQUE INDEX current_session_single ON current_session ((1));
INSERT INTO current_session
    (session, game, intensity, jokers, jokers_earned, jokers_spent, seen, session_limit, asked, asked_time,
     limit_full, asked_known)
SELECT m.id, c.game, c.intensity, c.jokers, c.jokers_earned, c.jokers_spent, c.seen, c.session_limit, c.asked,
       c.asked_time, c.limit_full, c.asked_known
FROM old_current_session AS c JOIN session_ids AS m ON m.old = c.started_at;

-- UUIDv7 of every set to do, from the start of the session in progress, in the order they were drawn.
CREATE TABLE set_ids (
    old INTEGER PRIMARY KEY,
    id TEXT NOT NULL
);
INSERT INTO set_ids (old, id)
SELECT s.id, printf('%08x-%04x-7%03x-%s%s-%s', (c.started_at * 1000) >> 16, (c.started_at * 1000) & 65535,
                    (row_number() OVER (ORDER BY s.id) - 1) % 4096,
                    substr('89ab', 1 + (random() & 3), 1), substr(lower(hex(randomblob(2))), 2),
                    lower(hex(randomblob(6))))
FROM old_current_sets AS s, old_current_session AS c;

CREATE TABLE current_sets (
    id TEXT PRIMARY KEY,
    position INTEGER NOT NULL,
    game TEXT NOT NULL,
    game_name TEXT NOT NULL,
    event TEXT NOT NULL,
    event_name TEXT NOT NULL,
    jokers INTEGER NOT NULL,
    trigger_count INTEGER NOT NULL,
    max_per_session INTEGER,
    spent INTEGER NOT NULL,
    time INTEGER NOT NULL,
    reduced INTEGER
);
INSERT INTO current_sets
    (id, position, game, game_name, event, event_name, jokers, trigger_count, max_per_session, spent, time, reduced)
SELECT m.id, s.position, s.game, s.game_name, s.event, s.event_name, s.jokers, s.trigger_count, s.max_per_session,
       s.spent, s.time, s.reduced
FROM old_current_sets AS s JOIN set_ids AS m ON m.old = s.id ORDER BY s.id;

CREATE TABLE current_set_items (
    set_id TEXT NOT NULL REFERENCES current_sets(id) ON DELETE CASCADE,
    drawn INTEGER NOT NULL,
    position INTEGER NOT NULL,
    exercise TEXT NOT NULL,
    name TEXT NOT NULL,
    unit TEXT NOT NULL,
    amount INTEGER NOT NULL,
    base INTEGER NOT NULL,
    halved INTEGER NOT NULL,
    done INTEGER NOT NULL,
    halve_cost INTEGER,
    skip_cost INTEGER NOT NULL,
    PRIMARY KEY (set_id, drawn, position)
);
INSERT INTO current_set_items
    (set_id, drawn, position, exercise, name, unit, amount, base, halved, done, halve_cost, skip_cost)
SELECT m.id, i.drawn, i.position, i.exercise, i.name, i.unit, i.amount, i.base, i.halved, i.done, i.halve_cost,
       i.skip_cost
FROM old_current_set_items AS i JOIN set_ids AS m ON m.old = i.set_id ORDER BY i.rowid;

DROP TABLE old_current_set_items;
DROP TABLE old_current_sets;
DROP TABLE old_current_session;
DROP TABLE old_session_durations;
DROP TABLE old_session_jokers;
DROP TABLE old_session_events;
DROP TABLE old_session_ends;
DROP TABLE old_sets;
DROP TABLE set_ids;
DROP TABLE session_ids;

CREATE INDEX sets_at ON sets(at);
CREATE INDEX sets_session ON sets(session);
