use bevy::{
    app::AppExit,
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use prototype2_rust::{
    animation::{Clip, Skeleton},
    controller,
    skin::SkinMesh,
};
use std::path::PathBuf;

#[derive(Resource)]
struct Character {
    skeletons: Vec<Skeleton>,
    skins: Vec<SkinMesh>,
    clips: Vec<Clip>,
}
#[derive(Resource)]
struct Playback {
    frame: f32,
    clip_index: usize,
    fixed: bool,
    paused: bool,
    renders: u32,
    stop_after: Option<u32>,
    screenshot: Option<PathBuf>,
    captured: bool,
    yaw: f32,
    pitch: f32,
    distance: f32,
    tracker: controller::Tracker,
    settings: controller::Settings,
}
#[derive(Component)]
struct SkinPart(usize);
pub fn run(
    skeletons: Vec<Skeleton>,
    skins: Vec<SkinMesh>,
    clips: Vec<Clip>,
    stop_after: Option<u32>,
    screenshot: Option<PathBuf>,
    sample_frame: Option<f32>,
    settings: controller::Settings,
) {
    let clip = &clips[0];
    println!(
        "ANIMATION_LOADED: {} at {} fps, end frame {}; {} skins. Inspection repeats the clip in place; extracted Motion_Root is not applied as gameplay movement.",
        clip.info.name,
        clip.info.frames_per_second,
        clip.info.end_frame,
        skins.len()
    );
    App::new().insert_resource(Character { skeletons,skins,clips })
        .insert_resource(Playback { frame: sample_frame.unwrap_or(0.),clip_index: 0,fixed: sample_frame.is_some(),paused: false,renders: 0,stop_after,screenshot,captured: false,yaw: 0.4,pitch: 0.1,distance: 3.5,tracker: controller::Tracker::default(),settings })
        .insert_resource(ClearColor(Color::srgb(0.035,0.045,0.065)))
        .add_plugins(DefaultPlugins.set(WindowPlugin { primary_window: Some(Window { title: "Prototype 2 Rust | Heller animation | A/B clips; sticks orbit/zoom; X pause; Y restart; LB slow; Menu exit".into(),resolution: (1280,900).into(),..default() }),..default() }))
        .add_systems(Startup,setup).add_systems(Update,play).run();
}
fn setup(
    mut commands: Commands,
    character: Res<Character>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (i, skin) in character.skins.iter().enumerate() {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, skin.positions.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, skin.normals.clone());
        mesh.insert_indices(Indices::U32(skin.indices.clone()));
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.5 + (i % 3) as f32 * 0.06, 0.55, 0.6),
            perceptual_roughness: 0.8,
            cull_mode: None,
            ..default()
        });
        commands.spawn((
            SkinPart(i),
            Name::new(skin.name.clone()),
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material),
            Transform::default(),
        ));
    }
    commands.spawn((
        DirectionalLight {
            illuminance: 18000.,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(3., 6., 4.).looking_at(Vec3::Y, Vec3::Y),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 1.5, 3.5).looking_at(Vec3::Y, Vec3::Y),
    ));
    println!(
        "ANIMATION_READY: original Heller skin and clip; inspection rendering/materials, no retail state machine"
    );
}
#[allow(clippy::too_many_arguments)]
fn play(
    mut commands: Commands,
    character: Res<Character>,
    mut state: ResMut<Playback>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut window: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
    parts: Query<(&SkinPart, &Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
    mut exit: MessageWriter<AppExit>,
) {
    let focused = window.iter().next().is_some_and(|w| w.focused);
    let settings = state.settings;
    let previous_slot = state.tracker.slot();
    let pad = state
        .tracker
        .sample(controller::connected(), focused, settings);
    if state.tracker.slot() != previous_slot {
        println!("ANIMATION_CONTROLLER: {:?}", state.tracker.slot());
    }
    if focused {
        if pad.exit || keys.just_pressed(KeyCode::Escape) {
            exit.write(AppExit::Success);
        }
        if pad.toggle_collision || keys.just_pressed(KeyCode::Space) {
            state.paused = !state.paused;
        }
        if pad.reset || keys.just_pressed(KeyCode::KeyR) {
            state.frame = 0.;
        }
        if !state.fixed && (pad.next_clip || keys.just_pressed(KeyCode::ArrowRight)) {
            state.clip_index = (state.clip_index + 1) % character.clips.len();
            state.frame = 0.;
        }
        if !state.fixed && (pad.previous_clip || keys.just_pressed(KeyCode::ArrowLeft)) {
            state.clip_index =
                (state.clip_index + character.clips.len() - 1) % character.clips.len();
            state.frame = 0.;
        }
        let dt = time.delta_secs().min(0.1);
        state.yaw += (pad.look[0] - pad.travel[0]) * dt;
        state.pitch = (state.pitch + pad.look[1] * dt).clamp(-0.7, 0.8);
        state.distance = (state.distance - pad.travel[1] * dt * 2.).clamp(1.5, 8.);
        if !state.fixed && !state.paused {
            state.frame = (state.frame
                + dt * character.clips[state.clip_index].info.frames_per_second
                    * if pad.fast { 0.2 } else { 1. })
            .rem_euclid(character.clips[state.clip_index].info.end_frame);
        }
    }
    // CPU inspection removes the animation's Motion_Root transform and retains
    // the bind root. The original root-motion driver/actor integration is separate.
    let worlds: anyhow::Result<Vec<_>> = character
        .skeletons
        .iter()
        .map(|s| {
            let mut world = character.clips[state.clip_index].sample(s, state.frame)?;
            let correction = s.bind_world[0] * world[0].inverse();
            for m in &mut world {
                *m = correction * *m;
            }
            Ok(world)
        })
        .collect();
    match worlds {
        Ok(worlds) => {
            for (part, handle) in &parts {
                let skin = &character.skins[part.0];
                let index = character
                    .skeletons
                    .iter()
                    .position(|s| s.name == skin.skeleton)
                    .expect("validated skeleton reference");
                match skin.deform(&character.skeletons[index], &worlds[index]) {
                    Ok(deformed) => {
                        if let Some(mesh) = meshes.get_mut(&handle.0) {
                            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, deformed.positions);
                            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, deformed.normals);
                        }
                    }
                    Err(e) => {
                        eprintln!("animation skin failed: {e:#}");
                        exit.write(AppExit::Error(std::num::NonZeroU8::new(1).unwrap()));
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("animation sample failed: {e:#}");
            exit.write(AppExit::Error(std::num::NonZeroU8::new(1).unwrap()));
        }
    }
    for mut t in &mut camera {
        let r = state.distance * state.pitch.cos();
        t.translation = Vec3::new(
            state.yaw.sin() * r,
            1. + state.distance * state.pitch.sin(),
            state.yaw.cos() * r,
        );
        t.look_at(Vec3::Y, Vec3::Y);
    }
    state.renders += 1;
    for mut window in &mut window {
        window.title = format!(
            "Prototype 2 Rust | {} | frame {:.1}{} | A/B clips; sticks orbit/zoom; X pause; Y restart; LB slow; Menu exit",
            character.clips[state.clip_index].info.name,
            state.frame,
            if state.paused { " paused" } else { "" }
        );
    }
    if state.renders >= 60
        && !state.captured
        && let Some(path) = state.screenshot.clone()
    {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
        state.captured = true;
        println!("ANIMATION_CAPTURE: frame {:.6}", state.frame);
    }
    if state
        .stop_after
        .is_some_and(|n| state.renders >= n.max(if state.screenshot.is_some() { 120 } else { 30 }))
    {
        exit.write(AppExit::Success);
    }
}
