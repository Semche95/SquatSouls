-- Adds `retry_at` to databases whose `steam_names` lacks it (V3 draft), without failing on the others
-- or losing their dates: in the subquery, `retry_at` refers to the `steam_names` column if it
-- exists, otherwise to the NULL of the enclosing query.
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
