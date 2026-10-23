-- Whether the exercises asked of the session in progress were counted from its start: a session saved before they
-- were has them counted again at launch, from what it kept.
ALTER TABLE current_session ADD COLUMN asked_known INTEGER NOT NULL DEFAULT 0;
