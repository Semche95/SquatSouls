-- The recovery between exercises is named `recovery`, `pause` being kept for holding the timer.
ALTER TABLE exercises RENAME COLUMN pause TO recovery;
UPDATE settings SET key = 'recovery' WHERE key = 'pause';
