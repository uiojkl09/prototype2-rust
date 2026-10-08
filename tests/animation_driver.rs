use glam::{Mat4, Quat, Vec3};
use prototype2_rust::animation_driver::{advance_phase, frame_at_phase};
use prototype2_rust::{
    animation::{Clip, ClipInfo, Joint, JointTrack, Keys, Skeleton},
    animation_driver::sample_locomotion,
};
use std::collections::HashMap;

fn rig() -> Skeleton {
    let tip = Mat4::from_translation(Vec3::X);
    Skeleton {
        name: "synthetic".into(),
        joints: vec![
            Joint {
                name: "root".into(),
                parent: None,
                bind_local: Mat4::IDENTITY,
            },
            Joint {
                name: "tip".into(),
                parent: Some(0),
                bind_local: tip,
            },
        ],
        bind_world: vec![Mat4::IDENTITY, tip],
        inverse_bind: vec![Mat4::IDENTITY, tip.inverse()],
    }
}
fn clip(name: &str, rotation: Quat, translation: Vec3) -> Clip {
    Clip {
        info: ClipInfo {
            name: name.into(),
            offset: 0,
            kind: *b"PTRN",
            frame_count: 11.,
            frames_per_second: 10.,
            cyclic: true,
            default_sync_frame: 0.,
        },
        tracks: HashMap::from([(
            "root".into(),
            JointTrack {
                rotation: Some(Keys {
                    frames: vec![0],
                    values: vec![rotation],
                }),
                translation: Some(Keys {
                    frames: vec![0],
                    values: vec![translation],
                }),
            },
        )]),
    }
}
#[test]
fn locomotion_blends_local_components_before_parent_composition() {
    let rig = rig();
    let clips = [
        clip("idle", Quat::IDENTITY, Vec3::ZERO),
        clip(
            "walk",
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            Vec3::X * 2.,
        ),
        clip(
            "run",
            Quat::from_rotation_z(std::f32::consts::PI),
            Vec3::X * 4.,
        ),
    ];
    let pose = sample_locomotion(
        &rig,
        [&clips[0], &clips[1], &clips[2]],
        0.4,
        [0., 0.5, 0.5],
        [0.; 3],
    )
    .unwrap();
    let tip = pose[1].transform_point3(Vec3::ZERO);
    let a = 1. / 2f32.sqrt();
    assert!((tip - Vec3::new(3. - a, a, 0.)).length() < 1e-5);
    let endpoint = sample_locomotion(
        &rig,
        [&clips[0], &clips[1], &clips[2]],
        0.4,
        [0., 1., 0.],
        [0.; 3],
    )
    .unwrap();
    assert_eq!(endpoint, clips[1].sample(&rig, 4.).unwrap());
    assert!(
        sample_locomotion(
            &rig,
            [&clips[0], &clips[1], &clips[2]],
            0.,
            [0.5, 0., 0.5],
            [0.; 3]
        )
        .is_err()
    );
    assert!(
        sample_locomotion(
            &rig,
            [&clips[0], &clips[1], &clips[2]],
            0.,
            [0.; 3],
            [0.; 3]
        )
        .is_err()
    );
    assert!(
        sample_locomotion(
            &rig,
            [&clips[0], &clips[0], &clips[2]],
            0.,
            [1., 0., 0.],
            [0.; 3]
        )
        .is_err()
    );
    let mut overflow = rig.clone();
    overflow.joints[0].bind_local = Mat4::from_scale(Vec3::splat(f32::MAX));
    overflow.joints[1].bind_local = Mat4::from_scale(Vec3::splat(2.));
    assert!(clips[0].sample(&overflow, 0.).is_err());
}

#[test]
fn pose_blend_respects_antipodal_rotations_and_native_zero_dot_hemisphere() {
    let rig = rig();
    let walk = clip("walk", Quat::IDENTITY, Vec3::ZERO);
    let idle = clip("idle", Quat::IDENTITY, Vec3::ZERO);
    let run = clip("run", Quat::from_xyzw(0., 0., 1., 0.), Vec3::ZERO);
    let pose = sample_locomotion(&rig, [&idle, &walk, &run], 0., [0., 0.5, 0.5], [0.; 3]).unwrap();
    assert!((pose[1].transform_point3(Vec3::ZERO) + Vec3::Y).length() < 1e-5);
    let run = clip("run", -Quat::IDENTITY, Vec3::ZERO);
    let pose = sample_locomotion(&rig, [&idle, &walk, &run], 0., [0., 0.5, 0.5], [0.; 3]).unwrap();
    assert!((pose[1].transform_point3(Vec3::ZERO) - Vec3::X).length() < 1e-5);
}

#[test]
fn clip_frame_mapping_applies_sync_offset_and_wraps_both_directions() {
    assert_eq!(frame_at_phase(0.5, 20., 0.).unwrap(), 10.);
    assert_eq!(frame_at_phase(0.5, 36., 0.).unwrap(), 18.);
    assert_eq!(frame_at_phase(0.75, 20., 10.).unwrap(), 5.);
    assert_eq!(frame_at_phase(0., 20., -5.).unwrap(), 15.);
    assert_eq!(frame_at_phase(0.5, 20., 50.).unwrap(), 0.);
    assert_eq!(frame_at_phase(0., 0., 0.).unwrap(), 0.);
    assert_eq!(frame_at_phase(0., 20., 20.000002).unwrap(), 0.);
    assert!(frame_at_phase(1., 20., 0.).is_err());
    assert!(frame_at_phase(0., -1., 0.).is_err());
    assert!(frame_at_phase(0., 20., f32::NAN).is_err());
    assert!(frame_at_phase(0.9, f32::MAX, f32::MAX).is_err());
}

#[test]
fn stages_blend_by_speed_and_phase_uses_weighted_cycles_and_overrun_ratio() {
    let speeds = [0., 1.5, 4.5];
    let cycles = [2., 1.2, 2. / 3.];
    let idle = advance_phase(0., 0., speeds, cycles, 0.2).unwrap();
    assert_eq!(idle.weights, [1., 0., 0.]);
    assert!((idle.phase - 0.1).abs() < 1e-6);
    let walk = advance_phase(0., 1.5, speeds, cycles, 0.12).unwrap();
    assert_eq!(walk.weights, [0., 1., 0.]);
    assert!((walk.phase - 0.1).abs() < 1e-6);
    let mixed = advance_phase(0., 3., speeds, cycles, 0.2).unwrap();
    assert_eq!(mixed.weights, [0., 0.5, 0.5]);
    assert!((mixed.cycle_seconds - 14. / 15.).abs() < 1e-6);
    assert!((mixed.phase - 3. / 14.).abs() < 1e-6);
    let doubled = advance_phase(0., 9., speeds, cycles, 0.1).unwrap();
    assert_eq!(doubled.weights, [0., 0., 1.]);
    assert!((doubled.phase - 0.3).abs() < 1e-6);
}

#[test]
fn minor_weights_are_removed_and_phase_wrap_handles_cycle_boundaries() {
    let speeds = [0., 1., 2.];
    let cycles = [1.; 3];
    assert_eq!(
        advance_phase(0., 0.005, speeds, cycles, 0.)
            .unwrap()
            .weights,
        [1., 0., 0.]
    );
    assert_eq!(
        advance_phase(0., 0.01, speeds, cycles, 0.).unwrap().weights,
        [0.99, 0.01, 0.]
    );
    assert_eq!(
        advance_phase(0.75, 0., speeds, cycles, 0.25).unwrap().phase,
        0.
    );
    assert_eq!(
        advance_phase(0.75, 0., speeds, cycles, 3.5).unwrap().phase,
        0.25
    );
    assert_eq!(
        advance_phase(0.75, 0., speeds, [0.; 3], 1.).unwrap().phase,
        0.75
    );
    assert!(advance_phase(f32::NAN, 0., speeds, cycles, 0.).is_err());
    assert!(advance_phase(0., 0., [0., 1., 1.], cycles, 0.).is_err());
    assert!(advance_phase(0., -1., speeds, cycles, 0.).is_err());
    assert!(advance_phase(0., 0., speeds, cycles, -1.).is_err());
    assert!(advance_phase(0., 4., speeds, cycles, f32::MAX).is_err());
}
