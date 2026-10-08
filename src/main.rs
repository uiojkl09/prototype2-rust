use anyhow::{Context, Result, bail, ensure};
use prototype2_rust::{
    animation, capsule, collision, controller, fight, install, meta, p3d, rcf::Archive, scene,
};
use std::{collections::BTreeMap, path::PathBuf};
#[cfg(feature = "viewer")]
mod animation_viewer;
#[cfg(feature = "viewer")]
mod viewer;

const DEFAULT_ENTRY: &str = "art\\locations\\yellow_zone\\Cell_29.p3d.rz";
fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        help();
        return Ok(());
    };
    if command == "--help" || command == "help" {
        help();
        return Ok(());
    }
    if command == "--version" {
        println!(
            "prototype2-rust {} (experimental data/viewer milestone)",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }
    let mut options = BTreeMap::new();
    while let Some(key) = args.next() {
        ensure!(
            [
                "--game",
                "--archive",
                "--entry",
                "--filter",
                "--limit",
                "--frames",
                "--screenshot",
                "--origin",
                "--direction",
                "--delta",
                "--seconds",
                "--dead-zone",
                "--look-speed",
                "--invert-y",
                "--clip",
                "--sample-frame",
                "--loco-speed",
                "--clip-start",
                "--clip-end",
                "--clip-speed",
                "--clip-duration",
                "--clip-loop",
                "--previous-frame",
                "--current-frame",
                "--relative-root",
                "--reverse"
            ]
            .contains(&key.as_str()),
            "unknown option {key}"
        );
        let value = args
            .next()
            .with_context(|| format!("missing value for {key}"))?;
        ensure!(
            options.insert(key.clone(), value).is_none(),
            "duplicate option {key}"
        );
    }
    let allowed: &[&str] = match command.as_str() {
        "inspect" => &["--game"],
        "capsule" => &["--game"],
        "verify" => &["--game", "--archive"],
        "scan" => &["--game", "--archive"],
        "list" => &["--game", "--archive", "--filter", "--limit"],
        "chunks" => &["--game", "--archive", "--entry", "--limit"],
        "meta" => &["--game", "--archive", "--entry", "--filter", "--limit"],
        "fight" => &["--game", "--archive", "--entry", "--filter", "--limit"],
        "animations" => &["--game", "--filter", "--limit"],
        "root-motion" => &[
            "--game",
            "--clip",
            "--previous-frame",
            "--current-frame",
            "--relative-root",
            "--reverse",
        ],
        "animate" => &[
            "--game",
            "--clip",
            "--sample-frame",
            "--loco-speed",
            "--clip-start",
            "--clip-end",
            "--clip-speed",
            "--clip-duration",
            "--clip-loop",
            "--frames",
            "--screenshot",
            "--dead-zone",
            "--look-speed",
            "--invert-y",
        ],
        "scene" => &["--game", "--archive", "--entry"],
        "ray" => &["--game", "--archive", "--entry", "--origin", "--direction"],
        "sweep" => &["--game", "--archive", "--entry", "--origin", "--delta"],
        "view" => &[
            "--game",
            "--archive",
            "--entry",
            "--frames",
            "--screenshot",
            "--dead-zone",
            "--look-speed",
            "--invert-y",
        ],
        "controller" => &["--seconds", "--dead-zone", "--look-speed", "--invert-y"],
        _ => bail!("unknown command {command}; use --help"),
    };
    for key in options.keys() {
        ensure!(
            allowed.contains(&key.as_str()),
            "{key} is not valid for {command}"
        );
    }
    let mut settings = controller::Settings::default();
    if let Some(value) = options.get("--dead-zone") {
        settings.left_dead_zone = value.parse()?;
        settings.right_dead_zone = settings.left_dead_zone;
    }
    if let Some(value) = options.get("--look-speed") {
        settings.look_speed = value.parse()?;
    }
    if let Some(value) = options.get("--invert-y") {
        settings.invert_y = value.parse()?;
    }
    ensure!(
        settings.valid(),
        "dead zone must be 0..0.9; look speed must be 0.1..10 radians/second; values must be finite"
    );
    if command == "controller" {
        ensure!(
            cfg!(target_os = "windows"),
            "native XInput is supported on Windows only"
        );
        let seconds: u32 = options
            .get("--seconds")
            .map(|s| s.parse())
            .transpose()?
            .unwrap_or(0);
        ensure!(seconds <= 60, "--seconds must be 0..60");
        let started = std::time::Instant::now();
        let mut tracker = controller::Tracker::default();
        println!(
            "Xbox/XInput diagnostics; slots 0..3; controls below are inspection controls, not recovered gameplay"
        );
        loop {
            let slots = controller::connected();
            let controls = tracker.sample(slots, true, settings);
            println!(
                "{}",
                serde_json::json!({"elapsed_seconds": started.elapsed().as_secs_f32(), "selected_slot": tracker.slot(), "raw_slots": slots, "controls": controls})
            );
            if started.elapsed().as_secs_f32() >= seconds as f32 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        return Ok(());
    }
    let game = options
        .get("--game")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("PROTOTYPE2_GAME").map(PathBuf::from))
        .context("supply --game <installed Prototype 2 directory> or PROTOTYPE2_GAME")?;
    ensure!(game.is_dir(), "game directory does not exist");
    if command == "animations" || command == "animate" || command == "root-motion" {
        let boot = Archive::open(game.join("boot.rcf"))?;
        let data = p3d::decode(&boot.read(boot.find("art\\alex\\alex.p3d")?)?)?;
        let chunks = p3d::parse(&data)?;
        if command == "animations" {
            let all = animation::clips(&data, &chunks)?;
            let filter = options
                .get("--filter")
                .map(String::as_str)
                .unwrap_or("heller");
            let limit: usize = options
                .get("--limit")
                .map(|s| s.parse())
                .transpose()?
                .unwrap_or(40);
            ensure!(limit <= 1000, "--limit must be at most 1000");
            for clip in all.iter().filter(|c| c.name.contains(filter)).take(limit) {
                println!("{}", serde_json::to_string(clip)?);
            }
            println!(
                "{} total clip headers; listing does not imply every encoding is playable",
                all.len()
            );
            return Ok(());
        }
        if command == "root-motion" {
            let name = options
                .get("--clip")
                .map(String::as_str)
                .unwrap_or("heller_loco_run_n");
            let clip = animation::load_clip(&data, &chunks, name)?;
            let track = clip
                .tracks
                .get("Motion_Root")
                .context("clip has no inspected Motion_Root track")?;
            let previous = options
                .get("--previous-frame")
                .map(|s| s.parse())
                .transpose()?
                .unwrap_or(0.);
            let current = options
                .get("--current-frame")
                .map(|s| s.parse())
                .transpose()?
                .unwrap_or(clip.info.last_frame());
            let relative_translation = options
                .get("--relative-root")
                .map(|s| s.parse())
                .transpose()?
                .unwrap_or(false);
            let reverse = options
                .get("--reverse")
                .map(|s| s.parse())
                .transpose()?
                .unwrap_or(false);
            let interval = prototype2_rust::root_motion::Interval {
                previous,
                current,
                first: 0.,
                last: clip.info.last_frame(),
                relative_translation,
                reverse,
            };
            let motion = prototype2_rust::root_motion::delta(Some(track), interval)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "clip": clip.info, "joint": "Motion_Root", "previous_frame": previous,"current_frame": current,
                    "relative_translation": relative_translation,"reverse": reverse,
                    "translation": motion.translation.to_array(), "rotation_xyzw": motion.rotation.to_array(),
                    "scope": "Authored single-clip root delta with at most one inferred cycle crossing; not actor movement, contacts or state scheduling"
                }))?
            );
            return Ok(());
        }
        #[cfg(feature = "viewer")]
        {
            let clip_name = options
                .get("--clip")
                .map(String::as_str)
                .unwrap_or("heller_loco_run_n");
            let clip = animation::load_clip(&data, &chunks, clip_name)?;
            let mut clips = vec![clip];
            let fig = p3d::decode(&boot.read(boot.find("art\\alex\\alex_fig.p3d")?)?)?;
            let graphs = fight::load(&fig, &p3d::parse(&fig)?)?;
            let graph = graphs
                .iter()
                .find(|g| g.name == "prototype")
                .context("missing main character graph")?;
            let mut main_tracks = graph
                .records
                .iter()
                .filter(|r| r.steer.is_some() && graph.branch_matches(r.branch, "bare/loco"));
            let record = main_tracks
                .next()
                .context("missing main bare locomotion track")?;
            ensure!(
                main_tracks.next().is_none(),
                "ambiguous main bare locomotion track"
            );
            let infos = animation::clips(&data, &chunks)?;
            let track = record.steer.as_ref().unwrap();
            let names = track
                .animations_idle_walk_run
                .iter()
                .map(|hash| animation::resolve_clip(&infos, *hash).map(|c| c.name.as_str()))
                .collect::<Result<Vec<_>>>()?;
            println!(
                "ANIMATION_REFERENCES: main LocoSteer 0x{:x}: idle/walk/run {:?}; state scheduling is not executed",
                record.offset, names
            );
            for name in names.iter().copied().chain([
                "heller_loco_run_sprint_n",
                "heller_loco_jump_from_idle",
                "heller_bare_punch_1",
                "heller_bare_kick_1",
            ]) {
                if name != clip_name {
                    clips.push(animation::load_clip(&data, &chunks, name)?);
                }
            }
            let skeletons = ["alex_reg_body_skeleton", "alex_reg_arms_skeleton"]
                .iter()
                .map(|name| animation::skeleton(&data, &chunks, name))
                .collect::<Result<Vec<_>>>()?;
            let art = Archive::open(game.join("art.rcf"))?;
            let model = p3d::decode(&art.read(art.find("art\\alex\\alex_model_main.p3d")?)?)?;
            let meshes = prototype2_rust::skin::load(&model, &p3d::parse(&model)?, &skeletons)?;
            let tod = p3d::decode(&boot.read(boot.find("art\\alex\\alex_tod.p3d")?)?)?;
            let fixups = prototype2_rust::pose_fixup::load(&tod, &p3d::parse(&tod)?)?;
            let bound_fixups = skeletons
                .iter()
                .map(|s| fixups.bind(s))
                .collect::<Result<Vec<_>>>()?;
            println!(
                "ANIMATION_FIXUPS: Heller configuration at 0x{:x}; collar and shoulder corrective joints, before skinning",
                fixups.definition_offset
            );
            let frames: Option<u32> = options.get("--frames").map(|s| s.parse()).transpose()?;
            ensure!(
                frames.is_none_or(|n| n >= 30),
                "--frames must be at least 30"
            );
            let sample_frame: Option<f32> = options
                .get("--sample-frame")
                .map(|s| s.parse())
                .transpose()?;
            let locomotion_speed: Option<f32> =
                options.get("--loco-speed").map(|s| s.parse()).transpose()?;
            ensure!(
                locomotion_speed
                    .is_none_or(|speed| speed.is_finite() && (0. ..=1000.).contains(&speed)),
                "--loco-speed must be finite and in 0..1000"
            );
            ensure!(
                sample_frame.is_none() || locomotion_speed.is_none(),
                "--sample-frame and --loco-speed cannot be combined"
            );
            let custom_timing = [
                "--clip-start",
                "--clip-end",
                "--clip-speed",
                "--clip-duration",
                "--clip-loop",
            ]
            .iter()
            .any(|key| options.contains_key(*key));
            ensure!(
                !custom_timing || locomotion_speed.is_none(),
                "clip timing options and --loco-speed cannot be combined"
            );
            let clip_loop = options
                .get("--clip-loop")
                .map(|s| s.parse())
                .transpose()?
                .unwrap_or(true);
            let clip_timing = if custom_timing {
                let value = |key: &str, default: f32| -> Result<f32> {
                    Ok(options
                        .get(key)
                        .map(|s| s.parse())
                        .transpose()?
                        .unwrap_or(default))
                };
                Some(prototype2_rust::animation_clock::ClipTiming::configure(
                    &clips[0].info,
                    value("--clip-start", 0.)?,
                    value("--clip-end", -1.)?,
                    value("--clip-duration", -1.)?,
                    value("--clip-speed", 1.)?,
                    false,
                )?)
            } else {
                None
            };
            let indices: [usize; 3] = names
                .iter()
                .map(|name| {
                    clips
                        .iter()
                        .position(|c| c.info.name == *name)
                        .context("locomotion clip missing from bank")
                })
                .collect::<Result<Vec<_>>>()?
                .try_into()
                .unwrap();
            ensure!(
                track
                    .sync_frames_idle_walk_run
                    .iter()
                    .all(|frame| *frame < 0.),
                "explicit LocoSteer sync overrides are not supported in this preview"
            );
            let sync_frames = indices.map(|i| clips[i].info.default_sync_frame);
            ensure!(
                sample_frame
                    .is_none_or(|n| n.is_finite() && n >= 0. && n <= clips[0].info.last_frame()),
                "--sample-frame out of clip range"
            );
            ensure!(
                sample_frame.is_none_or(|frame| clip_timing
                    .is_none_or(|t| frame >= t.first_frame() && frame <= t.last_frame())),
                "--sample-frame out of cropped clip range"
            );
            animation_viewer::run(
                skeletons,
                meshes,
                clips,
                bound_fixups,
                animation_viewer::LocomotionPreview {
                    indices,
                    velocities: [0., track.velocity_walk, track.velocity_run],
                    sync_frames,
                },
                animation_viewer::Inspection {
                    stop_after: frames,
                    screenshot: options.get("--screenshot").map(PathBuf::from),
                    sample_frame,
                    locomotion_speed,
                    clip_timing,
                    clip_loop,
                },
                settings,
            )?;
            return Ok(());
        }
        #[cfg(not(feature = "viewer"))]
        bail!("animation viewer disabled; build with default features");
    }
    if command == "inspect" {
        println!(
            "{}",
            serde_json::to_string_pretty(&install::fingerprint(&game)?)?
        );
        return Ok(());
    }
    if command == "capsule" {
        println!(
            "{}",
            serde_json::to_string_pretty(&character_factory(&game)?)?
        );
        println!("Recovered asset factory only; locomotion-state overrides remain unknown.");
        return Ok(());
    }
    let archive = options
        .get("--archive")
        .map(String::as_str)
        .unwrap_or(if command == "fight" {
            "boot.rcf"
        } else {
            "cells.rcf"
        });
    ensure!(
        archive.ends_with(".rcf") && !archive.contains(['/', '\\', ':']) && archive != "..",
        "--archive must be a filename within the install"
    );
    if command == "verify" {
        let mut paths: Vec<_> = if options.contains_key("--archive") {
            vec![game.join(archive)]
        } else {
            std::fs::read_dir(&game)?
                .filter_map(|r| r.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|e| e == "rcf"))
                .collect()
        };
        paths.sort();
        ensure!(!paths.is_empty(), "no RCF archives found");
        let mut total = 0;
        for path in paths {
            let a = Archive::open(&path)?;
            total += a.entries.len();
            println!(
                "{}: {} entries; names, hashes, ranges and alignment validated",
                path.file_name().unwrap().to_string_lossy(),
                a.entries.len()
            );
        }
        println!("PASS: {total} entries (payload decoding is tested separately)");
        return Ok(());
    }
    let a = Archive::open(game.join(archive))?;
    if command == "scan" {
        let mut files = 0usize;
        let mut stored = 0u64;
        let mut decoded = 0u64;
        let mut nodes = 0usize;
        let mut histogram = BTreeMap::<String, usize>::new();
        let started = std::time::Instant::now();
        for e in &a.entries {
            let name = e.name.to_ascii_lowercase();
            if !name.ends_with(".p3d") && !name.ends_with(".p3d.rz") {
                continue;
            }
            let data = p3d::decode(&a.read(e)?).with_context(|| format!("decode {}", e.name))?;
            let chunks = p3d::parse(&data).with_context(|| format!("parse {}", e.name))?;
            files += 1;
            stored += e.size;
            decoded += data.len() as u64;
            nodes += chunks.len();
            for (id, n) in p3d::histogram(&chunks) {
                *histogram.entry(id).or_default() += n;
            }
            if files.is_multiple_of(250) {
                eprintln!("scan: {files} files validated");
            }
        }
        ensure!(files > 0, "no Pure3D entries found");
        println!(
            "PASS: {files} P3D entries; {stored} stored bytes; {decoded} decoded bytes; {nodes} chunks; {:.3}s",
            started.elapsed().as_secs_f64()
        );
        println!("{}", serde_json::to_string_pretty(&histogram)?);
        return Ok(());
    }
    let limit: usize = options
        .get("--limit")
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(40);
    ensure!(limit <= 100000, "limit exceeds 100000");
    if command == "list" {
        let filter = options
            .get("--filter")
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        let matches: Vec<_> = a
            .entries
            .iter()
            .filter(|e| e.name.to_ascii_lowercase().contains(&filter))
            .collect();
        for e in matches.iter().take(limit) {
            println!("{:10}  {:10}  {}", e.size, e.offset, e.name);
        }
        println!(
            "{} matches; showing {}",
            matches.len(),
            matches.len().min(limit)
        );
        return Ok(());
    }
    let entry = a.find(options.get("--entry").map(String::as_str).unwrap_or(
        if command == "fight" {
            "art\\alex\\alex_fig.p3d"
        } else {
            DEFAULT_ENTRY
        },
    ))?;
    let raw = a.read(entry)?;
    let data = p3d::decode(&raw)?;
    let chunks = p3d::parse(&data)?;
    println!(
        "{} / {}: {} stored -> {} decoded bytes; {} chunks",
        archive,
        entry.name,
        raw.len(),
        data.len(),
        chunks.len()
    );
    if command == "chunks" {
        for c in chunks.iter().take(limit) {
            println!(
                "depth {:3}  offset 0x{:08x}  id 0x{:08x}  data {:9}  total {:9}",
                c.depth, c.offset, c.id, c.data_size, c.total_size
            );
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&p3d::histogram(&chunks))?
        );
        return Ok(());
    }
    if command == "meta" {
        let objects = meta::inspect(
            &data,
            &chunks,
            options.get("--filter").map(String::as_str).unwrap_or(""),
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&objects.iter().take(limit).collect::<Vec<_>>())?
        );
        println!(
            "{} matched metadata envelopes; showing {}. Bodies remain opaque; no property values or gameplay rules inferred.",
            objects.len(),
            objects.len().min(limit)
        );
        return Ok(());
    }
    if command == "fight" {
        let graphs = fight::load(&data, &chunks)?;
        let filter = options.get("--filter").map(String::as_str).unwrap_or("");
        for graph in &graphs {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "context": graph.name, "data_offset": graph.data_offset,
                    "branches": graph.branches.len(), "property_records": graph.records.len(),
                    "capsule_tracks": graph.records.iter().filter(|r| r.capsule.is_some()).count(),
                "steer_tracks": graph.records.iter().filter(|r| r.steer.is_some()).count(),
                "sprint_tracks": graph.records.iter().filter(|r| r.sprint.is_some()).count(),
                "animation_tracks": graph.records.iter().filter(|r| r.animation.is_some()).count(),
                }))?
            );
            let matched: Vec<_> = graph
                .records
                .iter()
                .filter(|r| {
                    (r.capsule.is_some()
                        || r.steer.is_some()
                        || r.sprint.is_some()
                        || r.animation.is_some())
                        && graph.branch_matches(r.branch, filter)
                })
                .collect();
            for record in matched.iter().take(limit) {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "branch": graph.branches[record.branch].derived_path, "record": record,
                        "animation_policies": record.animation.as_ref().map(|track| serde_json::json!({
                            "cycle": prototype2_rust::animation_policy::CyclePolicy::from_hash(track.cyclic)
                                .map(|p| p.label()).unwrap_or("unresolved"),
                            "sync_phase": prototype2_rust::animation_policy::SyncPhasePolicy::from_hash(track.sync_phase)
                                .map(|p| p.label()).unwrap_or("unresolved"),
                        })),
                    }))?
                );
            }
            println!(
                "{} matched decoded tracks; showing {}",
                matched.len(),
                matched.len().min(limit)
            );
        }
        println!(
            "Graph and selected asset tracks only; conditions and other actions remain opaque."
        );
        return Ok(());
    }
    let world = scene::load(&data, &chunks)?;
    println!("{}", serde_json::to_string_pretty(&world.summary)?);
    if command == "scene" {
        return Ok(());
    }
    if command == "sweep" {
        let vector = |key: &str| -> Result<[f64; 3]> {
            let values: Vec<f64> = options
                .get(key)
                .with_context(|| format!("{key} x,y,z is required"))?
                .split(',')
                .map(str::parse)
                .collect::<std::result::Result<_, _>>()?;
            ensure!(
                values.len() == 3 && values.iter().all(|x| x.is_finite()),
                "invalid {key}"
            );
            Ok([values[0], values[1], values[2]])
        };
        let factory = character_factory(&game)?;
        let origin = vector("--origin")?;
        let delta = vector("--delta")?;
        let hit = collision::sweep(&world.collision, &factory.shape, origin, delta, 0.)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "factory": factory, "origin": origin, "delta": delta, "hit": hit,
                "query": "Two-sided geometric capsule translation; no retail filtering or movement response"
            }))?
        );
        return Ok(());
    }
    if command == "ray" {
        let vec3 = |key: &str| -> Result<[f32; 3]> {
            let s = options
                .get(key)
                .with_context(|| format!("{key} x,y,z is required"))?;
            let v = s
                .split(',')
                .map(str::parse)
                .collect::<std::result::Result<Vec<f32>, _>>()?;
            ensure!(
                v.len() == 3 && v.iter().all(|f| f.is_finite()),
                "invalid {key}"
            );
            Ok([v[0], v[1], v[2]])
        };
        let origin = vec3("--origin")?;
        let mut dir = vec3("--direction")?;
        let len = (dir.iter().map(|x| x * x).sum::<f32>()).sqrt();
        ensure!(
            len.is_finite() && len > 1e-6,
            "invalid ray direction magnitude"
        );
        for x in &mut dir {
            *x /= len;
        }
        match scene::raycast(&world.collision, origin, dir) {
            Some(t) => println!(
                "Geometric hit distance {t:.6}; point {:.6},{:.6},{:.6}",
                origin[0] + dir[0] * t,
                origin[1] + dir[1] * t,
                origin[2] + dir[2] * t
            ),
            None => println!("No geometric intersection"),
        }
        return Ok(());
    }
    #[cfg(feature = "viewer")]
    {
        let frames = options.get("--frames").map(|s| s.parse()).transpose()?;
        ensure!(
            frames.is_none_or(|n: u32| n >= 30),
            "--frames must be at least 30"
        );
        viewer::run(
            world,
            frames,
            options.get("--screenshot").map(PathBuf::from),
            settings,
        );
        Ok(())
    }
    #[cfg(not(feature = "viewer"))]
    bail!("viewer disabled; build with default features")
}
fn character_factory(game: &std::path::Path) -> Result<capsule::Factory> {
    let boot = Archive::open(game.join("boot.rcf"))?;
    let raw = boot.read(boot.find("art\\alex\\alex_tod.p3d")?)?;
    let data = p3d::decode(&raw)?;
    capsule::load(&data, &p3d::parse(&data)?).context("read observed AlexPhysicsFactory")
}
fn help() {
    println!(
        "prototype2-rust: experimental retail-data viewer, NOT a playable reimplementation

Commands: inspect | verify | scan | list | chunks | meta | capsule | fight | animations | animate | root-motion | scene | ray | sweep | view | controller
Game commands require --game <Prototype 2 install directory> (or PROTOTYPE2_GAME)
Selection: --archive cells.rcf --entry <internal path>
list: --filter <substring> --limit 40
chunks: --limit 40
meta: --filter <ASCII name or body reference> --limit 40
fight: --filter <branch path substring> --limit 40 (defaults to boot.rcf / art\\alex\\alex_fig.p3d)
animations: --filter <clip substring> --limit 40 (character clip headers)
animate: --clip <name> --sample-frame <frame> --loco-speed <inspection speed> --frames <count> --screenshot <private PNG path>
  cropped inspection: --clip-start <frame> --clip-end <frame or negative count offset> --clip-speed <signed rate> --clip-duration <seconds> --clip-loop <true/false>
Animation inspection: A/B next/previous clip; sticks orbit/zoom; X pause; Y restart; LB slow; Menu exit.
Animation blend inspection: RB (keyboard L) toggles; left stick varies idle/walk/run speed; right stick orbits.
root-motion: --clip <name> --previous-frame <frame> --current-frame <frame> --relative-root true/false --reverse true/false
ray: --origin x,y,z --direction x,y,z
capsule: decode the observed AlexPhysicsFactory asset (boot.rcf)
sweep: --origin x,y,z --delta x,y,z (translation, not velocity; geometric query only)
view: --frames <at least 30> --screenshot <PNG path outside repository>
controller: --seconds <0..60> (no game installation needed)
view/controller preferences: --dead-zone <0..0.9> --look-speed <0.1..10 rad/s> --invert-y <true|false>

Xbox viewer: left stick move; right stick look; A/B up/down; LB fast; X collision; Y reset; Menu exit.
Keyboard: WASD, Q/E, arrows, Shift; C collision overlay; R reset; Escape exit.
Default section: yellow_zone/Cell_29. Unknown collision tags and retail character movement remain unimplemented."
    );
}
