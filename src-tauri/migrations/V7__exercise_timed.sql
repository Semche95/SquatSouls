-- Time unit of a timed exercise ('seconds' or 'minutes'): its drawn number is a duration, run by the timer.
-- NULL for an exercise counted otherwise (repetitions...).
ALTER TABLE exercises ADD COLUMN timed TEXT;
-- Existing exercises whose unit reads as a duration become timed once; afterwards only the setting counts.
UPDATE exercises SET timed = 'seconds'
WHERE lower(trim(unit)) IN ('s', 'sec', 'sec.', 'secs', 'second', 'seconds', 'seconde', 'secondes');
UPDATE exercises SET timed = 'minutes'
WHERE lower(trim(unit)) IN ('min', 'min.', 'mins', 'minute', 'minutes');
