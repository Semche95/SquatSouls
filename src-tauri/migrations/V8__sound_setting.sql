-- The timer sound setting becomes the global sound setting (timer and end of recovery), keeping its value.
UPDATE settings SET key = 'sound' WHERE key = 'timer_sound';
