-- Ajoute `retry_at` aux bases dont `steam_names` en manque (brouillon de V3), sans échouer sur les autres
-- ni perdre leurs dates : dans la sous-requête, `retry_at` désigne la colonne de `steam_names` si elle
-- existe, sinon le NULL de la requête englobante.
CREATE TABLE steam_names_new (
    app_id INTEGER PRIMARY KEY,
    name TEXT,
    retry_at INTEGER
);
INSERT INTO steam_names_new (app_id, name, retry_at)
SELECT app_id, name, (SELECT retry_at FROM steam_names AS s WHERE s.app_id = old.app_id)
FROM (SELECT app_id, name, NULL AS retry_at FROM steam_names) AS old;
DROP TABLE steam_names;
ALTER TABLE steam_names_new RENAME TO steam_names;
