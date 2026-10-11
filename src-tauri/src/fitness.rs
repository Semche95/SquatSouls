//! Logique métier : configuration, tirage aléatoire, séance et historique.
//! Aucun lien avec Tauri, ce qui permet de tout tester avec `cargo test`.

use chrono::{DateTime, Duration, Local, NaiveDate};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Exercise {
    pub id: String,
    pub name: String,
    /// Unité affichée après le nombre tiré : "rép.", "s", "rép. par jambe"...
    pub unit: String,
    /// Intervalle du nombre tiré, bornes incluses.
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameEvent {
    pub id: String,
    pub name: String,
    /// Nombre d'exercices différents tirés à chaque déclenchement.
    #[serde(default = "one")]
    pub draws: usize,
    /// Au-delà de ce nombre de déclenchements dans la séance, plus aucune série.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_per_session: Option<u32>,
    /// Récompense : jokers gagnés en validant la série de chaque déclenchement.
    #[serde(default)]
    pub jokers: u32,
}

fn one() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub events: Vec<GameEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub exercises: Vec<Exercise>,
    pub games: Vec<Game>,
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.exercises.is_empty() {
            return Err("Aucun exercice défini.".into());
        }
        let mut exercise_ids = HashSet::new();
        for ex in &self.exercises {
            if ex.id.trim().is_empty() || ex.name.trim().is_empty() {
                return Err("Un exercice a un id ou un nom vide.".into());
            }
            if !exercise_ids.insert(ex.id.as_str()) {
                return Err(format!("L'exercice « {} » est défini deux fois.", ex.id));
            }
            if ex.max == 0 || ex.min > ex.max {
                return Err(format!(
                    "Intervalle invalide pour « {} » ({} à {}).",
                    ex.name, ex.min, ex.max
                ));
            }
        }

        if self.games.is_empty() {
            return Err("Aucun jeu défini.".into());
        }
        let mut game_ids = HashSet::new();
        for game in &self.games {
            if game.id.trim().is_empty() || game.name.trim().is_empty() {
                return Err("Un jeu a un id ou un nom vide.".into());
            }
            if !game_ids.insert(game.id.as_str()) {
                return Err(format!("Le jeu « {} » est défini deux fois.", game.id));
            }
            if game.events.is_empty() {
                return Err(format!("Le jeu « {} » n'a aucun événement.", game.name));
            }
            let mut event_ids = HashSet::new();
            for ev in &game.events {
                let at = format!("{} › {}", game.name, ev.name);
                if ev.id.trim().is_empty() || ev.name.trim().is_empty() {
                    return Err(format!("{} : id ou nom d'événement vide.", game.name));
                }
                if !event_ids.insert(ev.id.as_str()) {
                    return Err(format!(
                        "{} : l'événement « {} » est défini deux fois.",
                        game.name, ev.id
                    ));
                }
                if ev.draws == 0 {
                    return Err(format!("{at} : il faut tirer au moins 1 exercice."));
                }
                if ev.draws > self.exercises.len() {
                    return Err(format!(
                        "{at} : {} exercices à tirer mais seulement {} définis.",
                        ev.draws,
                        self.exercises.len()
                    ));
                }
                if ev.max_per_session == Some(0) {
                    return Err(format!(
                        "{at} : la limite par séance doit valoir au moins 1 (ou être vide)."
                    ));
                }
            }
        }
        Ok(())
    }
}

fn event(id: &str, name: &str, draws: usize, max: Option<u32>) -> GameEvent {
    GameEvent {
        id: id.into(),
        name: name.into(),
        draws,
        max_per_session: max,
        jokers: 0,
    }
}

fn reward(id: &str, name: &str, jokers: u32) -> GameEvent {
    GameEvent {
        jokers,
        ..event(id, name, 2, None)
    }
}

/// Événements communs aux Souls, avec le vocabulaire propre à chaque jeu.
fn souls_events(rest: &str, discovered: &str, level: &str, lost: &str) -> Vec<GameEvent> {
    vec![
        event("mort", "Mort", 1, Some(15)),
        event("repos", rest, 1, None),
        event("decouverte", discovered, 1, None),
        event("niveau", level, 1, None),
        event("perte", lost, 1, None),
        event("boss", "Boss vaincu", 2, None),
        reward("boss-premier", "Boss vaincu au premier essai", 1),
        reward("boss-nohit", "Boss vaincu sans prendre un coup", 2),
    ]
}

pub fn default_config() -> Config {
    let ex = |id: &str, name: &str, unit: &str, min, max| Exercise {
        id: id.into(),
        name: name.into(),
        unit: unit.into(),
        min,
        max,
    };
    let game = |id: &str, name: &str, events| Game {
        id: id.into(),
        name: name.into(),
        events,
    };
    let ds = || {
        souls_events(
            "Repos à un feu connu",
            "Nouveau feu allumé",
            "Montée de niveau",
            "Âmes perdues",
        )
    };
    Config {
        exercises: vec![
            ex("pompes", "Pompes", "rép.", 8, 12),
            ex("tractions", "Tractions", "rép.", 2, 4),
            ex("squats", "Squats", "rép.", 12, 18),
            ex("fentes", "Fentes", "rép. par jambe", 5, 8),
            ex("gainage", "Gainage (planche)", "s", 20, 45),
            ex("superman", "Superman", "rép.", 10, 15),
        ],
        games: vec![
            game(
                "elden-ring",
                "Elden Ring",
                souls_events(
                    "Repos à un site de grâce connu",
                    "Site de grâce découvert",
                    "Montée de niveau",
                    "Runes perdues",
                ),
            ),
            game("ds1", "Dark Souls", ds()),
            game("ds2", "Dark Souls II", ds()),
            game("ds3", "Dark Souls III", ds()),
            game(
                "bloodborne",
                "Bloodborne",
                souls_events(
                    "Repos à une lanterne connue",
                    "Lanterne allumée",
                    "Montée de niveau",
                    "Échos de sang perdus",
                ),
            ),
            game(
                "sekiro",
                "Sekiro",
                souls_events(
                    "Repos à une statue du Sculpteur",
                    "Statue du Sculpteur découverte",
                    "Point de compétence gagné",
                    "Sen et expérience perdus",
                ),
            ),
        ],
    }
}

// ---------------------------------------------------------------------------
// Tirage et séance
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DrawItem {
    pub exercise_id: String,
    pub name: String,
    pub unit: String,
    pub amount: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Draw {
    pub id: u64,
    pub game_id: String,
    pub game_name: String,
    pub event_id: String,
    pub event_name: String,
    pub items: Vec<DrawItem>,
    /// Jokers gagnés à la validation (événement de récompense).
    pub jokers: u32,
    /// Vrai quand la limite de l'événement pour la séance est dépassée.
    pub capped: bool,
    pub trigger_count: u32,
    pub max_per_session: Option<u32>,
}

/// Une série faite, telle qu'enregistrée dans l'historique.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub at: i64,
    /// Début de la séance (0 pour les entrées d'avant ce champ).
    #[serde(default)]
    pub session: i64,
    pub game: String,
    pub event: String,
    pub exercise: String,
    pub unit: String,
    pub amount: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Total {
    pub exercise: String,
    pub unit: String,
    pub amount: u32,
    pub sets: u32,
}

/// Une série sautée grâce à un joker.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Skipped {
    pub at: i64,
    pub game: String,
    pub event: String,
    pub items: Vec<DrawItem>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TriggerCount {
    pub game_id: String,
    pub event_id: String,
    pub count: u32,
}

/// Fin d'une séance terminée, enregistrée dans l'historique à côté de ses séries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionEnd {
    pub session: i64,
    pub ended: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionView {
    pub started_at: i64,
    /// Jeu de la séance, fixé au premier événement.
    pub game_id: Option<String>,
    pub pending: Vec<Draw>,
    pub sets: usize,
    pub totals: Vec<Total>,
    /// Séries faites, la plus récente en premier.
    pub log: Vec<LogEntry>,
    pub triggers: Vec<TriggerCount>,
    pub jokers: u32,
    pub jokers_earned: u32,
    /// Séries sautées, la plus récente en premier.
    pub skipped: Vec<Skipped>,
}

#[derive(Debug, Default)]
pub struct Session {
    pub started_at: i64,
    game_id: Option<String>,
    triggers: HashMap<(String, String), u32>,
    pending: Vec<Draw>,
    done: Vec<LogEntry>,
    jokers: u32,
    jokers_earned: u32,
    skipped: Vec<Skipped>,
    next_id: u64,
}

impl Session {
    pub fn new(now: i64) -> Self {
        Session {
            started_at: now,
            ..Default::default()
        }
    }

    pub fn draw<R: Rng + ?Sized>(
        &mut self,
        config: &Config,
        game_id: &str,
        event_id: &str,
        rng: &mut R,
    ) -> Result<Draw, String> {
        let game = config
            .games
            .iter()
            .find(|g| g.id == game_id)
            .ok_or_else(|| format!("Jeu inconnu : {game_id}"))?;
        if self.game_id.as_ref().is_some_and(|id| id != &game.id) {
            return Err("La séance a déjà commencé sur un autre jeu : lance une nouvelle séance pour changer.".into());
        }
        let ev = game
            .events
            .iter()
            .find(|e| e.id == event_id)
            .ok_or_else(|| format!("Événement inconnu : {event_id}"))?;

        self.game_id = Some(game.id.clone());
        let count = self
            .triggers
            .entry((game.id.clone(), ev.id.clone()))
            .or_insert(0);
        // Une fois la limite atteinte, le compteur ne bouge plus.
        let capped = ev.max_per_session.is_some_and(|max| *count >= max);
        if !capped {
            *count += 1;
        }
        let trigger_count = *count;

        let items = if capped {
            Vec::new()
        } else {
            let picks = rand::seq::index::sample(rng, config.exercises.len(), ev.draws);
            picks
                .iter()
                .map(|i| {
                    let ex = &config.exercises[i];
                    DrawItem {
                        exercise_id: ex.id.clone(),
                        name: ex.name.clone(),
                        unit: ex.unit.clone(),
                        amount: rng.random_range(ex.min..=ex.max),
                    }
                })
                .collect()
        };

        self.next_id += 1;
        let draw = Draw {
            id: self.next_id,
            game_id: game.id.clone(),
            game_name: game.name.clone(),
            event_id: ev.id.clone(),
            event_name: ev.name.clone(),
            items,
            jokers: if capped { 0 } else { ev.jokers },
            capped,
            trigger_count,
            max_per_session: ev.max_per_session,
        };
        if !capped {
            self.pending.push(draw.clone());
        }
        Ok(draw)
    }

    /// Valide une série tirée : elle passe dans les séries faites.
    pub fn complete(&mut self, id: u64, now: i64) -> Result<Vec<LogEntry>, String> {
        let draw = self.take_pending(id)?;
        self.jokers += draw.jokers;
        self.jokers_earned += draw.jokers;
        let entries: Vec<LogEntry> = draw
            .items
            .into_iter()
            .map(|it| LogEntry {
                at: now,
                session: self.started_at,
                game: draw.game_name.clone(),
                event: draw.event_name.clone(),
                exercise: it.name,
                unit: it.unit,
                amount: it.amount,
            })
            .collect();
        self.done.extend(entries.iter().cloned());
        Ok(entries)
    }

    /// Annule un déclenchement fait par erreur : rien n'est compté.
    pub fn dismiss(&mut self, id: u64) -> Result<(), String> {
        let draw = self.take_pending(id)?;
        if let Some(count) = self.triggers.get_mut(&(draw.game_id, draw.event_id)) {
            *count = count.saturating_sub(1);
        }
        Ok(())
    }

    /// Dépense un joker pour sauter une série : le déclenchement reste compté.
    pub fn use_joker(&mut self, id: u64, now: i64) -> Result<(), String> {
        if self.jokers == 0 {
            return Err("Aucun joker disponible.".into());
        }
        let draw = self.take_pending(id)?;
        self.jokers -= 1;
        self.skipped.push(Skipped {
            at: now,
            game: draw.game_name,
            event: draw.event_name,
            items: draw.items,
        });
        Ok(())
    }

    /// Retire les tirages en attente (utile quand la configuration change).
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }

    fn take_pending(&mut self, id: u64) -> Result<Draw, String> {
        let pos = self
            .pending
            .iter()
            .position(|d| d.id == id)
            .ok_or("Cette série n'est plus en attente.")?;
        Ok(self.pending.remove(pos))
    }

    pub fn view(&self) -> SessionView {
        let mut triggers: Vec<TriggerCount> = self
            .triggers
            .iter()
            .filter(|(_, &count)| count > 0)
            .map(|((g, e), &count)| TriggerCount {
                game_id: g.clone(),
                event_id: e.clone(),
                count,
            })
            .collect();
        triggers.sort_by(|a, b| (&a.game_id, &a.event_id).cmp(&(&b.game_id, &b.event_id)));
        SessionView {
            started_at: self.started_at,
            game_id: self.game_id.clone(),
            pending: self.pending.clone(),
            sets: self.done.len(),
            totals: totals(&self.done),
            log: self.done.iter().rev().cloned().collect(),
            triggers,
            jokers: self.jokers,
            jokers_earned: self.jokers_earned,
            skipped: self.skipped.iter().rev().cloned().collect(),
        }
    }
}

/// Cumule les séries par exercice, dans l'ordre de première apparition.
pub fn totals(entries: &[LogEntry]) -> Vec<Total> {
    let mut out: Vec<Total> = Vec::new();
    for e in entries {
        match out
            .iter_mut()
            .find(|t| t.exercise == e.exercise && t.unit == e.unit)
        {
            Some(t) => {
                t.amount += e.amount;
                t.sets += 1;
            }
            None => out.push(Total {
                exercise: e.exercise.clone(),
                unit: e.unit.clone(),
                amount: e.amount,
                sets: 1,
            }),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Historique (une ligne JSON par série faite)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DayTotals {
    /// Date au format AAAA-MM-JJ.
    pub date: String,
    pub sets: usize,
    /// Séances du jour, de la plus récente à la plus ancienne.
    pub sessions: Vec<SessionTotals>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionTotals {
    /// Heure de la première série faite.
    pub start: i64,
    /// Secondes entre le début de la séance (ou la première série) et sa fin (ou la dernière série).
    pub duration: i64,
    pub game: String,
    pub sets: usize,
    pub totals: Vec<Total>,
}

/// Regroupe les séries par séance et par jeu (les anciennes entrées sans séance sont regroupées par jeu).
fn sessions(entries: &[LogEntry], ends: &HashMap<i64, i64>) -> Vec<SessionTotals> {
    let mut groups: Vec<((i64, &str), Vec<LogEntry>)> = Vec::new();
    for e in entries {
        let key = (e.session, e.game.as_str());
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, list)) => list.push(e.clone()),
            None => groups.push((key, vec![e.clone()])),
        }
    }
    let mut out: Vec<SessionTotals> = groups
        .into_iter()
        .map(|((session, game), list)| {
            let start = list.iter().map(|e| e.at).min().unwrap_or_default();
            let last = list.iter().map(|e| e.at).max().unwrap_or_default();
            let (begin, end) = if session > 0 {
                let end = ends.get(&session).map_or(last, |&ended| ended.max(last));
                (session.min(start), end)
            } else {
                (start, last)
            };
            SessionTotals {
                start,
                duration: end - begin,
                game: game.to_string(),
                sets: list.len(),
                totals: totals(&list),
            }
        })
        .collect();
    out.sort_by_key(|s| std::cmp::Reverse(s.start));
    out
}

pub fn append_history(path: &Path, entries: &[LogEntry]) -> io::Result<()> {
    if entries.is_empty() {
        return Ok(());
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    for e in entries {
        writeln!(file, "{}", serde_json::to_string(e)?)?;
    }
    Ok(())
}

pub fn append_session_end(path: &Path, session: i64, ended: i64) -> io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(
        file,
        "{}",
        serde_json::to_string(&SessionEnd { session, ended })?
    )
}

/// Retire de l'historique les séries d'une séance (les lignes illisibles sont gardées telles quelles).
pub fn remove_session(path: &Path, session: i64) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let kept: String = std::fs::read_to_string(path)?
        .lines()
        .filter(|line| {
            let line_session = serde_json::from_str::<LogEntry>(line)
                .map(|e| e.session)
                .or_else(|_| serde_json::from_str::<SessionEnd>(line).map(|e| e.session));
            line_session.map_or(true, |s| s != session)
        })
        .map(|line| format!("{line}\n"))
        .collect();
    std::fs::write(path, kept)
}

type ByDay = BTreeMap<NaiveDate, Vec<LogEntry>>;

/// Séries de l'historique regroupées par jour local, et heures de fin des séances terminées.
fn read_by_day(path: &Path) -> io::Result<(ByDay, HashMap<i64, i64>)> {
    let mut by_day = ByDay::new();
    let mut ends = HashMap::new();
    if path.exists() {
        let file = std::fs::File::open(path)?;
        for line in BufReader::new(file).lines() {
            let line = line?;
            if let Ok(end) = serde_json::from_str::<SessionEnd>(&line) {
                ends.insert(end.session, end.ended);
                continue;
            }
            // Une ligne abîmée ne doit pas empêcher de lire le reste.
            let Ok(entry) = serde_json::from_str::<LogEntry>(&line) else {
                continue;
            };
            let Some(date) =
                DateTime::from_timestamp(entry.at, 0).map(|d| d.with_timezone(&Local).date_naive())
            else {
                continue;
            };
            by_day.entry(date).or_default().push(entry);
        }
    }
    Ok((by_day, ends))
}

fn day_totals(date: NaiveDate, entries: &[LogEntry], ends: &HashMap<i64, i64>) -> DayTotals {
    DayTotals {
        date: date.format("%Y-%m-%d").to_string(),
        sets: entries.len(),
        sessions: sessions(entries, ends),
    }
}

/// Totaux des `days` derniers jours (aujourd'hui compris), du plus récent au plus ancien.
pub fn read_history(path: &Path, days: u32, today: NaiveDate) -> io::Result<Vec<DayTotals>> {
    let (mut by_day, ends) = read_by_day(path)?;
    Ok((0..days.max(1))
        .map(|offset| {
            let date = today - Duration::days(i64::from(offset));
            day_totals(date, &by_day.remove(&date).unwrap_or_default(), &ends)
        })
        .collect())
}

/// Tous les jours où des séries ont été faites, du plus récent au plus ancien.
pub fn read_full_history(path: &Path) -> io::Result<Vec<DayTotals>> {
    let (by_day, ends) = read_by_day(path)?;
    Ok(by_day
        .iter()
        .rev()
        .map(|(date, entries)| day_totals(*date, entries, &ends))
        .collect())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn rng() -> StdRng {
        StdRng::seed_from_u64(42)
    }

    #[test]
    fn default_config_is_valid_and_roundtrips() {
        let cfg = default_config();
        cfg.validate().unwrap();
        let json = serde_json::to_string_pretty(&cfg).unwrap();
        assert!(json.contains("maxPerSession"));
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cfg);
    }

    #[test]
    fn legacy_event_exercises_are_ignored() {
        let ev: GameEvent = serde_json::from_str(
            r#"{"id":"mort","name":"Mort","exercises":[{"exercise":"squats","min":1,"max":2}]}"#,
        )
        .unwrap();
        assert_eq!(ev, event("mort", "Mort", 1, None));
    }

    #[test]
    fn draws_stay_in_range_and_are_distinct() {
        let cfg = default_config();
        let mut s = Session::new(0);
        let mut r = rng();
        for _ in 0..200 {
            let d = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
            assert_eq!(d.items.len(), 2);
            let ids: HashSet<_> = d.items.iter().map(|i| i.exercise_id.as_str()).collect();
            assert_eq!(
                ids.len(),
                2,
                "les exercices d'un même tirage doivent être différents"
            );
            for it in &d.items {
                let ex = cfg
                    .exercises
                    .iter()
                    .find(|e| e.id == it.exercise_id)
                    .unwrap();
                assert!(it.amount >= ex.min && it.amount <= ex.max);
            }
        }
    }

    #[test]
    fn every_option_can_be_drawn() {
        let cfg = default_config();
        let mut s = Session::new(0);
        let mut r = rng();
        let mut seen = HashSet::new();
        for _ in 0..100 {
            let d = s.draw(&cfg, "elden-ring", "repos", &mut r).unwrap();
            seen.insert(d.items[0].exercise_id.clone());
        }
        assert_eq!(seen.len(), cfg.exercises.len());
    }

    #[test]
    fn death_cap_applies_per_game_and_session() {
        let cfg = default_config();
        let mut s = Session::new(0);
        let mut r = rng();
        for n in 1..=15 {
            let d = s.draw(&cfg, "ds1", "mort", &mut r).unwrap();
            assert!(!d.capped);
            assert_eq!(d.trigger_count, n);
        }
        for _ in 0..3 {
            let d = s.draw(&cfg, "ds1", "mort", &mut r).unwrap();
            assert!(d.capped && d.items.is_empty());
            assert_eq!(d.trigger_count, 15, "le compteur s'arrête à la limite");
        }
        assert_eq!(
            s.view().pending.len(),
            15,
            "un tirage plafonné n'est pas mis en attente"
        );
        // La séance reste sur son jeu.
        assert!(s.draw(&cfg, "ds2", "mort", &mut r).is_err());
        assert_eq!(s.view().game_id.as_deref(), Some("ds1"));
        // Une nouvelle séance remet tout à zéro.
        let mut s2 = Session::new(1);
        assert!(!s2.draw(&cfg, "ds1", "mort", &mut r).unwrap().capped);
    }

    #[test]
    fn complete_dismiss_and_totals() {
        let cfg = default_config();
        let mut s = Session::new(0);
        let mut r = rng();
        let a = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
        let b = s.draw(&cfg, "ds3", "mort", &mut r).unwrap();
        let c = s.draw(&cfg, "ds3", "mort", &mut r).unwrap();

        let logged = s.complete(a.id, 100).unwrap();
        assert_eq!(logged.len(), 2);
        s.complete(b.id, 101).unwrap();
        s.dismiss(c.id).unwrap();
        assert!(s.complete(c.id, 102).is_err());

        let v = s.view();
        assert!(v.pending.is_empty());
        assert_eq!(v.sets, 3);
        assert_eq!(v.log[0].event, "Mort");
        let count = v
            .triggers
            .iter()
            .find(|t| t.event_id == "mort")
            .unwrap()
            .count;
        assert_eq!(count, 1, "l'annulation retire le déclenchement");
        let sum: u32 = v.totals.iter().map(|t| t.sets).sum();
        assert_eq!(sum, 3);
    }

    #[test]
    fn jokers_are_earned_then_skip_a_set() {
        let cfg = default_config();
        let mut s = Session::new(0);
        let mut r = rng();
        let death = s.draw(&cfg, "ds3", "mort", &mut r).unwrap();
        assert!(s.use_joker(death.id, 5).is_err(), "pas de joker au départ");

        let nohit = s.draw(&cfg, "ds3", "boss-nohit", &mut r).unwrap();
        assert_eq!(
            nohit.items.len(),
            2,
            "la récompense s'accompagne d'une série"
        );
        assert_eq!(nohit.jokers, 2);
        assert_eq!(s.view().jokers, 0, "rien n'est gagné avant validation");
        assert_eq!(s.complete(nohit.id, 10).unwrap().len(), 2);
        assert_eq!(s.view().jokers, 2);

        s.use_joker(death.id, 20).unwrap();
        let v = s.view();
        assert_eq!(v.jokers, 1);
        assert_eq!(v.jokers_earned, 2);
        assert_eq!(v.skipped.len(), 1);
        assert_eq!((v.skipped[0].at, v.skipped[0].event.as_str()), (20, "Mort"));
        assert_eq!(v.skipped[0].items, death.items);
        assert!(v.pending.is_empty());
        assert_eq!(v.sets, 2, "seule la série du boss est faite");
        let count = v
            .triggers
            .iter()
            .find(|t| t.event_id == "mort")
            .unwrap()
            .count;
        assert_eq!(count, 1, "une mort sautée reste comptée dans la limite");

        let first = s.draw(&cfg, "ds3", "boss-premier", &mut r).unwrap();
        s.use_joker(first.id, 30).unwrap();
        let v = s.view();
        assert_eq!(v.jokers, 0, "une série sautée ne rapporte pas ses jokers");
        assert_eq!(v.jokers_earned, 2);
    }

    #[test]
    fn validation_catches_mistakes() {
        let mut cfg = default_config();
        cfg.exercises[0].min = 20;
        assert!(cfg.validate().unwrap_err().contains("Intervalle"));

        let mut cfg = default_config();
        cfg.games[0].events[0].draws = cfg.exercises.len() + 1;
        assert!(cfg.validate().is_err());

        let mut cfg = default_config();
        cfg.games[1].id = "elden-ring".into();
        assert!(cfg.validate().unwrap_err().contains("deux fois"));

        let mut cfg = default_config();
        cfg.exercises.clear();
        assert!(cfg.validate().unwrap_err().contains("Aucun exercice"));
    }

    #[test]
    fn history_roundtrip_by_day() {
        let dir = std::env::temp_dir().join(format!("squat-souls-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("history.jsonl");
        let _ = std::fs::remove_file(&path);

        let now = Local::now();
        let today = now.date_naive();
        let entry = |at: i64, ex: &str, amount| LogEntry {
            at,
            session: 1,
            game: "Dark Souls".into(),
            event: "Mort".into(),
            exercise: ex.into(),
            unit: "rép.".into(),
            amount,
        };
        let t = now.timestamp();
        append_history(
            &path,
            &[
                entry(t, "Squats", 12),
                entry(t, "Squats", 15),
                entry(t, "Pompes", 10),
            ],
        )
        .unwrap();
        append_history(&path, &[entry(t - 86_400 * 2, "Pompes", 8)]).unwrap();
        append_history(&path, &[entry(t - 86_400 * 30, "Pompes", 99)]).unwrap();
        std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"ligne abimee\n")
            .unwrap();

        let days = read_history(&path, 7, today).unwrap();
        assert_eq!(days.len(), 7);
        assert_eq!(days[0].sets, 3);
        assert_eq!(days[0].sessions.len(), 1);
        assert_eq!(days[0].sessions[0].game, "Dark Souls");
        assert_eq!(
            days[0].sessions[0].totals[0],
            Total {
                exercise: "Squats".into(),
                unit: "rép.".into(),
                amount: 27,
                sets: 2
            }
        );
        assert_eq!(days[2].sets, 1);
        assert_eq!(
            days.iter().map(|d| d.sets).sum::<usize>(),
            4,
            "l'entrée de 30 jours est hors fenêtre"
        );

        let full = read_full_history(&path).unwrap();
        assert_eq!(
            full.iter().map(|d| d.sets).collect::<Vec<_>>(),
            vec![3, 1, 1],
            "tous les jours actifs, du plus récent"
        );

        append_history(
            &path,
            &[LogEntry {
                session: 2,
                ..entry(t, "Squats", 5)
            }],
        )
        .unwrap();
        append_session_end(&path, 1, t + 600).unwrap();
        append_session_end(&path, 2, t + 60).unwrap();
        assert_eq!(read_history(&path, 1, today).unwrap()[0].sets, 4);
        remove_session(&path, 2).unwrap();
        assert!(!std::fs::read_to_string(&path)
            .unwrap()
            .contains(r#""session":2,"ended""#));
        assert_eq!(
            read_history(&path, 1, today).unwrap()[0].sets,
            3,
            "séance 2 effacée"
        );
        assert!(std::fs::read_to_string(&path)
            .unwrap()
            .contains("ligne abimee"));
        remove_session(&path, 1).unwrap();
        assert!(read_full_history(&path).unwrap().is_empty());

        let missing = read_history(&dir.join("absent.jsonl"), 3, today).unwrap();
        assert!(missing.iter().all(|d| d.sets == 0));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn sessions_grouped_by_session_and_game() {
        let entry = |at: i64, session: i64, game: &str, amount| LogEntry {
            at,
            session,
            game: game.into(),
            event: "Mort".into(),
            exercise: "Squats".into(),
            unit: "rép.".into(),
            amount,
        };
        let old: LogEntry = serde_json::from_str(
            r#"{"at":50,"game":"Elden Ring","event":"Mort","exercise":"Squats","unit":"rép.","amount":5}"#,
        )
        .unwrap();
        assert_eq!(old.session, 0);

        let groups = sessions(
            &[
                entry(100, 90, "Dark Souls", 10),
                entry(300, 250, "Sekiro", 8),
                entry(120, 90, "Dark Souls", 12),
                old,
            ],
            &HashMap::from([(90, 400)]),
        );
        assert_eq!(groups.len(), 3);
        assert_eq!((groups[0].game.as_str(), groups[0].sets), ("Sekiro", 1));
        assert_eq!(
            (groups[1].game.as_str(), groups[1].start, groups[1].sets),
            ("Dark Souls", 100, 2)
        );
        assert_eq!(groups[1].totals[0].amount, 22);
        assert_eq!(groups[0].duration, 50, "depuis le début de la séance");
        assert_eq!(groups[1].duration, 310, "jusqu'à la fin de la séance");
        assert_eq!(groups[2].game, "Elden Ring");
        assert_eq!(
            groups[2].duration, 0,
            "sans séance, depuis la première série"
        );
    }
}
