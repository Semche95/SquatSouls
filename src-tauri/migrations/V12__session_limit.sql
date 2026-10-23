-- Limit of the session in progress, as `kind:value`, NULL without one; the exercises asked and their estimated
-- time in seconds, against which it applies; and whether a set did not fit entirely within a limit in training
-- time. A session saved before the limit has none, and nothing asked.
ALTER TABLE current_session ADD COLUMN session_limit TEXT;
ALTER TABLE current_session ADD COLUMN asked INTEGER NOT NULL DEFAULT 0;
ALTER TABLE current_session ADD COLUMN asked_time INTEGER NOT NULL DEFAULT 0;
ALTER TABLE current_session ADD COLUMN limit_full INTEGER NOT NULL DEFAULT 0;

-- Estimated time of the exercises a set asked, in seconds, and the exercises the session's limit left it, NULL when
-- it did not reduce the set.
ALTER TABLE current_sets ADD COLUMN time INTEGER NOT NULL DEFAULT 0;
ALTER TABLE current_sets ADD COLUMN reduced INTEGER;
