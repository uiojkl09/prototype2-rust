use glam::{Quat, Vec3};
use prototype2_rust::{
    animation::{JointTrack, Keys},
    root_motion::{Interval, delta},
};

fn interval(previous: f32, current: f32) -> Interval {
    Interval {
        previous,
        current,
        first: 0.,
        last: 10.,
        relative_translation: false,
        reverse: false,
    }
}
fn moving() -> JointTrack {
    JointTrack {
        rotation: None,
        translation: Some(Keys {
            frames: vec![0, 10],
            values: vec![Vec3::ZERO, Vec3::Z * 10.],
        }),
    }
}
fn near(a: Vec3, b: Vec3) {
    assert!((a - b).length() < 1e-5, "{a:?} != {b:?}");
}

#[test]
fn translation_extracts_single_wrap_cropped_bounds_and_reverse_motion() {
    let track = moving();
    near(
        delta(Some(&track), interval(2., 7.)).unwrap().translation,
        Vec3::Z * 5.,
    );
    near(
        delta(Some(&track), interval(8., 2.)).unwrap().translation,
        Vec3::Z * 4.,
    );
    near(
        delta(Some(&track), interval(5., 5.)).unwrap().translation,
        Vec3::ZERO,
    );
    let cropped = Interval {
        first: 2.,
        last: 8.,
        ..interval(7., 3.)
    };
    near(
        delta(Some(&track), cropped).unwrap().translation,
        Vec3::Z * 2.,
    );
    let backwards = Interval {
        reverse: true,
        ..interval(7., 2.)
    };
    near(
        delta(Some(&track), backwards).unwrap().translation,
        -Vec3::Z * 5.,
    );
    let backwards_wrap = Interval {
        reverse: true,
        ..interval(2., 8.)
    };
    near(
        delta(Some(&track), backwards_wrap).unwrap().translation,
        -Vec3::Z * 4.,
    );
}

#[test]
fn orientation_relative_delta_uses_inverse_start_before_end_rotation() {
    let half = std::f32::consts::FRAC_PI_2;
    let mut track = moving();
    track.rotation = Some(Keys {
        frames: vec![0, 10],
        values: vec![Quat::from_rotation_x(half), Quat::from_rotation_y(half)],
    });
    let raw = delta(Some(&track), interval(0., 10.)).unwrap();
    near(raw.translation, Vec3::Z * 10.);
    near(raw.rotation * Vec3::Z, Vec3::X);
    let local = delta(
        Some(&track),
        Interval {
            relative_translation: true,
            ..interval(0., 10.)
        },
    )
    .unwrap();
    near(local.translation, Vec3::Y * 10.);
}

#[test]
fn wrapped_rotation_order_and_segment_translation_frames_are_native() {
    let half = std::f32::consts::FRAC_PI_2;
    let mut track = moving();
    track.rotation = Some(Keys {
        frames: vec![0, 4, 6, 10],
        values: vec![
            Quat::IDENTITY,
            Quat::from_rotation_x(half),
            Quat::from_rotation_y(half),
            Quat::IDENTITY,
        ],
    });
    let result = delta(
        Some(&track),
        Interval {
            relative_translation: true,
            ..interval(6., 4.)
        },
    )
    .unwrap();
    near(result.translation, Vec3::new(-4., 0., 4.));
    near(result.rotation * Vec3::X, -Vec3::Y);
    let backwards = delta(
        Some(&track),
        Interval {
            reverse: true,
            ..interval(4., 6.)
        },
    )
    .unwrap();
    near(backwards.translation, -Vec3::Z * 8.);
    near(backwards.rotation * Vec3::Y, -Vec3::X);
}

#[test]
fn absence_and_invalid_endpoints_keys_and_overflow_are_explicit() {
    let empty = delta(None, interval(0., 10.)).unwrap();
    assert_eq!(empty.translation, Vec3::ZERO);
    assert_eq!(empty.rotation, Quat::IDENTITY);
    let stationary = delta(Some(&JointTrack::default()), interval(8., 2.)).unwrap();
    assert_eq!(stationary.translation, Vec3::ZERO);
    assert_eq!(stationary.rotation, Quat::IDENTITY);
    for bounds in [
        interval(-1., 2.),
        interval(2., 11.),
        interval(f32::NAN, 2.),
        Interval {
            first: 11.,
            ..interval(2., 3.)
        },
    ] {
        assert!(delta(None, bounds).is_err());
    }
    let mut malformed = moving();
    malformed.translation.as_mut().unwrap().frames.clear();
    assert!(delta(Some(&malformed), interval(0., 10.)).is_err());
    let extreme = JointTrack {
        rotation: None,
        translation: Some(Keys {
            frames: vec![0, 10],
            values: vec![Vec3::splat(-f32::MAX), Vec3::splat(f32::MAX)],
        }),
    };
    assert_eq!(
        extreme.sample(0.).unwrap().translation.unwrap(),
        Vec3::splat(-f32::MAX)
    );
    assert!(delta(Some(&extreme), interval(0., 10.)).is_err());
}
