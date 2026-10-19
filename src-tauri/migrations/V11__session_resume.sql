-- Session in progress, saved at each change so that it resumes when the app reopens, after quitting as after a
-- crash. A single row while a session is in progress, none otherwise. Its sets done are read back from `sets`.
CREATE TABLE current_session (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    started_at INTEGER NOT NULL,
    game TEXT NOT NULL,
    intensity TEXT NOT NULL,
    jokers INTEGER NOT NULL,
    jokers_earned INTEGER NOT NULL,
    jokers_spent INTEGER NOT NULL,
    next_id INTEGER NOT NULL,
    -- Last moment the app was known to be open: written at each change, every minute and on quitting.
    seen INTEGER NOT NULL
);

-- Times each event was triggered, against which its limit applies.
CREATE TABLE current_triggers (
    game TEXT NOT NULL,
    event TEXT NOT NULL,
    count INTEGER NOT NULL,
    PRIMARY KEY (game, event)
);

-- Sets to do, in their order.
CREATE TABLE current_sets (
    id INTEGER PRIMARY KEY,
    position INTEGER NOT NULL,
    game TEXT NOT NULL,
    game_name TEXT NOT NULL,
    event TEXT NOT NULL,
    event_name TEXT NOT NULL,
    jokers INTEGER NOT NULL,
    trigger_count INTEGER NOT NULL,
    max_per_session INTEGER,
    spent INTEGER NOT NULL
);

-- Exercises of the sets to do (`drawn` = 0) and as they were drawn (`drawn` = 1), kept for the reserve.
CREATE TABLE current_set_items (
    set_id INTEGER NOT NULL REFERENCES current_sets(id) ON DELETE CASCADE,
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

-- Exercises of the dismissed sets, given back by the next trigger of the same event.
CREATE TABLE current_reserved (
    game TEXT NOT NULL,
    event TEXT NOT NULL,
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
    PRIMARY KEY (game, event, position)
);

-- Exercises not drawn yet in the current round (`last` = 0), and those of the last set (`last` = 1).
CREATE TABLE current_round (
    last INTEGER NOT NULL,
    position INTEGER NOT NULL,
    exercise TEXT NOT NULL,
    PRIMARY KEY (last, position)
);

-- Time a session was open, without the time the app was closed in between: `counted` seconds for the periods
-- already closed, and the start of the period in progress, NULL once the session ended. Sessions saved before
-- this was recorded have no row.
CREATE TABLE session_durations (
    session INTEGER PRIMARY KEY,
    counted INTEGER NOT NULL,
    opened INTEGER
);
