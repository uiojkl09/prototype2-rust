use prototype2_rust::{
    capsule::Capsule,
    capsule_state::CapsuleAction,
    fight::{Animated, CapsuleTrack, TrackHeader},
};

fn shape() -> Capsule {
    Capsule {
        centre: [0., 0.175, 0.],
        axis: [0., 1., 0.],
        extent: 0.175,
        radius: 0.5,
    }
}
fn track() -> CapsuleTrack {
    CapsuleTrack {
        header: TrackHeader {
            reference_index: -1,
            is_slave: false,
            time_begin: 0.,
            time_end: 0.2,
        },
        attach_to_joint: 0,
        animate: true,
        offset: Animated {
            enabled: true,
            initial: [0., 0., -0.1],
            final_value: [0.; 3],
        },
        extent: Animated {
            enabled: false,
            initial: 0.,
            final_value: 0.,
        },
        radius: Animated {
            enabled: true,
            initial: 0.7,
            final_value: 0.5,
        },
        axis: Animated {
            enabled: false,
            initial: [0.; 3],
            final_value: [0.; 3],
        },
        rotation_degrees: Animated {
            enabled: false,
            initial: [0.; 3],
            final_value: [0.; 3],
        },
    }
}
#[test]
fn transition_offsets_do_not_accumulate_and_end_restores_previous_shape() {
    let (action, initial) = CapsuleAction::begin(shape(), track()).unwrap();
    assert_eq!(initial.radius, 0.7);
    assert_eq!(initial.centre, [0., 0.175, -0.1]);
    let half = action.sample(0.1).unwrap();
    assert!((half.radius - 0.6).abs() < 1e-6);
    assert_eq!(half.centre, [0., 0.175, -0.05]);
    assert_eq!(action.sample(0.1).unwrap().centre, half.centre);
    assert_eq!(action.sample(1.).unwrap().centre, shape().centre);
    assert_eq!(action.end().radius, 0.5);
}
#[test]
fn static_and_zero_duration_actions_differ_and_unknown_transforms_fail() {
    let mut static_track = track();
    static_track.animate = false;
    let (action, _) = CapsuleAction::begin(shape(), static_track).unwrap();
    assert_eq!(action.sample(10.).unwrap().radius, 0.7);
    let mut immediate = track();
    immediate.header.time_end = 0.;
    let (action, initial) = CapsuleAction::begin(shape(), immediate).unwrap();
    assert_eq!(initial.radius, 0.7);
    assert_eq!(action.sample(0.).unwrap().radius, 0.5);
    assert!(action.sample(f32::NAN).is_err());
    let mut joint = track();
    joint.attach_to_joint = 1;
    assert!(CapsuleAction::begin(shape(), joint).is_err());
    let mut rotation = track();
    rotation.rotation_degrees.enabled = true;
    assert!(CapsuleAction::begin(shape(), rotation).is_err());
}

#[test]
fn begin_and_static_sampling_do_not_interpolate_unused_final_values() {
    let mut static_track = track();
    static_track.animate = false;
    static_track.offset.initial = [-f32::MAX, 0., 0.];
    static_track.offset.final_value = [f32::MAX, 0., 0.];
    let (action, initial) = CapsuleAction::begin(shape(), static_track).unwrap();
    assert_eq!(initial.centre[0], -f32::MAX);
    assert_eq!(action.sample(10.).unwrap().centre, initial.centre);
    let mut bad = track();
    bad.header.time_end = f32::NAN;
    assert!(CapsuleAction::begin(shape(), bad).is_err());
}
