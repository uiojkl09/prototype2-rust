//! Opt-in tests: retail data is read in place and never becomes a fixture.
use prototype2_rust::{meta, p3d, rcf::Archive, scene};
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
    println!("{}", serde_json::to_string_pretty(&objects).unwrap());
}
