use bevy::{
    app::AppExit,
    asset::RenderAssetUsages,
    camera::Exposure,
    ecs::system::SystemParam,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use prototype2_rust::{
    controller,
    scene::{MeshData, Scene, raycast},
};
use std::path::PathBuf;

#[derive(Resource)]
struct WorldData(Scene);
#[derive(Resource)]
struct Session {
    frames: u32,
    stop_after: Option<u32>,
    screenshot: Option<PathBuf>,
    captured: bool,
    home: Transform,
}
#[derive(Component)]
struct CollisionOverlay;

#[derive(Resource, Default)]
struct Controller {
    tracker: controller::Tracker,
    settings: controller::Settings,
    next_probe: f64,
}

#[derive(SystemParam)]
struct InputDevices<'w, 's> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    controller: ResMut<'w, Controller>,
    window: Query<'w, 's, &'static Window, With<bevy::window::PrimaryWindow>>,
}

pub fn run(
    world: Scene,
    frames: Option<u32>,
    screenshot: Option<PathBuf>,
    settings: controller::Settings,
) {
    let low = Vec3::from_array(world.summary.bounds[0]);
    let high = Vec3::from_array(world.summary.bounds[1]);
    let center = (low + high) * 0.5;
    // An inspection camera inside the section avoids the lair roof obscuring the first view.
    // This heuristic is not a recovered character spawn or a gameplay camera.
    let mut eye = center + Vec3::new(0.0, 0.0, (high.z - low.z) * 0.25);
    let mut best_space = 0.0;
    for ix in 1..10 {
        for iz in 1..10 {
            let origin = Vec3::new(
                low.x + (high.x - low.x) * ix as f32 / 10.0,
                center.y,
                low.z + (high.z - low.z) * iz as f32 / 10.0,
            );
            let Some(distance) =
                raycast(&world.collision, origin.to_array(), Vec3::NEG_Y.to_array())
            else {
                continue;
            };
            let candidate = origin + Vec3::Y * (2.0 - distance);
            let ceiling =
                raycast(&world.collision, candidate.to_array(), Vec3::Y.to_array()).unwrap_or(10.0);
            if ceiling < 2.0 {
                continue;
            }
            let space: f32 = [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z]
                .iter()
                .map(|dir| {
                    raycast(&world.collision, candidate.to_array(), dir.to_array())
                        .unwrap_or(20.0)
                        .min(20.0)
                })
                .sum();
            if space > best_space {
                best_space = space;
                eye = candidate;
            }
        }
    }
    let target = Vec3::new(center.x, eye.y - 0.5, center.z);
    let home = Transform::from_translation(eye).looking_at(target, Vec3::Y);
    println!(
        "Inspection camera: {:.3},{:.3},{:.3} (heuristic, not a retail spawn)",
        eye.x, eye.y, eye.z
    );
    App::new()
        .insert_resource(WorldData(world))
        .insert_resource(Controller { tracker: controller::Tracker::default(), settings, next_probe: 0.0 })
        .insert_resource(Session {
            frames: 0,
            stop_after: frames,
            screenshot,
            captured: false,
            home,
        })
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.055)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Prototype 2 Rust — Xbox: sticks move/look; A/B rise/descend; LB fast; X collision; Y reset; Menu exit".into(),
                resolution: (1600, 900).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_controls, capture_and_exit))
        .run();
}
fn mesh(data: &MeshData) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions.clone());
    mesh.insert_indices(Indices::U32(data.indices.clone()));
    mesh.compute_normals();
    mesh
}
fn setup(
    mut commands: Commands,
    world: Res<WorldData>,
    session: Res<Session>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let center = (Vec3::from_array(world.0.summary.bounds[0])
        + Vec3::from_array(world.0.summary.bounds[1]))
        * 0.5;
    for (i, data) in world.0.meshes.iter().enumerate() {
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(
                0.35 + (i % 4) as f32 * 0.09,
                0.47 + (i % 3) as f32 * 0.08,
                0.55 + (i % 2) as f32 * 0.08,
            ),
            perceptual_roughness: 0.85,
            unlit: true,
            cull_mode: None,
            ..default()
        });
        commands.spawn((
            Name::new(data.name.clone()),
            Mesh3d(meshes.add(mesh(data))),
            MeshMaterial3d(material),
            Transform::default(),
        ));
    }
    for c in &world.0.collision {
        let data = MeshData {
            name: "ground collision — tags unknown".into(),
            positions: c.positions.clone(),
            indices: c
                .faces
                .iter()
                .flat_map(|f| f[..3].iter().map(|&i| i as u32))
                .collect(),
        };
        let material = materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.25, 0.07, 0.55),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            depth_bias: 2.0,
            ..default()
        });
        commands.spawn((
            CollisionOverlay,
            Mesh3d(meshes.add(mesh(&data))),
            MeshMaterial3d(material),
            Transform::default(),
            Visibility::Hidden,
        ));
    }
    commands.spawn((
        DirectionalLight {
            illuminance: 18000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(center + Vec3::new(100.0, 180.0, 80.0))
            .looking_at(center, Vec3::Y),
    ));
    commands.spawn((Camera3d::default(), Exposure::SUNLIGHT, session.home));
    println!(
        "VIEWER_READY: {} world meshes, {} ground collision triangles. Xbox: left stick move, right stick look, A/B up/down, LB fast, X collision, Y reset, Menu exit. Keyboard: WASD/QE + arrows, Shift fast, C collision, R reset.",
        world.0.summary.world_meshes, world.0.summary.ground_collision_triangles
    );
}
fn camera_controls(
    input: InputDevices,
    time: Res<Time>,
    session: Res<Session>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
    mut overlays: Query<&mut Visibility, With<CollisionOverlay>>,
    mut exit: MessageWriter<AppExit>,
) {
    let InputDevices {
        keys,
        mut controller,
        window,
    } = input;
    let focused = window.iter().next().is_some_and(|w| w.focused);
    let previous_slot = controller.tracker.slot();
    let settings = controller.settings;
    // Poll the active pad per frame; probe disconnected slots at most once/second.
    let mut pads = [None; 4];
    let active = controller.tracker.slot();
    if let Some(slot) = active {
        pads[slot] = controller::poll(slot as u32);
    }
    if active.is_none_or(|slot| pads[slot].is_none())
        && (active.is_some() || time.elapsed_secs_f64() >= controller.next_probe)
    {
        pads = controller::connected();
        controller.next_probe = time.elapsed_secs_f64() + 1.0;
    }
    let pad = controller.tracker.sample(pads, focused, settings);
    if controller.tracker.slot() != previous_slot {
        match controller.tracker.slot() {
            Some(slot) => println!(
                "CONTROLLER_CONNECTED: Xbox/XInput slot {slot}; dead zones {:.3}/{:.3}; look {:.2} rad/s; invert Y {}",
                settings.left_dead_zone,
                settings.right_dead_zone,
                settings.look_speed,
                settings.invert_y
            ),
            None => {
                println!("CONTROLLER_DISCONNECTED: input cleared; reconnect an Xbox controller")
            }
        }
    }
    if !focused {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) || pad.exit {
        exit.write(AppExit::Success);
    }
    if keys.just_pressed(KeyCode::KeyC) || pad.toggle_collision {
        for mut v in &mut overlays {
            *v = if *v == Visibility::Hidden {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
    for mut t in &mut camera {
        if keys.just_pressed(KeyCode::KeyR) || pad.reset {
            *t = session.home;
        }
        let axis = |p, n| f32::from(keys.pressed(p)) - f32::from(keys.pressed(n));
        let dt = time.delta_secs().min(0.1);
        let (mut yaw, mut pitch, _) = t.rotation.to_euler(EulerRot::YXZ);
        yaw += (axis(KeyCode::ArrowLeft, KeyCode::ArrowRight) + pad.look[0]) * dt;
        pitch = (pitch + (axis(KeyCode::ArrowUp, KeyCode::ArrowDown) + pad.look[1]) * dt)
            .clamp(-1.48, 1.48);
        t.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
        let direction = t.forward() * (axis(KeyCode::KeyW, KeyCode::KeyS) + pad.travel[1])
            + t.right() * (axis(KeyCode::KeyD, KeyCode::KeyA) + pad.travel[0])
            + Vec3::Y * (axis(KeyCode::KeyE, KeyCode::KeyQ) + pad.travel[2]);
        let speed = if keys.pressed(KeyCode::ShiftLeft) || pad.fast {
            100.0
        } else {
            25.0
        };
        t.translation += direction.clamp_length_max(1.0) * dt * speed;
    }
}
fn capture_and_exit(
    mut commands: Commands,
    mut session: ResMut<Session>,
    mut exit: MessageWriter<AppExit>,
) {
    session.frames += 1;
    if session.frames >= 60
        && !session.captured
        && let Some(path) = session.screenshot.clone()
    {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
        session.captured = true;
    }
    if session.stop_after.is_some_and(|n| {
        session.frames
            >= n.max(if session.screenshot.is_some() {
                120
            } else {
                30
            })
    }) {
        exit.write(AppExit::Success);
    }
}
