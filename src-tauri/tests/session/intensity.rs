use super::*;

#[test]
fn intensity_scales_the_drawn_numbers() {
    assert_eq!(
        [0, 1, 2, 3, 10].map(|n| Intensity::Easy.scale(n)),
        [0, 1, 1, 1, 5],
        "rounded down, never to 0"
    );
    assert_eq!(Intensity::Normal.scale(7), 7);
    assert_eq!(
        [1, 2, 3, 10].map(|n| Intensity::Hard.scale(n)),
        [2, 3, 5, 15],
        "rounded up"
    );
    assert_eq!([2, 3, 9].map(|n| Intensity::Easy.halve(n)), [1, 1, 4]);
    for intensity in [Intensity::Normal, Intensity::Hard] {
        assert_eq!([2, 3, 9].map(|n| intensity.halve(n)), [1, 2, 5]);
    }
    let cfg = catalog_config(Language::Fr);
    let mut r = rng();
    for intensity in [Intensity::Easy, Intensity::Hard] {
        let mut s = Session::default();
        s.set_intensity(intensity);
        for _ in 0..50 {
            for it in s
                .draw(
                    &cfg,
                    &game_id("ds3"),
                    &event_id("ds3", "Boss defeated"),
                    0,
                    &mut r,
                )
                .unwrap()
                .items
            {
                let ex = cfg
                    .exercises
                    .iter()
                    .find(|e| e.id == it.exercise_id)
                    .unwrap();
                assert!(
                    it.amount >= intensity.scale(ex.min) && it.amount <= intensity.scale(ex.max)
                );
            }
        }
    }
}

#[test]
fn a_new_intensity_rescales_the_amounts_to_do_from_the_drawn_ones() {
    let cfg = catalog_config(Language::Fr);
    let mut s = Session::default();
    let boss = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated"),
            0,
            &mut rng(),
        )
        .unwrap();
    s.complete_exercise(&boss.id, &boss.items[0].exercise_id, 0)
        .unwrap();
    for intensity in [Intensity::Hard, Intensity::Easy, Intensity::Normal] {
        s.set_intensity(intensity);
        assert_eq!(s.view().intensity, intensity);
        let draw = pending(&s, &boss.id).unwrap();
        assert_eq!(draw.items[0].amount, boss.items[0].amount, "already done");
        assert_eq!(draw.items[1].amount, intensity.scale(boss.items[1].amount));
    }
}

#[test]
fn a_new_intensity_rescales_a_halved_amount_from_the_drawn_one_then_halves_it() {
    let (mut s, boss) = session_with_jokers();
    let item = &boss.items[0];
    s.halve(&boss.id, &item.exercise_id).unwrap();
    for intensity in [Intensity::Hard, Intensity::Easy] {
        s.set_intensity(intensity);
        let halved = &pending(&s, &boss.id).unwrap().items[0];
        assert_eq!(halved.amount, intensity.halve(intensity.scale(item.amount)));
        assert_eq!(halved.halve_cost, None, "still halved");
    }
}

/// Every trigger applies the intensity again: a number halved at Hard stays the half of the one shown.
#[test]
fn a_number_halved_at_hard_keeps_its_half_when_the_intensity_applies_again() {
    let mut cfg = catalog_config(Language::Fr);
    for ex in &mut cfg.exercises {
        (ex.min, ex.max) = (5, 5);
    }
    let (mut s, boss) = session_with_jokers_in(&cfg);
    s.set_intensity(Intensity::Hard);
    let id = &boss.items[0].exercise_id;
    assert_eq!(pending(&s, &boss.id).unwrap().items[0].amount, 8);
    s.halve(&boss.id, id).unwrap();
    s.set_intensity(Intensity::Hard);
    assert_eq!(pending(&s, &boss.id).unwrap().items[0].amount, 4);
}

#[test]
fn halving_at_easy_rounds_down() {
    let (mut s, boss) = session_with_jokers();
    s.set_intensity(Intensity::Easy);
    let item = pending(&s, &boss.id).unwrap().items[0].clone();
    s.halve(&boss.id, &item.exercise_id).unwrap();
    assert_eq!(
        pending(&s, &boss.id).unwrap().items[0].amount,
        item.amount / 2
    );
}
