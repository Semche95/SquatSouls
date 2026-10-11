-- Un exercice décoché reste défini mais sort du tirage.
ALTER TABLE exercises ADD COLUMN enabled INTEGER NOT NULL DEFAULT 1;
