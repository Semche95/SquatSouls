-- An unchecked exercise stays defined but is left out of the draw.
ALTER TABLE exercises ADD COLUMN enabled INTEGER NOT NULL DEFAULT 1;
