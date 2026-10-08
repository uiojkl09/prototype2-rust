use glam::{Mat4, Quat, Vec3};
use prototype2_rust::{
    animation::{Joint, Skeleton, world_matrices},
    pose_fixup::{Collar, Fixups, Strategy, decode_body},
};

fn rig(jaw_z: f32) -> Skeleton {
    let definitions = [
        (
            "root",
            None,
            Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2),
        ),
        (
            "parent",
            Some(0),
            Mat4::from_translation(Vec3::new(3., 2., 1.)),
        ),
        ("chin_parent", Some(1), Mat4::IDENTITY),
        ("chin", Some(2), Mat4::from_translation(Vec3::Z * jaw_z)),
        ("left", Some(1), Mat4::from_translation(-Vec3::Z)),
        ("right", Some(1), Mat4::from_translation(Vec3::Z)),
        ("shoulder", Some(1), Mat4::from_translation(Vec3::Y)),
        ("corrective", Some(1), Mat4::from_translation(Vec3::X * 2.)),
    ];
    let joints: Vec<_> = definitions
        .into_iter()
        .map(|(name, parent, bind_local)| Joint {
            name: name.into(),
            parent,
            bind_local,
        })
        .collect();
    let bind_world = world_matrices(
        &joints,
        &joints.iter().map(|j| j.bind_local).collect::<Vec<_>>(),
    )
    .unwrap();
    Skeleton {
        name: "synthetic".into(),
        joints,
        inverse_bind: bind_world.iter().map(|m| m.inverse()).collect(),
        bind_world,
    }
}
fn collar() -> Strategy {
    Strategy::Collar(Collar {
        left: "left".into(),
        right: "right".into(),
        chin: "chin".into(),
        chin_offset: [0.; 3],
        displacement_power: 2.,
        maximum_displacement: 1.,
    })
}
fn fixups(strategies: Vec<Strategy>) -> Fixups {
    Fixups {
        definition_offset: 0,
        body_offset: 0,
        strategies,
    }
}
fn near(actual: Vec3, expected: Vec3) {
    assert!(
        (actual - expected).length() < 1e-5,
        "{actual:?} != {expected:?}"
    );
}

#[test]
fn corrective_pose_copies_the_source_under_the_shared_parent() {
    let skeleton = rig(0.);
    let bound = fixups(vec![Strategy::Shoulder {
        pairs: vec![["shoulder".into(), "corrective".into()]],
    }])
    .bind(&skeleton)
    .unwrap();
    let mut world = skeleton.bind_world.clone();
    let local = Mat4::from_rotation_translation(
        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        Vec3::new(2., 1., 3.),
    );
    world[6] = world[1] * local;
    let untouched = world[4];
    bound.apply(&skeleton, &mut world).unwrap();
    assert_eq!(world[7], world[6]);
    near(world[7].transform_point3(Vec3::Y), Vec3::new(5., 3., -5.));
    assert_eq!(world[4], untouched);
}

#[test]
fn collar_uses_parent_space_signed_power_upper_limit_and_bind_translation() {
    let skeleton = rig(0.);
    let bound = fixups(vec![collar()]).bind(&skeleton).unwrap();
    for (displacement, left, right) in [
        (0., -1., 1.),
        (0.1, -1., 1.21),
        (0.5, -1., 2.),
        (-0.5, -2., 1.),
    ] {
        let mut world = skeleton.bind_world.clone();
        world[3] = world[1] * Mat4::from_translation(Vec3::Z * displacement);
        // Translation resets from bind, while current local orientation survives.
        let orientation = Mat4::from_rotation_x(std::f32::consts::FRAC_PI_2);
        world[4] = world[1] * Mat4::from_translation(Vec3::new(9., 8., 7.)) * orientation;
        bound.apply(&skeleton, &mut world).unwrap();
        let inverse = world[1].inverse();
        near((inverse * world[4]).w_axis.truncate(), Vec3::Z * left);
        near((inverse * world[5]).w_axis.truncate(), Vec3::Z * right);
        near((inverse * world[4]).transform_vector3(Vec3::Y), Vec3::Z);
    }
    // Native clamp is only an upper limit; a negative computed shift survives.
    let skeleton = rig(2.);
    let bound = fixups(vec![collar()]).bind(&skeleton).unwrap();
    let mut world = skeleton.bind_world.clone();
    world[3] = world[1] * Mat4::from_translation(Vec3::Z * 2.5);
    bound.apply(&skeleton, &mut world).unwrap();
    near(
        (world[1].inverse() * world[5]).w_axis.truncate(),
        -Vec3::Z * 0.25,
    );
    // A rotated chin projects the configured offset as a vector in its bind
    // parent's inverse orientation; translated world origin must not enter it.
    let skeleton = rig(0.);
    let mut offset = collar();
    if let Strategy::Collar(c) = &mut offset {
        c.chin_offset = [0., 0., -0.2];
    }
    let bound = fixups(vec![offset]).bind(&skeleton).unwrap();
    let mut world = skeleton.bind_world.clone();
    world[3] = world[1] * Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2);
    bound.apply(&skeleton, &mut world).unwrap();
    near(
        (world[1].inverse() * world[4]).w_axis.truncate(),
        -Vec3::Z * 1.44,
    );
}

#[test]
fn unsupported_hierarchies_changed_rigs_and_nonfinite_math_fail() {
    let skeleton = rig(0.);
    let shoulder = Strategy::Shoulder {
        pairs: vec![["shoulder".into(), "corrective".into()]],
    };
    for strategy in [
        Strategy::Shoulder {
            pairs: vec![["missing".into(), "corrective".into()]],
        },
        Strategy::Shoulder {
            pairs: vec![["shoulder".into(), "chin_parent".into()]],
        },
        Strategy::Shoulder {
            pairs: vec![["shoulder".into(), "shoulder".into()]],
        },
    ] {
        assert!(fixups(vec![strategy]).bind(&skeleton).is_err());
    }
    assert!(
        fixups(vec![shoulder.clone(), shoulder])
            .bind(&skeleton)
            .is_err()
    );
    let bound = fixups(vec![collar()]).bind(&skeleton).unwrap();
    let mut changed = skeleton.clone();
    changed.joints[3].parent = Some(99);
    assert!(
        bound
            .apply(&changed, &mut skeleton.bind_world.clone())
            .is_err()
    );
    let mut world = skeleton.bind_world.clone();
    world[1] = Mat4::ZERO;
    assert!(bound.apply(&skeleton, &mut world).is_err());
    world[1] = skeleton.bind_world[1];
    world[3].w_axis.x = f32::NAN;
    assert!(bound.apply(&skeleton, &mut world).is_err());
    let mut extreme = collar();
    if let Strategy::Collar(c) = &mut extreme {
        c.displacement_power = f32::MAX;
    }
    let bound = fixups(vec![extreme]).bind(&skeleton).unwrap();
    world[3] = world[1] * Mat4::from_translation(Vec3::Z * 10.);
    assert!(bound.apply(&skeleton, &mut world).is_err());
}

fn string(bytes: &mut Vec<u8>, text: &str) {
    bytes.extend_from_slice(&(text.len() as u32).to_le_bytes());
    bytes.extend_from_slice(text.as_bytes());
}
fn encoded() -> Vec<u8> {
    let mut bytes = b"META".to_vec();
    bytes.extend_from_slice(&2u32.to_le_bytes());
    string(&mut bytes, "test_collar");
    string(&mut bytes, "PoseStrategyCollarRig");
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&0x08c2bd93u32.to_le_bytes());
    for name in ["left", "right", "chin"] {
        string(&mut bytes, name);
    }
    for value in [0f32, 0., -0.2, 2., 1.] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(&[1, 0, 0, 0, 1, 0, 0, 0]);
    string(&mut bytes, "test_shoulder");
    string(&mut bytes, "PoseStrategyShoulderCon");
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&0x1a80a495u32.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    for name in ["shoulder", "corrective"] {
        string(&mut bytes, name);
    }
    bytes.extend_from_slice(&[1, 0, 0, 0, 1, 0, 0, 0]);
    bytes
}
#[test]
fn nested_layout_is_bounded_and_rejects_truncation_versions_and_unknown_fields() {
    let bytes = encoded();
    let strategies = decode_body(&bytes).unwrap();
    assert_eq!(strategies.len(), 2);
    let Strategy::Collar(parameters) = &strategies[0] else {
        panic!("wrong order")
    };
    assert_eq!(parameters.chin_offset, [0., 0., -0.2]);
    let Strategy::Shoulder { pairs } = &strategies[1] else {
        panic!("wrong order")
    };
    assert_eq!(pairs, &[["shoulder".to_owned(), "corrective".to_owned()]]);
    for n in 0..bytes.len() {
        assert!(
            decode_body(&bytes[..n]).is_err(),
            "accepted truncated length {n}"
        );
    }
    for position in [0, 4, 8, bytes.len() - 4] {
        let mut bad = bytes.clone();
        bad[position] = 0;
        assert!(decode_body(&bad).is_err());
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(decode_body(&trailing).is_err());
    let mut token = bytes.clone();
    let position = token
        .windows(4)
        .position(|w| w == 0x08c2bd93u32.to_le_bytes())
        .unwrap();
    token[position] ^= 1;
    assert!(decode_body(&token).is_err());
    let mut version = bytes.clone();
    version[position - 2] = 2;
    assert!(decode_body(&version).is_err());
}
