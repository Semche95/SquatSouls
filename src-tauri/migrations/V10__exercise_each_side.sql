-- A timed exercise done on each side: its timer runs its duration once per side.
ALTER TABLE exercises ADD COLUMN each_side INTEGER NOT NULL DEFAULT 0;
