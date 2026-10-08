use prototype2_rust::{animation::ClipInfo, animation_clock::ClipTiming};

fn clip() -> ClipInfo {
    ClipInfo {
        name: "synthetic".into(),
        offset: 0,
        kind: *b"PTRN",
        frame_count: 101.,
        frames_per_second: 20.,
        cyclic: true,
        default_sync_frame: 0.,
    }
}
fn timing(start: f32, end: f32, speed: f32) -> ClipTiming {
    ClipTiming::configure(&clip(), start, end, -1., speed, false).unwrap()
}
#[test]
fn cropped_ranges_use_count_relative_end_and_wrapper_clamps() {
    let cropped = timing(40., -21., 1.);
    assert_eq!(cropped.first_frame(), 40.);
    assert_eq!(cropped.last_frame(), 80.);
    assert_eq!(cropped.cycle_seconds().unwrap(), Some(2.));
    assert_eq!(timing(0., -1., 1.).last_frame(), 100.);
    assert_eq!(timing(-5., 500., 1.).first_frame(), 0.);
    assert_eq!(timing(-5., 500., 1.).last_frame(), 100.);
    assert_eq!(timing(150., 10., 1.).first_frame(), 100.);
    assert_eq!(timing(150., 10., 1.).last_frame(), 100.);
    assert_eq!(timing(40., -200., 1.).last_frame(), 100.);
    assert_eq!(timing(40., -101., 1.).last_frame(), 40.);
    assert_eq!(timing(40., 10., 1.).last_frame(), 40.);
}
#[test]
fn duration_fit_depends_on_existing_driver_state_and_requested_sign() {
    for (requested, expected) in [(3., 0.5), (-3., -0.5), (0., 0.)] {
        let fitted = ClipTiming::configure(&clip(), 40., 80., 4., requested, false).unwrap();
        assert_eq!(fitted.speed(), expected);
        assert_eq!(fitted.frame_delta(1.).unwrap(), expected * 20.);
        assert_eq!(
            fitted.cycle_seconds().unwrap(),
            if requested == 0. { None } else { Some(4.) }
        );
        let reused = ClipTiming::configure(&clip(), 40., 80., 4., requested, true).unwrap();
        assert_eq!(reused.speed(), requested);
    }
    for duration in [-1., 0., 0.00001] {
        assert_eq!(
            ClipTiming::configure(&clip(), 40., 80., duration, 3., false)
                .unwrap()
                .speed(),
            3.
        );
    }
    let held = ClipTiming::configure(&clip(), 40., 40., 2., 1., false).unwrap();
    assert_eq!(held.speed(), 0.);
    assert_eq!(held.advance(40., 100., false).unwrap(), 40.);
    let mut low_fps = clip();
    low_fps.frames_per_second = 0.000001;
    let sentinel = ClipTiming::configure(&low_fps, 0., -1., 2., 1., false).unwrap();
    assert_eq!(sentinel.speed(), -0.5);
    assert_eq!(sentinel.cycle_seconds().unwrap(), None);
}
#[test]
fn signed_steps_clamp_or_wrap_only_inside_the_cropped_range() {
    let forward = timing(40., 80., 1.);
    assert_eq!(forward.advance(50., 0.5, false).unwrap(), 60.);
    assert_eq!(forward.advance(75., 1., false).unwrap(), 80.);
    assert_eq!(forward.advance(75., 1., true).unwrap(), 55.);
    assert_eq!(forward.advance(50., 4.5, true).unwrap(), 60.);
    assert_eq!(forward.map_frame(80., true).unwrap(), 40.);
    assert_eq!(forward.map_frame(80., false).unwrap(), 80.);
    let reverse = timing(40., 80., -1.);
    assert_eq!(reverse.advance(70., 0.5, false).unwrap(), 60.);
    assert_eq!(reverse.advance(45., 1., false).unwrap(), 40.);
    assert_eq!(reverse.advance(45., 1., true).unwrap(), 65.);
    assert_eq!(reverse.cycle_seconds().unwrap(), Some(2.));
    let tiny = timing(40., 40.000004, 1.);
    assert_eq!(tiny.map_frame(100., true).unwrap(), 40.);
    // Do not snap an already in-range frame using the wrapping epsilon.
    assert_eq!(forward.map_frame(79.99999, true).unwrap(), 79.99999);
    let one = ClipInfo {
        frame_count: 1.,
        ..clip()
    };
    let held = ClipTiming::configure(&one, 0., -1., -1., 1., false).unwrap();
    assert_eq!(held.cycle_seconds().unwrap(), Some(0.));
    assert_eq!(held.advance(0., 10., true).unwrap(), 0.);
}
#[test]
fn invalid_inputs_and_overflow_fail_without_nonfinite_output() {
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for parameters in [
            [bad, -1., -1., 1.],
            [0., bad, -1., 1.],
            [0., -1., bad, 1.],
            [0., -1., -1., bad],
        ] {
            assert!(
                ClipTiming::configure(
                    &clip(),
                    parameters[0],
                    parameters[1],
                    parameters[2],
                    parameters[3],
                    false
                )
                .is_err()
            );
        }
        assert!(timing(0., -1., 1.).map_frame(bad, true).is_err());
        assert!(timing(0., -1., 1.).advance(0., bad, false).is_err());
    }
    for bad in [0., -1., 32769., f32::NAN] {
        let invalid = ClipInfo {
            frame_count: bad,
            ..clip()
        };
        assert!(ClipTiming::configure(&invalid, 0., -1., -1., 1., false).is_err());
    }
    for bad in [0., -1., 1001., f32::INFINITY] {
        let invalid = ClipInfo {
            frames_per_second: bad,
            ..clip()
        };
        assert!(ClipTiming::configure(&invalid, 0., -1., -1., 1., false).is_err());
    }
    assert!(timing(0., -1., 1.).advance(0., -1., false).is_err());
    assert!(timing(0., -1., f32::MAX).frame_delta(1.).is_err());
    assert!(timing(0., -1., 1.).frame_delta(f32::MAX).is_err());
    assert_eq!(timing(0., -1., 0.).advance(10., 100., true).unwrap(), 10.);
}
