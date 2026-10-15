# Behavior

This document describes how Squat Souls behaves: draws, intensity, limits, jokers, sessions, history, configuration, language, Steam detection, notification area, sharing and onboarding.

Squat Souls is meant to be kind and encouraging, and to frame without imposing: it gives a structure (draws, limits, jokers, rest), but each user stays free to adjust it and responsible for their training. Every rule below follows from that.

It is the reference: any change in the app's behavior must be reflected here, in the same change.

Button and setting names are quoted as they appear in the English interface.

## Confirming actions

- An action is confirmed only once: the button that confirms it (**Create**, confirmation of a removal, **End**, etc.) applies and saves it right away. No action already confirmed should wait for a second confirmation, such as the configuration's **Save** button.
- **Save** is only for changes made directly in the fields, which have no confirmation button of their own.
- Confirmation buttons are never hidden, only disabled when there is nothing to confirm: **Save** and **Reset the changes** stay shown, disabled while there is no pending change.
- Creating or importing a game, importing exercises, and the confirmation of a game removal, of the session's end or cancellation and of quitting open in a dialog over the window. Its button that confirms applies the action and closes it; **Cancel** (or the button that keeps), the × when there is one, Escape or a click outside closes it without changing anything. While it is open, the other shortcuts do nothing. Stopping a set is confirmed on its card instead, so as not to weigh on the training.

## Display

- Any section opened or block shown by an action (an exercise or event opened, a nested setting, a confirmation, a message after an action) that ends up partly hidden in the window or under the configuration's save bar scrolls just enough to show it entirely, or as much of it as fits, starting from its top.
- What appears without an action from the user (update notice, Steam offer, new sets in the queue) never scrolls the window, so the page never moves under them.

## Draw

- Each game offers a list of events. Triggering an event draws `draws` distinct exercises among the enabled exercises, each with a whole number picked at random between its `min` and its `max` (inclusive).
- Exercises come out in rounds: within a session, each exercise comes out once before any of them can come back. When a round runs out, a new one starts; it does not immediately pick again an exercise of the previous set, nor an exercise already drawn in the same draw (unless it is the only one left to draw, for the first case). A saved configuration restarts the round.
- The range belongs to the exercise: it is the same whatever the event.
- An exercise switched off in the configuration (toggle at the start of its row) stays defined but is no longer drawn, until it is switched on again. The toggle is saved with **Save**, is not carried by exercise sharing, and an imported exercise arrives enabled (or keeps its state if it already existed).
- A draw becomes a set "to do". Sets to do are handled from the oldest to the most recent.

## Intensity

- The **Intensity** of the session is chosen in Play, next to the game: **Easy**, **Normal** or **Hard**. It scales the number of each exercise drawn, without touching the configuration: half for **Easy**, one and a half for **Hard** (with a range of 10 to 20, **Easy** gives 5 to 10 and **Hard** 15 to 30).
- **Easy** always rounds down, without falling to 0, and **Normal** and **Hard** always round up, **Halve** included: 15 gives 7 in **Easy** and 23 in **Hard**, and a 9 halved gives 4 in **Easy** and 5 otherwise.
- It can change during the session and, as a changed minimum or maximum does, applies right away to the exercises to do: their number is the one drawn, halved first if it was, scaled by the new intensity (18 drawn gives 27 in **Hard** and 9 in **Easy**). Exercises already done never change.
- Hovering a level shows its effect on the number of exercises.
- It appears neither in the summary nor in the history, which show the numbers actually done.
- Each launch starts at **Normal**, unless **Remember the intensity between launches** is on in Settings (off by default, part of the configuration and saved with **Save**): the chosen intensity is then kept in the database, outside the configuration, from one launch to the next.

## Per-session limit

- Each trigger of an event is counted for the session, per game and per event.
- Beyond `maxPerSession` triggers, the event no longer gives a set: a message says the limit is reached. The trigger counter then stays stuck at the limit.

## Handling a set to do

- Sets are done in the order they were drawn: only the oldest offers **Done** and the jokers (the backend refuses them on the others); the next ones, queued, can only be dismissed.
- **Done** is marked exercise by exercise, in the order of the set: only its first remaining exercise offers it (the backend refuses the others), while the jokers stay offered on every remaining exercise. The exercise is added to the totals and written to the history, and stays displayed struck through until the end of the set. The set moves to the sets done when its last exercise is done or skipped, and the event's jokers are earned at that moment.
- **Cancel** a set not started yet (misclick): the set disappears and the trigger is uncounted, as if it never happened. The dismissed set is reserved: the next trigger of the same event gives back the same exercises with the same numbers, as drawn, so that dismissing cannot be used to change the draw. The reserve disappears with the session (end or cancellation); a saved configuration adjusts it like the sets to do.
- **Stop** a started set, after confirmation (**Stop the set**): its exercises already done stay in the totals and the history, the rest of the set disappears. The trigger stays counted, including for the per-session limit, the set earns no jokers and nothing is reserved. Stopping is free, even after a single exercise: each user is responsible for their training, and stopping at the slightest pain must cost nothing.
- **Jokers**: halve or skip an exercise, or skip the whole set (see Jokers). The trigger stays counted, including for the per-session limit.

## Recovery between exercises

- Each exercise done (**Done**) starts a recovery: its own recovery, or otherwise the global recovery (60 s in the default configuration). A 0 s recovery starts none.
- During the recovery, no exercise can be marked done (tooltip "Recovery in progress" on **Done**), neither in the same set nor in another one, including a set drawn during the recovery. Jokers, **Cancel** and **Stop** remain possible and do not start a recovery.
- When the recovery reaches its end, a bell rings twice, like the start of a boxing round. Ending it early or stopping it plays nothing.
- The recovery shows as a countdown on the top border of the current set (or of the empty block), its ring emptying as time passes. Clicking it (tooltip **Skip**) or pressing Escape ends the recovery right away. It also stops when the session ends or is cancelled, but not when the set whose exercise started it stops: that exercise stays done.

## Timer

- In Program, each exercise has a **Timer** choice: **Not timed** (default), **Seconds** or **Minutes**. The number of a timed exercise is a duration in that unit; its unit stays free text, only shown. The default configuration times the planks and the arm circles. On the update that brought the timer, existing exercises whose unit reads as a duration (`s`, `sec`, `seconds`, `secondes`, `min`, `minutes`...) became timed, once: afterwards only the choice counts.
- **Timer** (or T) is offered next to **Done** on the first remaining exercise of the current set, when it is timed. It runs the exercise's current number, halved included. It cannot start during the recovery.
- While the timer runs or is held, **Done** and **Timer** are disabled (tooltip "Timer running"); the jokers, **Cancel** and **Stop** remain possible.
- The timer shows in the recovery's countdown, in ember: first the countdown before the timer ("Ready"), then the exercise's duration. Clicking it or Space holds or resumes it (Space never marks an exercise done); Escape stops it, and it starts over from the beginning at the next start. It also stops as soon as its exercise is no longer the one to do as it was: halved or skipped, its set skipped, dismissed or stopped, the session cancelled.
- A short low note plays when the timer starts, after the countdown. At the end, two rising notes play and the exercise is marked done, which starts the recovery. The **Timer** card of Settings turns off the marking (**Mark the exercise done when the timer ends**, on by default: **Done** is then available again), and sets the **Countdown before the timer (s)** (3 s by default, none at 0). These settings are saved with **Save**.

## Sound

- **Play sounds**, in the **Sound** card of Settings (on by default, with its uses recalled underneath), turned off, silences every sound: those of the timer and the one at the end of the recovery. On the update that brought it, it took the value of the former timer sound setting. It is saved with **Save**.

## Jokers

- Only events with `jokers` earn some, once all the exercises of their set are done: spending a joker on a set (half or skip) makes it lose those it was meant to earn.
- **Halve** (1 joker): divides an exercise's number by two, rounded as the intensity does (down in **Easy**, up otherwise). Only once per exercise, and not for a number of 1.
- **Skip** an exercise (2 jokers, only 1 if it was already halved or cannot be, its number being 1): it is removed from the set, the rest remains to do. Skipping the last exercise removes the set.
- **Skip the set** (set with several remaining exercises): costs the sum of the skips of its remaining exercises; those already done stay done.
- An exercise done can no longer be halved or skipped.
- An action is only possible with enough jokers. What is removed is not done (neither totals nor history). The History tab only shows the jokers earned and used: the sets done list what was done, at its real number, without mentioning any joker.
- Unspent jokers are lost at the end of the session.

## Session

- A session does not start on its own, so that its duration only counts the time played: it starts with **Start the session**, shown in place of **End the session** until then, or with its first triggered event. When the app opens and after each end or cancellation, the next session waits to be started.
- The game is locked on the first triggered event: it can no longer change before the next session. The last chosen game is remembered from one launch to the next. On the very first launch, or if that game no longer exists or has no event left, no game is selected: it must be chosen before triggering an event. A game without event is saved, but the game choice shows it greyed out with "(no event)" and does not let it be chosen, nor the Steam detection select it.
- **End the session** is only possible with at least one set done and no set to do (the backend refuses it otherwise). The end time is saved in the history.
- **Cancel the session** resets everything (counters, jokers, sets to do and done) and removes the sets of this session from the history. The button stays disabled until the session has started. It asks for confirmation once the session has a set, done or to do; without any (for example after dismissing the only set), nothing is lost and it cancels right away.
- A saved configuration applies immediately, without restarting. It touches neither the sets done nor the counters (renaming a game or an event keeps its counters). The sets to do stay as drawn, with up-to-date names, except:
  - a removed event removes its sets to do;
  - a changed number of drawn exercises (`draws`) adjusts its sets: the missing exercises are drawn, or exercises not done yet are removed; a set with nothing left to do is finished and earns its jokers;
  - a removed or disabled exercise is replaced by another enabled exercise drawn at random, absent from the set;
  - a changed minimum or maximum brings the number of an exercise to do back within the new bounds (scaled by the intensity), unless it was halved;
  - changed jokers apply to the event's sets, except those on which jokers were already spent, which no longer earn any.

  Exercises already done never change. If it removes the game of the current session, the session is cancelled (as with **Cancel the session**) and the game must be chosen again.

## History and summary

- Each exercise done is written to the history (one entry per exercise), with the time, the session, the game, the event, the exercise and the amount. It is kept from one launch to the next.
- The summary groups sets by session, under the local day the session started (a session past midnight stays whole on that day), then by game, with totals per exercise (cumulated amount and number of sets). Each session shows its start time.
- A session's duration goes from its start to its end time if it was ended, otherwise to its last set.
- The summary stays positive: it shows what was done, never what was skipped. The history counts the days of activity since the beginning (days on which a session with sets done started), a number that only goes up: no streak of consecutive days that could be lost, and a day without a set reads "rest".
- Once a session ends, the Play tab shows an encouragement picked at random, translated, above the session buttons, until the next session starts.
- **Copy the summary** copies to the clipboard a text to paste in a messaging app, for the current session (once a set is done) and for each session of the history. One line per item: the game and the duration, the events triggered with their number (in the order of the game's events), the total per exercise, then the jokers earned and used. The current session lasts until the copy. The events and jokers of each session are saved in the history as the session goes; sessions saved before this was recorded leave out those two lines.

## Configuration

- **Program** (games, events, exercises) and **Settings** are separate: each tab's **Save** and **Reset the changes** only apply to its own changes.
- The configuration is validated before being saved. It is refused if: there is no exercise, no enabled exercise or no game, an id or a name is empty, an id is duplicated, two games have the same name (ignoring case and surrounding spaces), `draws` is 0 or exceeds the number of enabled exercises, `maxPerSession` is 0, an exercise has `max` at 0 or `min` greater than `max`, a name (game, event, exercise) exceeds 300 characters or a unit 50, a name or unit contains a control or invisible character (line break, tab, zero width character, text direction override; the zero width joiner of emoji sequences is allowed), or there are more than 100 exercises or more than 100 events in a game. The name and unit fields do not accept more characters than allowed.
- A configuration saved before these limits is brought within them at load rather than refused: control and invisible characters are removed, names and units are cut at the limit, exercises and events beyond 100 are dropped, and a name left empty takes its id. It is saved this way at the next **Save**.
- Each number field (recovery times, countdown before the timer, minimum, maximum, required exercises, limit, jokers) must be a whole number, 0 or more, and not too large. This format is checked while typing: a badly filled field (text, negative or decimal number) gets a red border and the problem shows right away in a tooltip while typing; once the field is left, the tooltip closes and reopens on hover. Other errors and confirmations only appear on save. While a field is badly filled, **Save** is disabled and its tooltip names the first faulty field (for example "Squats › minimum: a whole number is needed, 0 or more."). A cleared required field is 0, a cleared optional field (own recovery, limit) is removed.
- Games are always listed by name, ignoring case and accents, in the Play tab as in the Program tab. Without a chosen game, the configuration opens on the first one in the list.
- In Program, the game is chosen in a select, like in Play. **Rename** turns it into a text field with the caret at the end of the name: **Done** (or Enter) applies the new name, which waits for **Save** like the other changes, and **Cancel** (or Esc) keeps the old one. **Done** is disabled while the name is empty or already taken by another game.
- A new database (first launch, no saved game) receives the default configuration, described in `src-tauri/src/app/seed.en.json` and `seed.fr.json` (same ids, numbers and AppIds, only the names and units differ), in the effective language at that launch (see Language). If the saved configuration is unreadable or invalid, the default configuration in the effective language is used and the error is shown.

## Language

- The **Language** setting, first card of Settings, has three values: **System language** (default), **English** and **Français**. The effective language is always English or French.
- With the system language, the effective language is French if the system locale starts with `fr` (`fr`, `fr-FR`, `fr_CA.UTF-8`...), English otherwise. It is resolved by the backend at each launch, never frozen in the database. A missing or unknown saved value counts as the system language.
- The setting is saved in the database outside the configuration: a change applies right away (interface, notification area menu, notifications) and is saved immediately, without going through **Save** and without touching the pending changes. If saving fails, the error is shown and the list goes back to the saved value.
- Dates, times and the alphabetical order of games follow the effective language. Data (names of games, events and exercises, units) is never translated: the default configuration is only chosen at the first launch, and a language change never modifies an existing database.
- The backend holds no translated text: it only sends error codes with their values, and the interface translates them (a generic message for an unknown code). Technical errors (database, files) only show a generic message, their detail goes to the logs. The notification area menu and the notifications use texts sent by the interface: until it has sent them, the backend creates neither the icon nor a notification.

## Steam game detection

- Optional, disabled by default: the **Detect the running Steam game** toggle in Settings is part of the configuration and is saved with **Save**.
- When enabled, the app reads the processes every 10 seconds and looks for the AppId of a game launched by Steam (`SteamLaunch AppId=…` in its command line, or `SteamAppId` in its environment). Without a result and if a Steam process is running, it takes the AppId Steam records for the running game (`RunningAppID` in the registry on Windows, in `registry.vdf` on macOS and Linux), for games whose environment is unreadable. Only Steam games are detected, and no game list is bundled.
- A configured game is recognized by its Steam AppId. Several games can carry the same one: a game in the configuration is a challenge setup, and the same Steam game can have several. A game without AppId is recognized by its name, which it repeats or extends without an adjoining letter ("Megabonk 2" for "Megabonk"), ignoring case and surrounding spaces: the name of the games carrying the detected AppId, or otherwise the one given by the Steam Store. The games of the default configuration have their AppId, except Bloodborne, absent from Steam.
- Configured game: the Play tab offers one button per recognized game to select it, unless one of them already is or the session's game is locked. Recognized by its name, it then receives the detected AppId, saved right away.
- Unknown game: its name is asked from the Steam Store API in the interface language (without the ™ and ® marks, the control and invisible characters, and cut at 300 characters like any name; a name already obtained is kept, even after a language change), then the Play tab offers to **Add** or **Ignore** it. **Add** opens game creation, prefilled with that name and the AppId, which is saved with **Create**. **Ignore** adds it right away to the ignored games, which are never offered again.
- Settings lists the ignored games: **Add** opens the same creation, and the created game leaves the list; **Remove** removes it from the list right away, and it will be offered again.
- To never get blocked by Steam, the Steam Store is only called for an unknown, non-ignored game: a name obtained is never asked again; a game it did not name, or a request left unanswered, is asked again an hour later at the earliest; at most one call per minute, with a 10 s maximum wait. If it refuses for too many requests (status 429 or 403), it is not called for 24 h: unknown games are no longer offered, configured games are still detected. These delays and the names obtained are kept in the database, from one launch to the next.
- **Notify when a game is launched**, visible under the detection toggle when it is on, and on by default, sends a system notification only once per launch: for a configured game ("Select it in Play to start the session.") or an unknown one ("add it from Play") once its name is known, never for an ignored game nor when one of the games recognized is already selected: selecting another one then does not notify it until it is relaunched. A game relaunched after no game has been running is notified again. It is saved with **Save**.
- Under the toggle, Settings shows the status of the saved detection: the detected game (even if already selected or ignored), no game, or a Steam Store rate-limiting requests. If the processes are unreadable, nothing is detected; the next scan tries again.

## Notification area

- Optional, disabled by default: the **Show the icon in the notification area** toggle in Settings is part of the configuration and is saved with **Save**, which adds or removes the icon right away.
- When on, it unfolds two toggles, off by default: **Closing the window hides it instead of quitting** (the app and Steam detection keep running) and **Start hidden** (the app starts without a window). Without the icon, they have no effect: closing the window quits the app, which always starts with its window shown.
- A click on the icon shows the window; its menu offers **Show** and **Quit** (on Linux, only the menu is available).
- **Ctrl+Q** (**Cmd+Q** on macOS) quits the app completely, from any tab, even when closing the window only hides it.
- During a session (started and not ended), quitting asks for confirmation in a dialog (see Confirming actions): **Ctrl+Q**, **Quit** in the icon menu (which shows the window) and closing the window when it quits the app. Its sets to do and jokers are lost, its sets done stay in the history. **Quit** or a second **Ctrl+Q** quits, **Keep the session** closes the dialog.

## Sharing

- A game is shared on its own: its name, its Steam AppId if it has one, and its events (drawn number, limit, jokers), without the exercises, which are personal. A game without event cannot be shared. Exercises are shared separately: name, unit, minimum, maximum, own recovery and timer. The global recovery and the timer settings are not shared.
- **Share** copies to the clipboard a compressed text to pass on (unreadable as is, it withstands the line breaks added by messaging apps), taken from the saved configuration (pending changes are not included). It starts with a short identifier that tells the content: `squatsouls/game/elden-ring:` or `squatsouls/exercises:`. This identifier is only informative: the import ignores it and reads the format version and the content type in the compressed text. The import reads each text according to its version and refuses, asking to update the app, a version it does not know. In a text shared before exercises could be timed, an exercise whose unit reads as a duration is timed, as on the update that brought the timer.
- **Import** opens a dialog asking to paste this text and is only possible if it is valid. A pasted text is never trusted: it must follow the configuration limits (names, units, number of exercises and events, characters), and a text or decompressed content over 256 KiB is refused without being fully decompressed. Two events of the game, or two exercises, with the same name (ignoring case and surrounding spaces) do not block the import: only the first is kept. A game pasted into the exercise import, or the reverse, is refused with a message that says where to import it. The import is saved right away, without going through **Save**, and keeps the other pending changes.
- An imported game is added, or replaces the game with the same name (ignoring case and surrounding spaces): the form announces it before the import. Without a Steam AppId in the import, the replaced game keeps its own. An event that draws more exercises than are defined is capped at that number, and the confirmation message reports it.
- Imported exercises update the unit, minimum, maximum, own recovery and timer of those with the same name (without an own recovery in the import, the exercise goes back to the global recovery; without a timer, it is no longer timed) and add the others; no exercise is removed.

## Update

- At launch, the installed app looks for a newer version among the published releases and downloads it in the background. Offline or on failure, nothing is shown: only a warning is logged. The development app, and on Linux the `.deb` and `.rpm` packages (updated by their package manager), never look for one.
- Once downloaded, a banner at the top of the window announces that the version will be installed on quitting, with a **Restart now** button that installs it and restarts the app. During a session, the button is disabled, and its tooltip says it is available once the session is over.
- Quitting the app (whatever the way) installs the downloaded update, without restarting it. An update never interrupts a session.
- The database is kept from one version to the next: the new version applies its missing migrations at launch.

## About

- **About**, last entry of the menu (at the bottom of the sidebar, right above the session status), opens a window without leaving the current tab: a description of the app, its author, its version, a notice that the project is not affiliated with any game publisher, the **Safety advice** of the onboarding, and a **Source code on GitHub** link, which opens the repository in the default browser. Its cross in the top right corner or Escape closes it.

## Onboarding

- At the first launch, existing databases included, the onboarding covers the whole window, in four steps shown one at a time, with **Back** and **Next**:
  - the concept of the app in one line;
  - **Not a fitness app**: the user fills in the program, and the exercises already there are only examples to adjust before starting;
  - **Take care of yourself**: a doctor's advice if in doubt, warming up and stretching, form before numbers, splitting long sets, breaks and water, stopping at the slightest pain;
  - **No need to git gud**: playing badly is not punished, doing more than planned brings nothing, taking a break is allowed.
- It can only be left from its last step: **See the program** opens the exercises of the **Program** tab, **Let's go** stays on the current tab. It is then never shown again, and its safety advice stays in **About**.
