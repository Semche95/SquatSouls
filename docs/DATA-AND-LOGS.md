# Data and logs

## Database

Everything is saved in a SQLite database: configuration (including the global recovery, the sound setting, the timer settings, the notification area, Steam detection, its notification and the ignored games), history (sets done, and per session its end, events triggered and jokers), last chosen game and names given by the Steam Store.

| OS | Path |
| --- | --- |
| Linux | `~/.config/com.semche.squatsouls/squatsouls.db` |
| Windows | `%APPDATA%\com.semche.squatsouls\squatsouls.db` |
| macOS | `~/Library/Application Support/com.semche.squatsouls/squatsouls.db` |

The development app, started by `pnpm tauri dev`, uses `squatsouls-dev.db` in the same folder, so it never touches the database of the installed app.

On save, only the modified exercises, games and events are rewritten.

## Migrations

The schema lives in `src-tauri/migrations/`, one `V<n>__<name>.sql` file per migration. At launch, the database receives the schema migrations it is missing, each dated when applied. A migration is never modified once created: every schema change adds a new one. A database created by a newer version of the app is refused. Updates and reinstalls never touch the database folder, so the data is kept from one version to the next.

## Logs

Warnings and errors are written to a log file, in local time: technical detail of the errors shown as internal, database or notification area failures, configuration ignored at launch, crashes, uncaught interface errors.

| OS | Folder |
| --- | --- |
| Linux | `~/.local/share/com.semche.squatsouls/logs` |
| Windows | `%LOCALAPPDATA%\com.semche.squatsouls\logs` |
| macOS | `~/Library/Logs/com.semche.squatsouls` |

The file is `squatsouls.log` (`squatsouls-dev.log` for the development app). Past 1 MB, the file is archived with its date and only the 3 most recent archives are kept.

## Cache

On Linux, `~/.cache/com.semche.squatsouls` holds rebuildable files only: `icon.png`, the notification icon, and for the AppImage `gstreamer-registry.bin`, the list of its bundled sound plugins, kept apart from the system's GStreamer cache.
