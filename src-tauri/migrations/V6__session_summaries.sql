-- Events triggered and jokers of each session, kept for the summary shared from the history.
-- Names are copied, like those of `sets`, so that a later configuration change does not alter them.
CREATE TABLE session_events (
    session INTEGER NOT NULL,
    position INTEGER NOT NULL,
    event TEXT NOT NULL,
    count INTEGER NOT NULL,
    PRIMARY KEY (session, position)
);
CREATE TABLE session_jokers (
    session INTEGER PRIMARY KEY,
    earned INTEGER NOT NULL,
    spent INTEGER NOT NULL
);
