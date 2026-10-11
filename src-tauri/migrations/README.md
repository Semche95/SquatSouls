# Migrations du schéma SQLite

Appliquées au lancement par refinery, dans l'ordre de `V<n>`, et datées dans la table `refinery_schema_history`.

**Une migration créée ne se modifie jamais** (ni renommée, ni fusionnée, ni supprimée), même avant d'être commitée et même pour corriger une erreur : toute évolution du schéma passe par un nouveau fichier `V<n+1>__<nom>.sql`.
