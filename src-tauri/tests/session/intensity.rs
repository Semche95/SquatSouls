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
    let cfg = seed_config(Language::Fr);
    let mut r = rng();
    for intensity in [Intensity::Easy, Intensity::Hard] {
        let mut s = Session::default();
        s.set_intensity(intensity);
        for _ in 0..50 {
            for it in s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap().items {
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
    let cfg = seed_config(Language::Fr);
    let mut s = Session::default();
    let boss = s.draw(&cfg, "ds3", "boss", 0, &mut rng()).unwrap();
    s.complete_exercise(boss.id, &boss.items[0].exercise_id, 0)
        .unwrap();
    for intensity in [Intensity::Hard, Intensity::Easy, Intensity::Normal] {
        s.set_intensity(intensity);
        assert_eq!(s.view().intensity, intensity);
        let draw = pending(&s, boss.id).unwrap();
        assert_eq!(draw.items[0].amount, boss.items[0].amount, "already done");
        assert_eq!(draw.items[1].amount, intensity.scale(boss.items[1].amount));
    }
}

#[test]
fn a_new_intensity_rescales_a_halved_amount_from_the_halved_drawn_one() {
    let (mut s, boss) = session_with_jokers();
    let item = &boss.items[0];
    s.halve(boss.id, &item.exercise_id).unwrap();
    for intensity in [Intensity::Hard, Intensity::Easy] {
        s.set_intensity(intensity);
        let halved = &pending(&s, boss.id).unwrap().items[0];
        assert_eq!(halved.amount, intensity.scale(intensity.halve(item.amount)));
        assert_eq!(halved.halve_cost, None, "still halved");
    }
}

#[test]
fn halving_at_easy_rounds_down() {
    let (mut s, boss) = session_with_jokers();
    s.set_intensity(Intensity::Easy);
    let item = pending(&s, boss.id).unwrap().items[0].clone();
    s.halve(boss.id, &item.exercise_id).unwrap();
    assert_eq!(
        pending(&s, boss.id).unwrap().items[0].amount,
        item.amount / 2
    );
}
