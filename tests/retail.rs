//! Opt-in tests: retail data is read in place and never becomes a fixture.
use prototype2_rust::{capsule, collision, fight, meta, p3d, rcf::Archive, scene};

#[test]
#[ignore = "requires PROTOTYPE2_GAME pointing at the user's owned installation"]
fn installed_heller_fixups_bind_and_deform_selected_original_poses() {
    use prototype2_rust::{animation, pose_fixup, skin};
    let game = std::path::PathBuf::from(std::env::var_os("PROTOTYPE2_GAME").unwrap());
    let boot = Archive::open(game.join("boot.rcf")).unwrap();
    let tod = p3d::decode(
        &boot
            .read(boot.find("art\\alex\\alex_tod.p3d").unwrap())
            .unwrap(),
    )
    .unwrap();
    let fixups = pose_fixup::load(&tod, &p3d::parse(&tod).unwrap()).unwrap();
    assert_eq!(
        (fixups.definition_offset, fixups.body_offset),
        (0x36a, 0x3f1)
    );
    assert_eq!(fixups.strategies.len(), 2);
    let pose_fixup::Strategy::Collar(collar) = &fixups.strategies[0] else {
        panic!("wrong first fixup")
    };
    assert_eq!(
        (&*collar.left, &*collar.right, &*collar.chin),
        ("Collar_L", "Collar_R", "Jaw")
    );
    assert_eq!(collar.chin_offset, [0., 0., -0.1]);
    assert_eq!(
        (collar.displacement_power, collar.maximum_displacement),
        (5., 1.)
    );
    let pose_fixup::Strategy::Shoulder { pairs } = &fixups.strategies[1] else {
        panic!("wrong second fixup")
    };
    assert_eq!(pairs.len(), 2);
    let data = p3d::decode(
        &boot
            .read(boot.find("art\\alex\\alex.p3d").unwrap())
            .unwrap(),
    )
    .unwrap();
    let chunks = p3d::parse(&data).unwrap();
    let skeletons: Vec<_> = ["alex_reg_body_skeleton", "alex_reg_arms_skeleton"]
        .into_iter()
        .map(|name| animation::skeleton(&data, &chunks, name).unwrap())
        .collect();
    let art = Archive::open(game.join("art.rcf")).unwrap();
    let model = p3d::decode(
        &art.read(art.find("art\\alex\\alex_model_main.p3d").unwrap())
            .unwrap(),
    )
    .unwrap();
    let meshes = skin::load(&model, &p3d::parse(&model).unwrap(), &skeletons).unwrap();
    let mut changed = false;
    for (name, frame) in [
        ("alex_amb_stand", 20.),
        ("heller_amb_stand", 20.),
        ("heller_loco_walk_n", 18.),
        ("heller_loco_run_n", 10.5),
        ("heller_loco_run_sprint_n", 8.),
        ("heller_loco_jump_from_idle", 50.),
    ] {
        let clip = animation::load_clip(&data, &chunks, name).unwrap();
        for skeleton in &skeletons {
            let bound = fixups.bind(skeleton).unwrap();
            let original = clip.sample(skeleton, frame).unwrap();
            let mut world = original.clone();
            bound.apply(skeleton, &mut world).unwrap();
            for [source, target] in pairs {
                assert!(!clip.tracks.contains_key(target));
                let source = skeleton
                    .joints
                    .iter()
                    .position(|j| j.name == *source)
                    .unwrap();
                let target = skeleton
                    .joints
                    .iter()
                    .position(|j| j.name == *target)
                    .unwrap();
                assert_eq!(world[target], world[source]);
            }
            for mesh in meshes.iter().filter(|m| m.skeleton == skeleton.name) {
                let before = mesh.deform(skeleton, &original).unwrap();
                let after = mesh.deform(skeleton, &world).unwrap();
                assert_eq!(before.positions.len(), after.positions.len());
                changed |= before
                    .positions
                    .iter()
                    .zip(&after.positions)
                    .any(|(a, b)| a.iter().zip(b).any(|(a, b)| (*a - *b).abs() > 1e-5));
            }
        }
    }
    assert!(
        changed,
        "configured pose fixups must affect the skinned corpus"
    );
}

#[test]
#[ignore = "requires PROTOTYPE2_GAME pointing at the user's owned installation"]
fn installed_root_tracks_extract_measured_clip_motion_in_both_spaces() {
    use prototype2_rust::{
        animation,
        root_motion::{self, Interval},
    };
    let game = std::path::PathBuf::from(std::env::var_os("PROTOTYPE2_GAME").unwrap());
    let boot = Archive::open(game.join("boot.rcf")).unwrap();
    let data = p3d::decode(
        &boot
            .read(boot.find("art\\alex\\alex.p3d").unwrap())
            .unwrap(),
    )
    .unwrap();
    let chunks = p3d::parse(&data).unwrap();
    for (name, raw, relative) in [
        ("alex_amb_stand", [0.; 3], [0.; 3]),
        ("heller_loco_walk_n", [0., 0., -1.66], [0., 0., -1.66]),
        (
            "heller_loco_run_n",
            [0., 0., 3.9757304],
            [0., 0., -3.9757304],
        ),
        (
            "heller_loco_run_sprint_n",
            [0., 0., -4.999793],
            [0., 0., -4.999793],
        ),
        (
            "heller_loco_jump_from_idle",
            [0.061584473, 1.0410156, -1.6240234],
            [0.061584473, 1.0410156, -1.6240234],
        ),
    ] {
        let clip = animation::load_clip(&data, &chunks, name).unwrap();
        let track = clip.tracks.get("Motion_Root").unwrap();
        for (relative_translation, expected) in [(false, raw), (true, relative)] {
            let motion = root_motion::delta(
                Some(track),
                Interval {
                    previous: 0.,
                    current: clip.info.last_frame(),
                    first: 0.,
                    last: clip.info.last_frame(),
                    relative_translation,
                    reverse: false,
                },
            )
            .unwrap();
            assert!(
                motion
                    .translation
                    .to_array()
                    .iter()
                    .zip(expected)
                    .all(|(a, b)| (*a - b).abs() < 1e-5),
                "unexpected root displacement for {name}"
            );
            assert!((motion.rotation - glam::Quat::IDENTITY).length() < 1e-5);
        }
    }
}

#[test]
#[ignore = "requires PROTOTYPE2_GAME pointing at the user's owned installation"]
fn installed_character_skeletons_and_skin_bind_pose_agree() {
    use prototype2_rust::{animation, skin};
    let game = std::path::PathBuf::from(std::env::var_os("PROTOTYPE2_GAME").unwrap());
    let boot = Archive::open(game.join("boot.rcf")).unwrap();
    let data = p3d::decode(
        &boot
            .read(boot.find("art\\alex\\alex.p3d").unwrap())
            .unwrap(),
    )
    .unwrap();
    let chunks = p3d::parse(&data).unwrap();
    let skeletons = ["alex_reg_body_skeleton", "alex_reg_arms_skeleton"]
        .map(|name| animation::skeleton(&data, &chunks, name).unwrap());
    assert!(skeletons.iter().all(|s| s.joints.len() == 92));
    let clips = animation::clips(&data, &chunks).unwrap();
    assert_eq!(clips.len(), 931);
    let run = clips
        .iter()
        .find(|c| c.name == "heller_loco_run_n")
        .unwrap();
    assert_eq!(run.offset, 0x77176e);
    assert_eq!(run.frames_per_second, 30.);
    assert_eq!(run.frame_count, 21.);
    assert_eq!(run.last_frame(), 20.);
    assert!(clips.iter().all(|c| c.default_sync_frame == 0.));
    let clip = animation::load_clip(&data, &chunks, "heller_loco_run_n").unwrap();
    assert_eq!(clip.tracks.len(), 62);
    let start = clip.sample(&skeletons[0], 0.).unwrap();
    let middle = clip.sample(&skeletons[0], 10.5).unwrap();
    assert!(start.iter().zip(&middle).any(|(a, b)| a != b));
    for name in [
        "heller_amb_stand",
        "alex_amb_stand",
        "heller_loco_walk_n",
        "heller_loco_jump_from_idle",
        "heller_loco_run_sprint_n",
    ] {
        let clip = animation::load_clip(&data, &chunks, name).unwrap();
        for frame in [
            0.,
            clip.info.last_frame() * 0.25,
            clip.info.last_frame() * 0.5,
            clip.info.last_frame(),
        ] {
            assert!(
                clip.sample(&skeletons[0], frame)
                    .unwrap()
                    .iter()
                    .all(|m| m.is_finite())
            );
        }
    }
    let fig = p3d::decode(
        &boot
            .read(boot.find("art\\alex\\alex_fig.p3d").unwrap())
            .unwrap(),
    )
    .unwrap();
    let graphs = fight::load(&fig, &p3d::parse(&fig).unwrap()).unwrap();
    let graph = graphs.iter().find(|g| g.name == "prototype").unwrap();
    let main: Vec<_> = graph
        .records
        .iter()
        .filter(|r| r.steer.is_some() && graph.branch_matches(r.branch, "bare/loco"))
        .collect();
    assert_eq!(main.len(), 1);
    assert_eq!(main[0].offset, 0x51495);
    let names: Vec<_> = main[0]
        .steer
        .as_ref()
        .unwrap()
        .animations_idle_walk_run
        .iter()
        .map(|hash| {
            animation::resolve_clip(&clips, *hash)
                .unwrap()
                .name
                .as_str()
        })
        .collect();
    assert_eq!(
        names,
        ["alex_amb_stand", "heller_loco_walk_n", "heller_loco_run_n"]
    );
    let cycles: [f32; 3] = names
        .iter()
        .map(|name| {
            clips
                .iter()
                .find(|c| c.name == *name)
                .unwrap()
                .cycle_seconds()
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    assert_eq!(cycles[1], 36. / 30.);
    assert_eq!(cycles[2], 20. / 30.);
    let track = main[0].steer.as_ref().unwrap();
    assert_eq!(track.sync_frames_idle_walk_run, [-1.; 3]);
    let phase = prototype2_rust::animation_driver::advance_phase(
        0.,
        track.velocity_walk,
        [0., track.velocity_walk, track.velocity_run],
        cycles,
        0.12,
    )
    .unwrap();
    assert_eq!(phase.weights, [0., 1., 0.]);
    assert!((phase.phase - 0.1).abs() < 1e-6);
    let walk = clips.iter().find(|c| c.name == names[1]).unwrap();
    assert!(
        (prototype2_rust::animation_driver::frame_at_phase(
            phase.phase,
            walk.last_frame(),
            walk.default_sync_frame,
        )
        .unwrap()
            - 3.6)
            .abs()
            < 1e-6
    );
    let stage_clips: Vec<_> = names
        .iter()
        .map(|name| animation::load_clip(&data, &chunks, name).unwrap())
        .collect();
    let stages = [&stage_clips[0], &stage_clips[1], &stage_clips[2]];
    for speed in [
        0.,
        track.velocity_walk * 0.5,
        track.velocity_walk,
        (track.velocity_walk + track.velocity_run) * 0.5,
        track.velocity_run,
        track.velocity_run * 2.,
    ] {
        let step = prototype2_rust::animation_driver::advance_phase(
            0.25,
            speed,
            [0., track.velocity_walk, track.velocity_run],
            cycles,
            0.,
        )
        .unwrap();
        for skeleton in &skeletons {
            let pose = prototype2_rust::animation_driver::sample_locomotion(
                skeleton,
                stages,
                step.phase,
                step.weights,
                [0.; 3],
            )
            .unwrap();
            assert!(pose.iter().all(|m| m.is_finite()));
            if speed == track.velocity_walk {
                assert_eq!(
                    pose,
                    stage_clips[1]
                        .sample(skeleton, stage_clips[1].info.last_frame() * 0.25)
                        .unwrap()
                );
            }
        }
    }
    let art = Archive::open(game.join("art.rcf")).unwrap();
    let data = p3d::decode(
        &art.read(art.find("art\\alex\\alex_model_main.p3d").unwrap())
            .unwrap(),
    )
    .unwrap();
    let meshes = skin::load(&data, &p3d::parse(&data).unwrap(), &skeletons).unwrap();
    assert_eq!(meshes.len(), 9);
    for mesh in &meshes {
        let s = skeletons.iter().find(|s| s.name == mesh.skeleton).unwrap();
        let deformed = mesh.deform(s, &s.bind_world).unwrap();
        for (p, q) in mesh.positions.iter().zip(deformed.positions) {
            assert!(
                (0..3).all(|i| (p[i] - q[i]).abs() < 0.0001),
                "bind-pose skinning changed vertex"
            );
        }
    }
    println!(
        "931 clip headers, two 92-joint skeletons, nine skinned meshes; bind pose and native locomotion blend endpoints checked"
    );
}

#[test]
#[ignore = "requires PROTOTYPE2_GAME pointing at the user's owned installation"]
fn installed_fight_graph_separates_main_locomotion_from_testing_branch() {
    let game = std::path::PathBuf::from(std::env::var_os("PROTOTYPE2_GAME").unwrap());
    let archive = Archive::open(game.join("boot.rcf")).unwrap();
    let data = p3d::decode(
        &archive
            .read(archive.find("art\\alex\\alex_fig.p3d").unwrap())
            .unwrap(),
    )
    .unwrap();
    let graphs = fight::load(&data, &p3d::parse(&data).unwrap()).unwrap();
    assert_eq!(graphs.len(), 6);
    assert_eq!(graphs.iter().map(|g| g.branches.len()).sum::<usize>(), 2930);
    let prototype = graphs.iter().find(|g| g.name == "prototype").unwrap();
    let steer = prototype
        .records
        .iter()
        .find(|r| r.offset == 0x51495)
        .unwrap();
    assert!(prototype.branch_matches(steer.branch, "bare/loco"));
    let steer = steer.steer.as_ref().unwrap();
    assert_eq!(steer.acceleration, 15.);
    assert_eq!(steer.velocity_walk, 1.5);
    assert_eq!(steer.velocity_run, 4.5);
    assert_eq!(steer.turning_velocity_degrees, 360.);
    assert_eq!(steer.turning_velocity_run_degrees, 720.);
    let capsule = prototype
        .records
        .iter()
        .find(|r| r.offset == 0x519d5)
        .unwrap()
        .capsule
        .as_ref()
        .unwrap();
    assert!(capsule.animate && capsule.radius.enabled);
    assert_eq!(capsule.radius.initial, 0.7);
    assert_eq!(capsule.radius.final_value, 0.5);
    assert_eq!(capsule.offset.initial, [0., 0., -0.1]);
    let testing = prototype
        .records
        .iter()
        .find(|r| r.type_hash == fight::name_hash("locomotion"))
        .unwrap();
    assert!(prototype.branch_matches(testing.branch, "new_feature_testing/locomotion"));
    let sprint_graph = graphs
        .iter()
        .find(|g| g.name == "prototype_sprint")
        .unwrap();
    let sprint = sprint_graph
        .records
        .iter()
        .find(|r| r.offset == 0x107cc1)
        .unwrap();
    assert!(sprint_graph.branch_matches(sprint.branch, "sprint/sprint/sprint"));
    let sprint = sprint.sprint.as_ref().unwrap();
    assert_eq!(sprint.velocities_min_mid_max, [7., 10., 16.]);
    assert_eq!(sprint.accelerations_min_mid_max, [1.5, 0.75, 1.25]);
    assert!(sprint.force_animation_velocities);
    assert_eq!(sprint.unlockable_velocity_min, 9.);
    println!(
        "{} contexts, 2930 branches; main LocoSteer, capsule transition and testing branch checked",
        graphs.len()
    );
}
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
