use rand::rngs::StdRng;
use rand::SeedableRng;
use squat_souls_lib::fitness::*;
use std::collections::HashSet;

fn rng() -> StdRng {
    StdRng::seed_from_u64(42)
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
fn removed_game_is_detected() {
    let mut cfg = default_config();
    let mut s = Session::new(0);
    assert!(!s.game_removed(&cfg), "séance pas encore commencée");
    s.draw(&cfg, "ds1", "mort", &mut rng()).unwrap();
    assert!(!s.game_removed(&cfg));
    cfg.games.retain(|g| g.id != "ds2");
    assert!(!s.game_removed(&cfg), "un autre jeu supprimé ne compte pas");
    cfg.games.retain(|g| g.id != "ds1");
    assert!(s.game_removed(&cfg));
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
