use anyhow::{Context, Result, bail, ensure};
use prototype2_rust::{install, p3d, rcf::Archive, scene};
use std::{collections::BTreeMap, path::PathBuf};
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
                "--direction"
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
        "verify" => &["--game", "--archive"],
        "scan" => &["--game", "--archive"],
        "list" => &["--game", "--archive", "--filter", "--limit"],
        "chunks" => &["--game", "--archive", "--entry", "--limit"],
        "scene" => &["--game", "--archive", "--entry"],
        "ray" => &["--game", "--archive", "--entry", "--origin", "--direction"],
        "view" => &["--game", "--archive", "--entry", "--frames", "--screenshot"],
        _ => bail!("unknown command {command}; use --help"),
    };
    for key in options.keys() {
        ensure!(
            allowed.contains(&key.as_str()),
            "{key} is not valid for {command}"
        );
    }
    let game = options
        .get("--game")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("PROTOTYPE2_GAME").map(PathBuf::from))
        .context("supply --game <installed Prototype 2 directory> or PROTOTYPE2_GAME")?;
    ensure!(game.is_dir(), "game directory does not exist");
    if command == "inspect" {
        println!(
            "{}",
            serde_json::to_string_pretty(&install::fingerprint(&game)?)?
        );
        return Ok(());
    }
    let archive = options
        .get("--archive")
        .map(String::as_str)
        .unwrap_or("cells.rcf");
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
    let entry = a.find(
        options
            .get("--entry")
            .map(String::as_str)
            .unwrap_or(DEFAULT_ENTRY),
    )?;
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
    let world = scene::load(&data, &chunks)?;
    println!("{}", serde_json::to_string_pretty(&world.summary)?);
    if command == "scene" {
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
        );
        Ok(())
    }
    #[cfg(not(feature = "viewer"))]
    bail!("viewer disabled; build with default features")
}
fn help() {
    println!(
        "prototype2-rust: experimental retail-data viewer, NOT a playable reimplementation\n\nCommands: inspect | verify | scan | list | chunks | scene | ray | view\nRequired: --game <Prototype 2 install directory> (or PROTOTYPE2_GAME)\nSelection: --archive cells.rcf --entry <internal path>\nlist: --filter <substring> --limit 40\nchunks: --limit 40\nray: --origin x,y,z --direction x,y,z\nview: --frames <at least 30> --screenshot <PNG path outside repository>\n\nDefault section: yellow_zone/Cell_29. Viewer: WASD, Q/E, arrows, Shift; C collision overlay; R reset; Escape exit.\nUnknown collision tags and retail character movement remain unimplemented."
    );
}
