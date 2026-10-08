//! Opt-in tests: retail data is read in place and never becomes a fixture.
use prototype2_rust::{capsule, collision, meta, p3d, rcf::Archive, scene};
#[test]
#[ignore = "requires PROTOTYPE2_GAME pointing at the user's owned installation"]
fn installed_cell_decodes_and_references_real_ground_collision() {
    let game = std::path::PathBuf::from(
        std::env::var_os("PROTOTYPE2_GAME").expect("PROTOTYPE2_GAME is required"),
    );
    let a = Archive::open(game.join("cells.rcf")).unwrap();
    let raw = a
        .read(
            a.find("art\\locations\\yellow_zone\\Cell_29.p3d.rz")
                .unwrap(),
        )
        .unwrap();
    let data = p3d::decode(&raw).unwrap();
    let chunks = p3d::parse(&data).unwrap();
    let s = scene::load(&data, &chunks).unwrap();
    assert!(s.summary.world_meshes > 0 && s.summary.world_triangles > 0);
    assert!(s.summary.ground_collision_triangles > 0);
    let b = s.summary.bounds;
    let mut hits = 0;
    for ix in 1..10 {
        for iz in 1..10 {
            let x = b[0][0] + (b[1][0] - b[0][0]) * ix as f32 / 10.;
            let z = b[0][2] + (b[1][2] - b[0][2]) * iz as f32 / 10.;
            if scene::raycast(&s.collision, [x, b[1][1] + 10., z], [0., -1., 0.]).is_some() {
                hits += 1;
            }
        }
    }
    assert!(hits > 0, "no downward ray hit in section bounds");
    println!("{}", serde_json::to_string_pretty(&s.summary).unwrap());
    println!("{hits}/81 downward sample rays hit geometric ground triangles");
}

#[test]
#[ignore = "requires PROTOTYPE2_GAME pointing at the user's owned installation"]
fn installed_character_metadata_locates_capsule_reference() {
    let game = std::path::PathBuf::from(
        std::env::var_os("PROTOTYPE2_GAME").expect("PROTOTYPE2_GAME is required"),
    );
    let archive = Archive::open(game.join("boot.rcf")).unwrap();
    let raw = archive
        .read(archive.find("art\\alex\\alex_tod.p3d").unwrap())
        .unwrap();
    let data = p3d::decode(&raw).unwrap();
    let chunks = p3d::parse(&data).unwrap();
    let objects = meta::inspect(&data, &chunks, "CollisionCapsuleFactory").unwrap();
    assert_eq!(objects.len(), 1);
    assert_eq!(objects[0].short_name, "AlexPhysicsFactory");
    assert_eq!(objects[0].body_bytes, 128);
    assert!(objects[0].has_meta_signature);
    assert_eq!(objects[0].matched_reference_offsets, [0x3bb3]);
    let factory = capsule::load(&data, &chunks).unwrap();
    assert_eq!(factory.material_reference, "AlexFrictionlessFlesh");
    assert_eq!(factory.intersection_properties_reference, "Character");
    assert_eq!(factory.shape.centre, [0., 0.175, 0.]);
    assert_eq!(factory.shape.axis, [0., 1., 0.]);
    assert_eq!(factory.shape.extent, 0.175);
    assert_eq!(factory.shape.radius, 0.5);
    assert_eq!(factory.unknown_tail_bytes, 13);
    println!("{}", serde_json::to_string_pretty(&objects).unwrap());
}

#[test]
#[ignore = "requires PROTOTYPE2_GAME pointing at the user's owned installation"]
fn installed_capsule_sweep_matches_flat_surface_in_real_section() {
    let game = std::path::PathBuf::from(std::env::var_os("PROTOTYPE2_GAME").unwrap());
    let boot = Archive::open(game.join("boot.rcf")).unwrap();
    let data = p3d::decode(
        &boot
            .read(boot.find("art\\alex\\alex_tod.p3d").unwrap())
            .unwrap(),
    )
    .unwrap();
    let shape = capsule::load(&data, &p3d::parse(&data).unwrap())
        .unwrap()
        .shape;
    let cells = Archive::open(game.join("cells.rcf")).unwrap();
    let data = p3d::decode(
        &cells
            .read(
                cells
                    .find("art\\locations\\yellow_zone\\Cell_29.p3d.rz")
                    .unwrap(),
            )
            .unwrap(),
    )
    .unwrap();
    let world = scene::load(&data, &p3d::parse(&data).unwrap()).unwrap();
    let origin = [1218.130, 55.001, -1898.738];
    let ray = scene::raycast(&world.collision, origin.map(|x| x as f32), [0., -1., 0.]).unwrap();
    let hit = collision::sweep(&world.collision, &shape, origin, [0., -50., 0.], 0.)
        .unwrap()
        .unwrap();
    assert!(!hit.initial_overlap);
    assert!(
        hit.normal[1] > 0.99,
        "expected previously inspected flat patch"
    );
    assert!((hit.fraction * 50. - (f64::from(ray) - 0.5)).abs() < 0.002);
    println!("{}", serde_json::to_string_pretty(&hit).unwrap());
}
