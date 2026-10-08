use prototype2_rust::{
    capsule::{self, Capsule},
    collision,
    scene::CollisionMesh,
};

fn shape() -> Capsule {
    Capsule {
        centre: [0., 1., 0.],
        axis: [0., 1., 0.],
        extent: 0.5,
        radius: 0.25,
    }
}
fn wall() -> CollisionMesh {
    CollisionMesh {
        chunk_offset: 42,
        positions: vec![
            [0., -10., -10.],
            [0., 10., -10.],
            [0., 10., 10.],
            [0., -10., 10.],
        ],
        faces: vec![[0, 1, 2, 9], [0, 2, 3, 11]],
        unknown_tail_bytes: 0,
    }
}
fn body() -> Vec<u8> {
    let mut b = b"META".to_vec();
    b.extend(37u32.to_le_bytes());
    b.extend(b"ravenphysics::CollisionCapsuleFactory");
    b.extend(1u16.to_le_bytes());
    b.extend(0x9ca40a36u32.to_le_bytes());
    for label in ["TestMaterial", "TestGroup"] {
        b.push(label.len() as u8);
        b.extend(label.as_bytes());
    }
    for value in [0f32, 1., 0., 0., 1., 0., 0.5, 0.25] {
        b.extend(value.to_le_bytes());
    }
    b.extend([0; 13]);
    b
}
#[test]
fn capsule_asset_rejects_wrong_version_truncation_and_invalid_geometry() {
    let b = body();
    let (_, _, s, tail) = capsule::decode_body(&b).unwrap();
    assert_eq!(s.endpoints([0.; 3]), [[0., 0.5, 0.], [0., 1.5, 0.]]);
    assert_eq!(tail, 13);
    for cut in 0..b.len() {
        assert!(capsule::decode_body(&b[..cut]).is_err());
    }
    let mut bad = b.clone();
    bad[45] = 2;
    assert!(capsule::decode_body(&bad).is_err());
    let mut bad = b.clone();
    bad.push(0);
    assert!(capsule::decode_body(&bad).is_err());
    for invalid in [0., -1., f32::NAN, f32::INFINITY] {
        let mut s = shape();
        s.radius = invalid;
        assert!(s.validate().is_err());
    }
    let mut s = shape();
    s.axis = [0.; 3];
    assert!(s.validate().is_err());
}
#[test]
fn fast_sweep_stops_at_wall_without_tunnelling_and_preserves_tag() {
    let hit = collision::sweep(&[wall()], &shape(), [-10., 0., 0.], [1000., 0., 0.], 0.)
        .unwrap()
        .unwrap();
    assert!((hit.fraction - 0.00975).abs() < 1e-10);
    assert_eq!(hit.normal, [-1., 0., 0.]);
    assert!(!hit.initial_overlap);
    assert_eq!(hit.chunk_offset, 42);
    assert!([9, 11].contains(&hit.unknown_face_tag));
    assert!(
        collision::sweep(&[wall()], &shape(), [-10., 0., 0.], [-1000., 0., 0.], 0.)
            .unwrap()
            .is_none()
    );
}
#[test]
fn two_sided_query_is_independent_of_triangle_winding() {
    let mut mesh = wall();
    for face in &mut mesh.faces {
        face.swap(0, 2);
    }
    let h = collision::sweep(&[mesh], &shape(), [10., 0., 0.], [-20., 0., 0.], 0.)
        .unwrap()
        .unwrap();
    assert!((h.fraction - 0.4875).abs() < 1e-10);
    assert_eq!(h.normal, [1., 0., 0.]);
}
#[test]
fn tangential_floor_contact_allows_travel_but_downward_sweep_hits() {
    let floor = CollisionMesh {
        chunk_offset: 1,
        positions: vec![[-100., 0., -100.], [100., 0., -100.], [0., 0., 100.]],
        faces: vec![[0, 1, 2, 0]],
        unknown_tail_bytes: 0,
    };
    let origin = [0., -0.25, 0.];
    assert!(
        collision::sweep(
            std::slice::from_ref(&floor),
            &shape(),
            origin,
            [10., 0., 0.],
            0.
        )
        .unwrap()
        .is_none()
    );
    let h = collision::sweep(&[floor], &shape(), [0., 10., 0.], [0., -100., 0.], 0.)
        .unwrap()
        .unwrap();
    assert!((h.fraction - 0.1025).abs() < 1e-10);
    assert_eq!(h.normal, [0., 1., 0.]);
}
#[test]
fn edge_and_capsule_side_contacts_are_detected() {
    let triangle = CollisionMesh {
        chunk_offset: 1,
        positions: vec![[0., 0.9, -1.], [0., 1.1, -1.], [0., 1., 1.]],
        faces: vec![[0, 1, 2, 0]],
        unknown_tail_bytes: 0,
    };
    // Both segment endpoints lie outside this small triangle; its middle intersects.
    let h = collision::sweep(&[triangle], &shape(), [-1., 0., 0.], [2., 0., 0.], 0.)
        .unwrap()
        .unwrap();
    assert!((h.fraction - 0.375).abs() < 1e-10);
    let h = collision::sweep(&[wall()], &shape(), [-1., 0., 10.1], [2., 0., 0.], 0.)
        .unwrap()
        .unwrap();
    let expected = (1. - (0.25f64.powi(2) - 0.1f64.powi(2)).sqrt()) / 2.;
    assert!((h.fraction - expected).abs() < 1e-8);
}
#[test]
fn overlap_and_malformed_queries_are_reported() {
    let h = collision::sweep(&[wall()], &shape(), [0., 0., 0.], [0.; 3], 0.)
        .unwrap()
        .unwrap();
    assert!(h.initial_overlap);
    assert_eq!(h.fraction, 0.);
    assert!(collision::sweep(&[wall()], &shape(), [f64::NAN, 0., 0.], [1.; 3], 0.).is_err());
    let mut malformed = wall();
    malformed.faces[0][0] = 99;
    assert!(collision::sweep(&[malformed], &shape(), [-1., 0., 0.], [2., 0., 0.], 0.).is_err());
}

#[test]
fn degenerate_point_and_sweep_endpoint_contacts_are_not_dropped() {
    let point = CollisionMesh {
        chunk_offset: 3,
        positions: vec![[0., 1., 0.]],
        faces: vec![[0, 0, 0, 0]],
        unknown_tail_bytes: 0,
    };
    let h = collision::sweep(
        std::slice::from_ref(&point),
        &shape(),
        [-1., 0., 0.],
        [0.75, 0., 0.],
        0.,
    )
    .unwrap()
    .unwrap();
    assert!((h.fraction - 1.).abs() < 1e-10);
    let h = collision::sweep(&[point], &shape(), [0.; 3], [0.; 3], 0.)
        .unwrap()
        .unwrap();
    assert!(h.initial_overlap);
}
