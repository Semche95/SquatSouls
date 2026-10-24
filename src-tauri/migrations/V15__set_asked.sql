-- Exercises a set asked when it was triggered, counted against the session's limit and freed if it is dismissed,
-- even once a saved configuration changed the set. NULL for a set saved before, which counts its exercises as drawn.
ALTER TABLE current_sets ADD COLUMN asked INTEGER;
