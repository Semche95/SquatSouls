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
}

/// Un exercice possible pour un événement, avec son intervalle (bornes incluses).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExerciseRange {
    pub exercise: String,
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
    pub exercises: Vec<ExerciseRange>,
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
    pub fn exercise(&self, id: &str) -> Option<&Exercise> {
        self.exercises.iter().find(|e| e.id == id)
    }

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
                    return Err(format!("{} : l'événement « {} » est défini deux fois.", game.name, ev.id));
                }
                if ev.exercises.is_empty() {
                    return Err(format!("{at} : aucun exercice possible."));
                }
                if ev.draws == 0 {
                    return Err(format!("{at} : « draws » doit valoir au moins 1."));
                }
                if ev.draws > ev.exercises.len() {
                    return Err(format!(
                        "{at} : {} exercices à tirer mais seulement {} possibles.",
                        ev.draws,
                        ev.exercises.len()
                    ));
                }
                if ev.max_per_session == Some(0) {
                    return Err(format!("{at} : « maxPerSession » doit valoir au moins 1 (ou être retiré)."));
                }
                let mut seen = HashSet::new();
                for r in &ev.exercises {
                    if !exercise_ids.contains(r.exercise.as_str()) {
                        return Err(format!("{at} : exercice inconnu « {} ».", r.exercise));
                    }
                    if !seen.insert(r.exercise.as_str()) {
                        return Err(format!("{at} : l'exercice « {} » apparaît deux fois.", r.exercise));
                    }
                    if r.max == 0 || r.min > r.max {
                        return Err(format!(
                            "{at} : intervalle invalide pour « {} » ({} à {}).",
                            r.exercise, r.min, r.max
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

fn range(exercise: &str, min: u32, max: u32) -> ExerciseRange {
    ExerciseRange { exercise: exercise.into(), min, max }
}

fn event(id: &str, name: &str, draws: usize, max: Option<u32>, exercises: Vec<ExerciseRange>) -> GameEvent {
    GameEvent { id: id.into(), name: name.into(), draws, max_per_session: max, exercises }
}

/// Événements communs aux Souls, avec le vocabulaire propre à chaque jeu.
fn souls_events(rest: &str, discovered: &str, lost: &str) -> Vec<GameEvent> {
    let boss = || vec![range("pompes", 10, 14), range("tractions", 3, 5), range("squats", 15, 20)];
    vec![
        event("mort", "Mort", 1, Some(15), vec![
            range("squats", 12, 18),
            range("fentes", 5, 8),
            range("gainage", 20, 30),
        ]),
        event("repos", rest, 1, None, vec![range("pompes", 8, 12), range("superman", 10, 15)]),
        event("decouverte", discovered, 1, None, vec![range("fentes", 5, 8), range("squats", 12, 18)]),
        event("niveau", "Montée de niveau", 1, None, vec![range("tractions", 2, 4), range("pompes", 8, 12)]),
        event("perte", lost, 1, None, vec![range("gainage", 30, 45), range("superman", 10, 15)]),
        event("boss", "Boss vaincu", 3, None, boss()),
        event("boss-premier", "Boss vaincu du premier essai", 1, None, boss()),
        event("boss-nohit", "Boss vaincu sans prendre un coup", 1, None, boss()),
    ]
}

pub fn default_config() -> Config {
    let ex = |id: &str, name: &str, unit: &str| Exercise { id: id.into(), name: name.into(), unit: unit.into() };
    let game = |id: &str, name: &str, events| Game { id: id.into(), name: name.into(), events };
    let ds = || souls_events("Repos à un feu connu", "Nouveau feu allumé", "Âmes perdues");
    Config {
        exercises: vec![
            ex("pompes", "Pompes", "rép."),
            ex("tractions", "Tractions", "rép."),
            ex("squats", "Squats", "rép."),
            ex("fentes", "Fentes", "rép. par jambe"),
            ex("gainage", "Gainage (planche)", "s"),
            ex("superman", "Superman", "rép."),
        ],
        games: vec![
            game(
                "elden-ring",
                "Elden Ring",
                souls_events("Repos à un site de grâce connu", "Site de grâce découvert", "Runes perdues"),
            ),
            game("ds1", "Dark Souls", ds()),
            game("ds2", "Dark Souls II", ds()),
            game("ds3", "Dark Souls III", ds()),
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
    /// Vrai quand la limite de l'événement pour la séance est dépassée.
    pub capped: bool,
    pub trigger_count: u32,
    pub max_per_session: Option<u32>,
}

/// Une série faite, telle qu'enregistrée dans l'historique.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub at: i64,
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

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TriggerCount {
    pub game_id: String,
    pub event_id: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionView {
    pub started_at: i64,
    pub pending: Vec<Draw>,
    pub sets: usize,
    pub totals: Vec<Total>,
    /// Séries faites, la plus récente en premier.
    pub log: Vec<LogEntry>,
    pub triggers: Vec<TriggerCount>,
}

#[derive(Debug, Default)]
pub struct Session {
    pub started_at: i64,
    triggers: HashMap<(String, String), u32>,
    pending: Vec<Draw>,
    done: Vec<LogEntry>,
    next_id: u64,
}

impl Session {
    pub fn new(now: i64) -> Self {
        Session { started_at: now, ..Default::default() }
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
        let ev = game
            .events
            .iter()
            .find(|e| e.id == event_id)
            .ok_or_else(|| format!("Événement inconnu : {event_id}"))?;

        let count = self.triggers.entry((game.id.clone(), ev.id.clone())).or_insert(0);
        *count += 1;
        let trigger_count = *count;
        let capped = ev.max_per_session.is_some_and(|max| trigger_count > max);

        let items = if capped {
            Vec::new()
        } else {
            let picks = rand::seq::index::sample(rng, ev.exercises.len(), ev.draws);
            picks
                .iter()
                .map(|i| {
                    let r = &ev.exercises[i];
                    let ex = config
                        .exercise(&r.exercise)
                        .ok_or_else(|| format!("Exercice inconnu : {}", r.exercise))?;
                    Ok(DrawItem {
                        exercise_id: ex.id.clone(),
                        name: ex.name.clone(),
                        unit: ex.unit.clone(),
                        amount: rng.random_range(r.min..=r.max),
                    })
                })
                .collect::<Result<Vec<_>, String>>()?
        };

        self.next_id += 1;
        let draw = Draw {
            id: self.next_id,
            game_id: game.id.clone(),
            game_name: game.name.clone(),
            event_id: ev.id.clone(),
            event_name: ev.name.clone(),
            items,
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
        let entries: Vec<LogEntry> = draw
            .items
            .into_iter()
            .map(|it| LogEntry {
                at: now,
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
            .map(|((g, e), &count)| TriggerCount { game_id: g.clone(), event_id: e.clone(), count })
            .collect();
        triggers.sort_by(|a, b| (&a.game_id, &a.event_id).cmp(&(&b.game_id, &b.event_id)));
        SessionView {
            started_at: self.started_at,
            pending: self.pending.clone(),
            sets: self.done.len(),
            totals: totals(&self.done),
            log: self.done.iter().rev().cloned().collect(),
            triggers,
        }
    }
}

/// Cumule les séries par exercice, dans l'ordre de première apparition.
pub fn totals(entries: &[LogEntry]) -> Vec<Total> {
    let mut out: Vec<Total> = Vec::new();
    for e in entries {
        match out.iter_mut().find(|t| t.exercise == e.exercise && t.unit == e.unit) {
            Some(t) => {
                t.amount += e.amount;
                t.sets += 1;
            }
            None => out.push(Total { exercise: e.exercise.clone(), unit: e.unit.clone(), amount: e.amount, sets: 1 }),
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
    pub totals: Vec<Total>,
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

/// Totaux des `days` derniers jours (aujourd'hui compris), du plus récent au plus ancien.
pub fn read_history(path: &Path, days: u32, today: NaiveDate) -> io::Result<Vec<DayTotals>> {
    let days = days.max(1);
    let first_day = today - Duration::days(i64::from(days) - 1);
    let mut by_day: BTreeMap<NaiveDate, Vec<LogEntry>> = BTreeMap::new();

    if path.exists() {
        let file = std::fs::File::open(path)?;
        for line in BufReader::new(file).lines() {
            let line = line?;
            // Une ligne abîmée ne doit pas empêcher de lire le reste.
            let Ok(entry) = serde_json::from_str::<LogEntry>(&line) else { continue };
            let Some(date) = DateTime::from_timestamp(entry.at, 0).map(|d| d.with_timezone(&Local).date_naive())
            else {
                continue;
            };
            if date >= first_day && date <= today {
                by_day.entry(date).or_default().push(entry);
            }
        }
    }

    Ok((0..days)
        .map(|offset| {
            let date = today - Duration::days(i64::from(offset));
            let entries = by_day.remove(&date).unwrap_or_default();
            DayTotals { date: date.format("%Y-%m-%d").to_string(), sets: entries.len(), totals: totals(&entries) }
        })
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
    fn draws_stay_in_range_and_are_distinct() {
        let cfg = default_config();
        let mut s = Session::new(0);
        let mut r = rng();
        for _ in 0..200 {
            let d = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
            assert_eq!(d.items.len(), 3);
            let ids: HashSet<_> = d.items.iter().map(|i| i.exercise_id.as_str()).collect();
            assert_eq!(ids.len(), 3, "les exercices d'un même tirage doivent être différents");
            let ev = &cfg.games[3].events.iter().find(|e| e.id == "boss").unwrap();
            for it in &d.items {
                let r = ev.exercises.iter().find(|r| r.exercise == it.exercise_id).unwrap();
                assert!(it.amount >= r.min && it.amount <= r.max);
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
        assert_eq!(seen.len(), 2);
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
        let d = s.draw(&cfg, "ds1", "mort", &mut r).unwrap();
        assert!(d.capped && d.items.is_empty());
        assert_eq!(s.view().pending.len(), 15, "un tirage plafonné n'est pas mis en attente");
        // Un autre jeu a son propre compteur.
        assert!(!s.draw(&cfg, "ds2", "mort", &mut r).unwrap().capped);
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
        assert_eq!(logged.len(), 3);
        s.complete(b.id, 101).unwrap();
        s.dismiss(c.id).unwrap();
        assert!(s.complete(c.id, 102).is_err());

        let v = s.view();
        assert!(v.pending.is_empty());
        assert_eq!(v.sets, 4);
        assert_eq!(v.log[0].event, "Mort");
        let count = v.triggers.iter().find(|t| t.event_id == "mort").unwrap().count;
        assert_eq!(count, 1, "l'annulation retire le déclenchement");
        let sum: u32 = v.totals.iter().map(|t| t.sets).sum();
        assert_eq!(sum, 4);
    }

    #[test]
    fn validation_catches_mistakes() {
        let mut cfg = default_config();
        cfg.games[0].events[0].exercises[0].exercise = "burpees".into();
        assert!(cfg.validate().unwrap_err().contains("burpees"));

        let mut cfg = default_config();
        cfg.games[0].events[0].exercises[0].min = 20;
        assert!(cfg.validate().unwrap_err().contains("intervalle"));

        let mut cfg = default_config();
        cfg.games[0].events[0].draws = 9;
        assert!(cfg.validate().is_err());

        let mut cfg = default_config();
        cfg.games[1].id = "elden-ring".into();
        assert!(cfg.validate().unwrap_err().contains("deux fois"));
    }

    #[test]
    fn history_roundtrip_by_day() {
        let dir = std::env::temp_dir().join(format!("souls-fit-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("history.jsonl");
        let _ = std::fs::remove_file(&path);

        let now = Local::now();
        let today = now.date_naive();
        let entry = |at: i64, ex: &str, amount| LogEntry {
            at,
            game: "Dark Souls".into(),
            event: "Mort".into(),
            exercise: ex.into(),
            unit: "rép.".into(),
            amount,
        };
        let t = now.timestamp();
        append_history(&path, &[entry(t, "Squats", 12), entry(t, "Squats", 15), entry(t, "Pompes", 10)]).unwrap();
        append_history(&path, &[entry(t - 86_400 * 2, "Pompes", 8)]).unwrap();
        append_history(&path, &[entry(t - 86_400 * 30, "Pompes", 99)]).unwrap();
        std::fs::OpenOptions::new().append(true).open(&path).unwrap().write_all(b"ligne abimee\n").unwrap();

        let days = read_history(&path, 7, today).unwrap();
        assert_eq!(days.len(), 7);
        assert_eq!(days[0].sets, 3);
        assert_eq!(days[0].totals[0], Total { exercise: "Squats".into(), unit: "rép.".into(), amount: 27, sets: 2 });
        assert_eq!(days[2].sets, 1);
        assert_eq!(days.iter().map(|d| d.sets).sum::<usize>(), 4, "l'entrée de 30 jours est hors fenêtre");

        let missing = read_history(&dir.join("absent.jsonl"), 3, today).unwrap();
        assert!(missing.iter().all(|d| d.sets == 0));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
