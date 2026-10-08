use prototype2_rust::locomotion::{SteerSpeeds, advance_speed, turn_toward};

#[test]
fn analog_curve_distinguishes_actor_modes_and_retains_overrange_input() {
    let speeds = SteerSpeeds {
        walk: 1.5,
        run: 4.5,
    };
    for (input, expected) in [
        (0., 0.),
        (0.25, 0.75),
        (0.5, 1.5),
        (0.75, 3.),
        (1., 4.5),
        (1.1, 4.95),
    ] {
        assert!((speeds.target(input, true).unwrap() - expected).abs() < 1e-6);
    }
    assert!((speeds.target(0.25, false).unwrap() - 1.125).abs() < 1e-6);
    let same = SteerSpeeds { walk: 4., run: 4. };
    assert!(same.target(0.995, false).unwrap().is_finite());
    assert!(speeds.target(f32::NAN, true).is_err());
    assert!(speeds.target(-0.1, true).is_err());
}
#[test]
fn acceleration_release_and_direction_limits_are_explicit_in_seconds() {
    let mut speed = 0.;
    for _ in 0..10 {
        speed = advance_speed(speed, 4.5, 15., 0.01).unwrap();
    }
    assert!((speed - 1.5).abs() < 1e-6);
    for _ in 0..20 {
        speed = advance_speed(speed, 4.5, 15., 0.01).unwrap();
    }
    assert!((speed - 4.5).abs() < 2e-6);
    assert_eq!(advance_speed(0., 4.5, 15., 1.).unwrap(), 4.5);
    assert_eq!(advance_speed(4.5, 0., 15., 1.).unwrap(), 0.);
    assert_eq!(advance_speed(1., 4.5, 0., 0.).unwrap(), 4.5);
    assert!(advance_speed(0., 1., 15., -1.).is_err());
    let initial = 179f32.to_radians();
    let turned = turn_toward(initial, (-179f32).to_radians(), 0., 4.5, 1., 2., 0.01).unwrap();
    assert!((turned - initial - 0.01).abs() < 1e-6);
    let full = turn_toward(0., 2., 4.5, 4.5, 1., 2., 0.1).unwrap();
    assert!((full - 0.2).abs() < 1e-6);
}
