use rand::rngs::StdRng;
use rand::SeedableRng;
use squat_souls_lib::fitness::*;
use std::collections::HashSet;

fn rng() -> StdRng {
    StdRng::seed_from_u64(42)
}

/// Note faits, un à un, les exercices restants d'une série.
fn complete(s: &mut Session, id: u64, now: i64) -> Result<Vec<LogEntry>, String> {
    let draw = s
        .view()
        .pending
        .into_iter()
        .find(|d| d.id == id)
        .ok_or("Cette série n'est plus en attente.")?;
    draw.items
        .iter()
        .filter(|it| !it.done)
        .map(|it| s.complete_exercise(id, &it.exercise_id, now))
        .collect()
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
fn each_exercise_comes_out_once_per_round() {
    let cfg = default_config();
    let n = cfg.exercises.len();
    let mut s = Session::new(0);
    let mut r = rng();
    for _ in 0..20 {
        let round: HashSet<_> = (0..n)
            .map(|_| {
                s.draw(&cfg, "elden-ring", "repos", &mut r).unwrap().items[0]
                    .exercise_id
                    .clone()
            })
            .collect();
        assert_eq!(
            round.len(),
            n,
            "aucun exercice ne revient avant la fin du tour"
        );
    }
}

#[test]
fn a_draw_across_two_rounds_stays_distinct() {
    let cfg = default_config();
    let n = cfg.exercises.len();
    let mut s = Session::new(0);
    let mut r = rng();
    // Un exercice tiré seul décale le tour : avec un nombre pair d'exercices, les tirages de 2 chevauchent alors deux tours.
    assert_eq!(n % 2, 0);
    s.draw(&cfg, "ds3", "repos", &mut r).unwrap();
    let mut out: Vec<String> = Vec::new();
    for _ in 0..50 {
        let d = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
        assert_ne!(d.items[0].exercise_id, d.items[1].exercise_id);
        out.extend(d.items.into_iter().map(|it| it.exercise_id));
    }
    // Après le premier exercice tiré seul, chaque tour complet compte chaque exercice une fois.
    for round in out[n - 1..].chunks_exact(n) {
        assert_eq!(round.iter().collect::<HashSet<_>>().len(), n);
    }
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

    let logged = complete(&mut s, a.id, 100).unwrap();
    assert_eq!(logged.len(), 2);
    complete(&mut s, b.id, 101).unwrap();
    s.dismiss(c.id).unwrap();
    assert!(complete(&mut s, c.id, 102).is_err());

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

/// Séance où un « boss no hit » validé a rapporté 2 jokers, avec une série de boss à deux exercices en attente.
fn session_with_jokers() -> (Session, Draw) {
    let cfg = default_config();
    let mut s = Session::new(0);
    let mut r = rng();
    let boss = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
    assert!(s.skip_set(boss.id, 5).is_err(), "pas de joker au départ");

    let nohit = s.draw(&cfg, "ds3", "boss-nohit", &mut r).unwrap();
    assert_eq!(
        nohit.items.len(),
        2,
        "la récompense s'accompagne d'une série"
    );
    assert_eq!(nohit.jokers, 2);
    assert_eq!(s.view().jokers, 0, "rien n'est gagné avant validation");
    assert_eq!(complete(&mut s, nohit.id, 10).unwrap().len(), 2);
    assert_eq!(s.view().jokers, 2);
    (s, boss)
}

#[test]
fn skipping_a_set_costs_its_exercises() {
    let (mut s, boss) = session_with_jokers();
    assert_eq!(skip_set_cost(&boss), 2 * SKIP_COST);
    assert!(
        s.skip_set(boss.id, 20).is_err(),
        "deux exercices coûtent 4 jokers"
    );

    let cfg = default_config();
    let death = s.draw(&cfg, "ds3", "mort", &mut rng()).unwrap();
    s.skip_set(death.id, 20).unwrap();
    let v = s.view();
    assert_eq!(v.jokers, 0);
    assert_eq!(v.jokers_earned, 2);
    assert_eq!(v.skipped.len(), 1);
    let k = &v.skipped[0];
    assert_eq!(
        (k.at, k.event.as_str(), k.jokers, k.halved),
        (20, "Mort", SKIP_COST, false)
    );
    assert_eq!(k.items, death.items);
    assert_eq!(v.pending.len(), 1, "seule la série du boss reste");
    assert_eq!(v.sets, 2, "seule la série du no hit est faite");
    let count = v
        .triggers
        .iter()
        .find(|t| t.event_id == "mort")
        .unwrap()
        .count;
    assert_eq!(count, 1, "une mort sautée reste comptée dans la limite");
}

#[test]
fn skipping_an_exercise_keeps_the_rest_of_the_set() {
    let (mut s, boss) = session_with_jokers();
    let (first, second) = (&boss.items[0], &boss.items[1]);
    s.skip_exercise(boss.id, &first.exercise_id, 20).unwrap();
    let v = s.view();
    assert_eq!(v.jokers, 0);
    assert_eq!(v.pending[0].items, vec![second.clone()]);
    assert_eq!(v.skipped[0].items, vec![first.clone()]);
    assert_eq!(v.skipped[0].jokers, SKIP_COST);
    assert!(
        s.skip_exercise(boss.id, &second.exercise_id, 30).is_err(),
        "plus de joker"
    );
    assert!(
        s.skip_exercise(boss.id, &first.exercise_id, 30).is_err(),
        "déjà sauté"
    );

    let entries = complete(&mut s, boss.id, 40).unwrap();
    assert_eq!(entries.len(), 1, "seul l'exercice restant est fait");
    assert_eq!(entries[0].amount, second.amount);
}

#[test]
fn halving_then_skipping_costs_a_full_skip() {
    let (mut s, boss) = session_with_jokers();
    let item = &boss.items[0];
    assert!(item.amount >= 2);
    assert_eq!(item.halve_cost, Some(HALVE_COST));
    s.halve(boss.id, &item.exercise_id, 20).unwrap();
    let v = s.view();
    let halved = &v.pending[0].items[0];
    assert_eq!(
        halved.amount,
        item.amount.div_ceil(2),
        "arrondi au supérieur"
    );
    assert!(halved.halved);
    assert_eq!(halved.halve_cost, None, "une seule moitié par exercice");
    assert_eq!(halved.skip_cost, SKIP_COST - HALVE_COST);
    assert_eq!(v.jokers, 2 - HALVE_COST);
    let k = &v.skipped[0];
    assert!(k.halved);
    assert_eq!(k.jokers, HALVE_COST);
    assert_eq!(k.items[0].amount, item.amount / 2, "la moitié retirée");
    assert!(s.halve(boss.id, &item.exercise_id, 30).is_err());

    s.skip_exercise(boss.id, &item.exercise_id, 30).unwrap();
    assert_eq!(
        s.view().jokers,
        0,
        "le reste d'un exercice réduit coûte la différence"
    );

    let other = &boss.items[1];
    assert!(
        s.halve(boss.id, &other.exercise_id, 40).is_err(),
        "plus de joker"
    );
}

#[test]
fn spending_jokers_on_a_rewarding_set_forfeits_its_reward() {
    let cfg = default_config();
    for spend in [Session::halve, Session::skip_exercise] {
        let (mut s, _) = session_with_jokers();
        let nohit = s.draw(&cfg, "ds3", "boss-nohit", &mut rng()).unwrap();
        let first = nohit
            .items
            .iter()
            .find(|it| it.halve_cost.is_some())
            .unwrap();
        spend(&mut s, nohit.id, &first.exercise_id, 20).unwrap();
        assert_eq!(s.view().pending[1].jokers, 0, "la récompense est perdue");
        let left = s.view().jokers;
        complete(&mut s, nohit.id, 30).unwrap();
        let v = s.view();
        assert_eq!(v.jokers, left, "rien n'est gagné à la validation");
        assert_eq!(v.jokers_earned, 2);
    }
}

#[test]
fn skipping_every_exercise_removes_the_set() {
    let (mut s, _) = session_with_jokers();
    let cfg = default_config();
    let death = s.draw(&cfg, "ds3", "mort", &mut rng()).unwrap();
    s.skip_exercise(death.id, &death.items[0].exercise_id, 20)
        .unwrap();
    assert!(s.view().pending.iter().all(|d| d.id != death.id));
}

#[test]
fn a_single_unit_cannot_be_halved() {
    let mut cfg = default_config();
    for ex in &mut cfg.exercises {
        ex.min = 1;
        ex.max = 1;
    }
    let d = Session::new(0)
        .draw(&cfg, "ds3", "mort", &mut rng())
        .unwrap();
    assert_eq!(d.items[0].halve_cost, None);
}

#[test]
fn exercises_are_done_one_by_one() {
    let (mut s, boss) = session_with_jokers();
    let cfg = default_config();
    let nohit = s.draw(&cfg, "ds3", "boss-nohit", &mut rng()).unwrap();
    let (first, second) = (&nohit.items[0], &nohit.items[1]);
    let entry = s
        .complete_exercise(nohit.id, &first.exercise_id, 20)
        .unwrap();
    assert_eq!((entry.at, entry.amount), (20, first.amount));
    let v = s.view();
    let left = v.pending.iter().find(|d| d.id == nohit.id).unwrap();
    assert!(left.items[0].done && !left.items[1].done);
    assert_eq!(
        skip_set_cost(left),
        SKIP_COST,
        "seul l'exercice restant compte"
    );
    assert_eq!(v.jokers, 2, "la récompense attend la fin de la série");
    for again in [
        s.complete_exercise(nohit.id, &first.exercise_id, 21)
            .map(|_| ()),
        s.halve(nohit.id, &first.exercise_id, 21),
        s.skip_exercise(nohit.id, &first.exercise_id, 21),
    ] {
        assert!(again.is_err(), "déjà fait");
    }

    s.complete_exercise(nohit.id, &second.exercise_id, 30)
        .unwrap();
    let v = s.view();
    assert_eq!(v.jokers, 4, "série finie : la récompense est gagnée");
    assert_eq!(v.pending.len(), 1, "seule la série du boss reste");
    assert_eq!(v.log[0].at, 30);

    s.complete_exercise(boss.id, &boss.items[0].exercise_id, 40)
        .unwrap();
    s.skip_set(boss.id, 50).unwrap();
    let v = s.view();
    assert_eq!(
        v.jokers,
        4 - SKIP_COST,
        "sauter le reste ne coûte que l'exercice restant"
    );
    assert_eq!(v.skipped[0].items, vec![boss.items[1].clone()]);
    assert_eq!(v.sets, 5);
}

#[test]
fn dismissing_a_started_set_drops_its_done_exercises() {
    let cfg = default_config();
    let mut s = Session::new(0);
    let mort = s.draw(&cfg, "ds3", "mort", &mut rng()).unwrap();
    let kept = complete(&mut s, mort.id, 5).unwrap();
    let boss = s.draw(&cfg, "ds3", "boss", &mut rng()).unwrap();
    let entry = s
        .complete_exercise(boss.id, &boss.items[0].exercise_id, 10)
        .unwrap();
    assert_eq!(
        s.dismiss(boss.id).unwrap(),
        vec![entry],
        "à retirer de l'historique"
    );
    let v = s.view();
    assert!(v.pending.is_empty());
    assert_eq!(
        v.log,
        kept.into_iter().rev().collect::<Vec<_>>(),
        "rien n'est compté de la série annulée"
    );
    assert_eq!(v.triggers.len(), 1, "le déclenchement est décompté");
}
