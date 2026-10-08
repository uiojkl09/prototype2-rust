//! Bounded readers for the inspected Prototype 2 `fig0` character graph.
//! Conditions and most actions stay opaque; parsing a graph does not execute it.
use crate::{binary::Cursor, p3d::Chunk};
use anyhow::{Context, Result, bail, ensure};
use serde::Serialize;
use std::collections::BTreeMap;

pub const FIGHT_DEFINITION: u32 = 0x20000701;
pub const FIGHT_DATA: u32 = 0x20000702;
pub const MAX_BRANCHES: usize = 100_000;
pub const MAX_RECORDS: usize = 1_000_000;
pub const MAX_DEPTH: usize = 64;
const MAX_STRING: usize = 4096;

/// Case-preserving wrapping hash, corroborated by names in the inspected graphs.
pub const fn name_hash(name: &str) -> u64 {
    let bytes = name.as_bytes();
    let mut hash = 0u64;
    let mut i = 0;
    while i < bytes.len() {
        hash = hash.wrapping_mul(65599) ^ bytes[i] as u64;
        i += 1;
    }
    hash
}

#[derive(Clone, Debug, Serialize)]
pub struct BranchReference {
    pub name: String,
    pub index: i32,
}
#[derive(Clone, Debug, Serialize)]
pub struct Branch {
    pub offset: usize,
    pub kind: String,
    pub name_hash: u64,
    pub serialized_path: String,
    pub parent: Option<usize>,
    pub sibling_count: u32,
    pub reference: Option<BranchReference>,
    pub is_slave: Option<bool>,
    /// Components are resolved only by hashes of strings stored in this graph.
    pub derived_path: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct Record {
    pub offset: usize,
    pub body_offset: usize,
    pub body_bytes: usize,
    pub type_hash: u64,
    pub group: String,
    pub branch: usize,
    pub capsule: Option<CapsuleTrack>,
    pub steer: Option<SteerTrack>,
    pub sprint: Option<SprintTrack>,
    pub animation: Option<AnimationTrack>,
}
#[derive(Debug, Serialize)]
pub struct Graph {
    pub name: String,
    pub definition_offset: usize,
    pub data_offset: usize,
    pub context_name_hash: u64,
    pub context_type_hash: u64,
    pub root: BranchReference,
    pub declared_branches_including_root: u32,
    pub branches: Vec<Branch>,
    pub records: Vec<Record>,
}
impl Graph {
    /// Match a contiguous ancestry path by its hashes even when labels are absent.
    pub fn branch_matches(&self, index: usize, filter: &str) -> bool {
        let Some(branch) = self.branches.get(index) else {
            return false;
        };
        if filter.is_empty() || branch.derived_path.contains(filter) {
            return true;
        }
        let wanted: Vec<_> = filter
            .split('/')
            .filter(|s| !s.is_empty())
            .map(name_hash)
            .collect();
        if wanted.is_empty() {
            return true;
        }
        let mut chain = Vec::new();
        let mut next = Some(index);
        while let Some(index) = next {
            let Some(branch) = self.branches.get(index) else {
                return false;
            };
            chain.push(branch.name_hash);
            if branch.parent.is_some_and(|p| p >= index) {
                return false;
            }
            next = branch.parent;
        }
        chain.reverse();
        chain.windows(wanted.len()).any(|path| path == wanted)
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct TrackHeader {
    pub reference_index: i32,
    pub is_slave: bool,
    pub time_begin: f32,
    pub time_end: f32,
}
#[derive(Clone, Debug, Serialize)]
pub struct Animated<T> {
    pub enabled: bool,
    pub initial: T,
    pub final_value: T,
}
#[derive(Clone, Debug, Serialize)]
pub struct CapsuleTrack {
    pub header: TrackHeader,
    pub attach_to_joint: u64,
    pub animate: bool,
    pub offset: Animated<[f32; 3]>,
    pub extent: Animated<f32>,
    pub radius: Animated<f32>,
    pub axis: Animated<[f32; 3]>,
    pub rotation_degrees: Animated<[f32; 3]>,
}
#[derive(Clone, Debug, Serialize)]
pub struct AnimationTrack {
    pub header: TrackHeader,
    pub animation: u64,
    pub speed: f32,
    pub random_speed_variation: f32,
    pub init_frame: f32,
    pub start_frame: f32,
    pub end_frame: f32,
    /// Serialized enum-name hash; no scheduler policy is inferred from it.
    pub cyclic: u64,
    pub sync_frame: bool,
    pub phase_match: bool,
    /// Serialized enum-name hash; retained without executing synchronization.
    pub sync_phase: u64,
    pub sync_phase_min_frame: f32,
    pub sync_phase_max_frame: f32,
    pub reuse_existing_driver: bool,
    pub has_root_translation: bool,
    pub has_root_rotation: bool,
    pub blend_out_root_translation: bool,
    pub blend_out_root_rotation: bool,
    pub additive_joints: bool,
    pub partition: u64,
    pub weight: f32,
    pub priority: i32,
    pub blend_in: f32,
    pub blend_out: f32,
    pub synch_tracks_branch: BranchReference,
}
#[derive(Clone, Debug, Serialize)]
pub struct SteerTrack {
    pub header: TrackHeader,
    pub acceleration: f32,
    pub turning_velocity_degrees: f32,
    pub turning_velocity_run_degrees: f32,
    pub velocity_walk: f32,
    pub velocity_run: f32,
    pub smooth_steering_angle: f32,
    pub preserve_facing_joint: u64,
    pub preserve_facing_priority: i32,
    pub use_locomotion_phase: bool,
    pub phase: f32,
    pub locomotion: u64,
    pub animations_idle_walk_run: [u64; 3],
    pub sync_frames_idle_walk_run: [f32; 3],
    pub partition: u64,
    pub priority: i32,
    pub blend_in: f32,
    pub blend_out: f32,
    pub slave_branch: BranchReference,
}
#[derive(Clone, Debug, Serialize)]
pub struct SprintTrack {
    pub header: TrackHeader,
    pub sprint_button: u64,
    pub velocities_min_mid_max: [f32; 3],
    pub accelerations_min_mid_max: [f32; 3],
    pub deceleration: f32,
    pub turning_velocity_degrees_min_max: [f32; 2],
    pub turning_acceleration_degrees_min_max: [f32; 2],
    pub turn_deceleration: f32,
    pub lean_rate: f32,
    pub phase: f32,
    pub locomotion: u64,
    pub use_locomotion_phase: bool,
    pub animation_names_min_mid_max: [[u64; 3]; 3],
    pub force_animation_velocities: bool,
    pub unlockable_velocity_min: f32,
    pub unlockable_first_last: [u64; 2],
    pub partition: u64,
    pub priority: i32,
    pub blend_in: f32,
    pub blend_out: f32,
}

fn u64(c: &mut Cursor<'_>) -> Result<u64> {
    Ok(u64::from_le_bytes(c.take(8)?.try_into()?))
}
fn i32(c: &mut Cursor<'_>) -> Result<i32> {
    Ok(i32::from_le_bytes(c.take(4)?.try_into()?))
}
fn boolean(c: &mut Cursor<'_>) -> Result<bool> {
    match c.u32()? {
        0 => Ok(false),
        1 => Ok(true),
        value => bail!("invalid FIG boolean {value}"),
    }
}
fn string(c: &mut Cursor<'_>) -> Result<String> {
    let n = c.u32()? as usize;
    ensure!(n <= MAX_STRING, "FIG string exceeds limit");
    let bytes = c.take(n)?;
    ensure!(!bytes.contains(&0), "embedded null in FIG string");
    let value = std::str::from_utf8(bytes)
        .context("invalid FIG string")?
        .to_owned();
    ensure!(
        c.take((4 - n % 4) % 4)?.iter().all(|&b| b == 0),
        "nonzero FIG string padding"
    );
    Ok(value)
}
fn reference(c: &mut Cursor<'_>) -> Result<BranchReference> {
    let name = string(c)?;
    let index = i32(c)?;
    ensure!(index >= -1, "invalid FIG branch reference index");
    Ok(BranchReference { name, index })
}
fn header(c: &mut Cursor<'_>) -> Result<TrackHeader> {
    let reference_index = i32(c)?;
    ensure!(reference_index >= -1, "invalid FIG track reference index");
    Ok(TrackHeader {
        reference_index,
        is_slave: boolean(c)?,
        time_begin: c.f32()?,
        time_end: c.f32()?,
    })
}
fn scalar(c: &mut Cursor<'_>) -> Result<Animated<f32>> {
    Ok(Animated {
        enabled: boolean(c)?,
        initial: c.f32()?,
        final_value: c.f32()?,
    })
}
fn vector(c: &mut Cursor<'_>) -> Result<Animated<[f32; 3]>> {
    Ok(Animated {
        enabled: boolean(c)?,
        initial: c.vec3()?,
        final_value: c.vec3()?,
    })
}
pub fn capsule_track(bytes: &[u8]) -> Result<CapsuleTrack> {
    ensure!(
        bytes.len() == 136,
        "unsupported capsule track length {}",
        bytes.len()
    );
    let mut c = Cursor::new(bytes);
    let track = CapsuleTrack {
        header: header(&mut c)?,
        attach_to_joint: u64(&mut c)?,
        animate: boolean(&mut c)?,
        offset: vector(&mut c)?,
        extent: scalar(&mut c)?,
        radius: scalar(&mut c)?,
        axis: vector(&mut c)?,
        rotation_degrees: vector(&mut c)?,
    };
    ensure!(c.pos == bytes.len(), "capsule track trailing bytes");
    for (field, minimum) in [(&track.extent, 0.), (&track.radius, 0.)] {
        if field.enabled {
            ensure!(
                field.initial >= minimum && (!track.animate || field.final_value >= minimum),
                "negative active capsule dimension"
            );
        }
    }
    Ok(track)
}
pub fn steer_track(bytes: &[u8]) -> Result<SteerTrack> {
    let mut c = Cursor::new(bytes);
    let track = SteerTrack {
        header: header(&mut c)?,
        acceleration: c.f32()?,
        turning_velocity_degrees: c.f32()?,
        turning_velocity_run_degrees: c.f32()?,
        velocity_walk: c.f32()?,
        velocity_run: c.f32()?,
        smooth_steering_angle: c.f32()?,
        preserve_facing_joint: u64(&mut c)?,
        preserve_facing_priority: i32(&mut c)?,
        use_locomotion_phase: boolean(&mut c)?,
        phase: c.f32()?,
        locomotion: u64(&mut c)?,
        animations_idle_walk_run: [u64(&mut c)?, u64(&mut c)?, u64(&mut c)?],
        sync_frames_idle_walk_run: [c.f32()?, c.f32()?, c.f32()?],
        partition: u64(&mut c)?,
        priority: i32(&mut c)?,
        blend_in: c.f32()?,
        blend_out: c.f32()?,
        slave_branch: reference(&mut c)?,
    };
    ensure!(c.pos == bytes.len(), "steer track trailing bytes");
    Ok(track)
}

/// Inspected PuppetAnimationTrack properties. Decoding does not execute an action.
pub fn animation_track(bytes: &[u8]) -> Result<AnimationTrack> {
    ensure!(
        (132..=132 + MAX_STRING).contains(&bytes.len()),
        "unsupported animation track length {}",
        bytes.len()
    );
    let mut c = Cursor::new(bytes);
    let track = AnimationTrack {
        header: header(&mut c)?,
        animation: u64(&mut c)?,
        speed: c.f32()?,
        random_speed_variation: c.f32()?,
        init_frame: c.f32()?,
        start_frame: c.f32()?,
        end_frame: c.f32()?,
        cyclic: u64(&mut c)?,
        sync_frame: boolean(&mut c)?,
        phase_match: boolean(&mut c)?,
        sync_phase: u64(&mut c)?,
        sync_phase_min_frame: c.f32()?,
        sync_phase_max_frame: c.f32()?,
        reuse_existing_driver: boolean(&mut c)?,
        has_root_translation: boolean(&mut c)?,
        has_root_rotation: boolean(&mut c)?,
        blend_out_root_translation: boolean(&mut c)?,
        blend_out_root_rotation: boolean(&mut c)?,
        additive_joints: boolean(&mut c)?,
        partition: u64(&mut c)?,
        weight: c.f32()?,
        priority: i32(&mut c)?,
        blend_in: c.f32()?,
        blend_out: c.f32()?,
        synch_tracks_branch: reference(&mut c)?,
    };
    ensure!(c.pos == bytes.len(), "animation track trailing bytes");
    Ok(track)
}

pub fn sprint_track(bytes: &[u8]) -> Result<SprintTrack> {
    ensure!(
        bytes.len() == 208,
        "unsupported sprint track length {}",
        bytes.len()
    );
    let mut c = Cursor::new(bytes);
    let track = SprintTrack {
        header: header(&mut c)?,
        sprint_button: u64(&mut c)?,
        velocities_min_mid_max: [c.f32()?, c.f32()?, c.f32()?],
        accelerations_min_mid_max: [c.f32()?, c.f32()?, c.f32()?],
        deceleration: c.f32()?,
        turning_velocity_degrees_min_max: [c.f32()?, c.f32()?],
        turning_acceleration_degrees_min_max: [c.f32()?, c.f32()?],
        turn_deceleration: c.f32()?,
        lean_rate: c.f32()?,
        phase: c.f32()?,
        locomotion: u64(&mut c)?,
        use_locomotion_phase: boolean(&mut c)?,
        animation_names_min_mid_max: [
            [u64(&mut c)?, u64(&mut c)?, u64(&mut c)?],
            [u64(&mut c)?, u64(&mut c)?, u64(&mut c)?],
            [u64(&mut c)?, u64(&mut c)?, u64(&mut c)?],
        ],
        force_animation_velocities: boolean(&mut c)?,
        unlockable_velocity_min: c.f32()?,
        unlockable_first_last: [u64(&mut c)?, u64(&mut c)?],
        partition: u64(&mut c)?,
        priority: i32(&mut c)?,
        blend_in: c.f32()?,
        blend_out: c.f32()?,
    };
    ensure!(c.pos == bytes.len(), "sprint track trailing bytes");
    Ok(track)
}

struct Envelope<'a> {
    key: u64,
    body: &'a [u8],
    offset: usize,
    body_offset: usize,
}
fn envelope<'a>(c: &mut Cursor<'a>, base: usize) -> Result<Envelope<'a>> {
    let offset = base + c.pos;
    let key = u64(c)?;
    ensure!(key != 0, "zero FIG record type at 0x{offset:x}");
    let len = c.u32()? as usize;
    let body_offset = base + c.pos;
    let body = c
        .take(len)
        .with_context(|| format!("FIG record at 0x{offset:x} exceeds parent"))?;
    ensure!(u64(c)? == 0, "nonzero FIG record trailer at 0x{offset:x}");
    Ok(Envelope {
        key,
        body,
        offset,
        body_offset,
    })
}
fn branch_kind(key: u64) -> Option<&'static str> {
    // This observed type token does not equal the hash of this type label.
    if key == 0xf55e99665d47c7d3 {
        return Some("contentNode");
    }
    [
        "bank",
        "node",
        "store",
        "reference",
        "contentNode",
        "passiveNode",
        "commandNode",
        "commandBank",
        "conditionCheckAtSequenceTimeNode",
    ]
    .into_iter()
    .find(|name| name_hash(name) == key)
}
fn group_kind(key: u64) -> Option<&'static str> {
    [
        "conditions",
        "tracks",
        "onBegin",
        "onEnd",
        "functions",
        "enterTracks",
        "exitTracks",
        "initialTracks",
        "commands",
        "sequenceTimeConditions",
    ]
    .into_iter()
    .find(|name| name_hash(name) == key)
}
fn terminator(c: &mut Cursor<'_>) -> Result<bool> {
    if c.data.len() - c.pos == 8 && c.data[c.pos..] == [0; 8] {
        c.take(8)?;
        return Ok(true);
    }
    Ok(false)
}
fn branches(
    c: &mut Cursor<'_>,
    base: usize,
    graph: &mut Graph,
    parent: Option<usize>,
    depth: usize,
) -> Result<()> {
    ensure!(depth <= MAX_DEPTH, "FIG nesting exceeds limit");
    while c.pos < c.data.len() {
        if terminator(c)? {
            break;
        }
        ensure!(
            graph.branches.len() < MAX_BRANCHES,
            "FIG branch count exceeds limit"
        );
        let record = envelope(c, base)?;
        let kind = branch_kind(record.key).with_context(|| {
            format!(
                "unsupported FIG branch 0x{:016x} at 0x{:x}",
                record.key, record.offset
            )
        })?;
        let mut body = Cursor::new(record.body);
        let name_hash = u64(&mut body)?;
        let serialized_path = string(&mut body)?;
        let sibling_count = body.u32()?;
        ensure!(
            sibling_count <= MAX_BRANCHES as u32,
            "FIG sibling count exceeds limit"
        );
        let reference = if kind == "reference" {
            Some(reference(&mut body)?)
        } else {
            None
        };
        let is_slave = if kind == "contentNode" {
            Some(boolean(&mut body)?)
        } else {
            None
        };
        let index = graph.branches.len();
        graph.branches.push(Branch {
            offset: record.offset,
            kind: kind.to_owned(),
            name_hash,
            serialized_path,
            parent,
            sibling_count,
            reference,
            is_slave,
            derived_path: String::new(),
        });
        while body.pos < body.data.len() {
            if terminator(&mut body)? {
                break;
            }
            let saved = body.pos;
            let property = envelope(&mut body, record.body_offset)?;
            if branch_kind(property.key).is_some() {
                body.pos = saved;
                branches(&mut body, record.body_offset, graph, Some(index), depth + 1)?;
                break;
            }
            let group = group_kind(property.key).with_context(|| {
                format!(
                    "unsupported FIG group 0x{:016x} at 0x{:x}",
                    property.key, property.offset
                )
            })?;
            let mut entries = Cursor::new(property.body);
            let count = entries.u32()? as usize;
            ensure!(
                count <= MAX_RECORDS - graph.records.len(),
                "FIG property count exceeds limit"
            );
            for _ in 0..count {
                let entry = envelope(&mut entries, property.body_offset)?;
                let capsule =
                    if ["tracks", "initialTracks", "enterTracks", "exitTracks"].contains(&group) {
                        if entry.key == self::name_hash("physicsCollisionCapsule") {
                            Some(capsule_track(entry.body).with_context(|| {
                                format!("capsule track at 0x{:x}", entry.offset)
                            })?)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                let steer = if group == "tracks" && entry.key == self::name_hash("locoSteer") {
                    Some(
                        steer_track(entry.body)
                            .with_context(|| format!("steer track at 0x{:x}", entry.offset))?,
                    )
                } else {
                    None
                };
                let sprint = if group == "tracks" && entry.key == self::name_hash("locoSprint") {
                    Some(
                        sprint_track(entry.body)
                            .with_context(|| format!("sprint track at 0x{:x}", entry.offset))?,
                    )
                } else {
                    None
                };
                let animation = if ["tracks", "initialTracks", "enterTracks", "exitTracks"]
                    .contains(&group)
                    && entry.key == self::name_hash("animation")
                {
                    Some(
                        animation_track(entry.body)
                            .with_context(|| format!("animation track at 0x{:x}", entry.offset))?,
                    )
                } else {
                    None
                };
                graph.records.push(Record {
                    offset: entry.offset,
                    body_offset: entry.body_offset,
                    body_bytes: entry.body.len(),
                    type_hash: entry.key,
                    group: group.to_owned(),
                    branch: index,
                    capsule,
                    steer,
                    sprint,
                    animation,
                });
            }
            ensure!(
                entries.pos == entries.data.len(),
                "FIG group length/count mismatch at 0x{:x}",
                property.offset
            );
        }
    }
    Ok(())
}
fn resolve_paths(graph: &mut Graph) -> Result<()> {
    let mut names = BTreeMap::<u64, String>::new();
    for path in graph
        .branches
        .iter()
        .flat_map(|b| {
            [
                Some(b.serialized_path.as_str()),
                b.reference.as_ref().map(|r| r.name.as_str()),
            ]
        })
        .flatten()
    {
        for component in path.split('/').filter(|s| !s.is_empty()) {
            let key = name_hash(component);
            if let Some(previous) = names.insert(key, component.to_owned()) {
                ensure!(previous == component, "ambiguous FIG component hash");
            }
        }
    }
    for index in 0..graph.branches.len() {
        let branch = &graph.branches[index];
        let label = if branch.serialized_path.is_empty() {
            names
                .get(&branch.name_hash)
                .cloned()
                .unwrap_or_else(|| format!("0x{:016x}", branch.name_hash))
        } else {
            branch.serialized_path.clone()
        };
        let path = if let Some(parent) = branch.parent {
            format!("{}/{}", graph.branches[parent].derived_path, label)
        } else {
            label
        };
        ensure!(
            path.len() <= MAX_STRING * 4,
            "FIG derived path exceeds limit"
        );
        graph.branches[index].derived_path = path;
    }
    Ok(())
}

/// Parse a single checked FightData body. `base` preserves decoded-P3D provenance.
pub fn parse_body(bytes: &[u8], base: usize, name: &str) -> Result<Graph> {
    ensure!(
        bytes.len() as u64 <= crate::p3d::MAX_DECODED_BYTES,
        "FIG body exceeds byte limit"
    );
    base.checked_add(bytes.len())
        .context("FIG provenance offset overflow")?;
    let mut c = Cursor::new(bytes);
    ensure!(c.take(4)? == b"fig0", "unsupported FIG signature");
    ensure!(
        c.u32()? == 5 && c.u32()? == 0,
        "unsupported FIG header flags"
    );
    let root = envelope(&mut c, base)?;
    ensure!(root.key == name_hash("chunk"), "unsupported FIG root type");
    ensure!(c.pos == bytes.len(), "trailing FIG root bytes");
    let mut body = Cursor::new(root.body);
    ensure!(body.u32()? == 1, "unsupported FIG context version");
    let context_name_hash = u64(&mut body)?;
    ensure!(
        context_name_hash == name_hash(name),
        "FIG definition/context name mismatch"
    );
    let context_type_hash = u64(&mut body)?;
    let root_reference = reference(&mut body)?;
    let count = body.u32()?;
    ensure!(
        count > 0 && count as usize <= MAX_BRANCHES,
        "FIG declared branch count exceeds limit"
    );
    let mut graph = Graph {
        name: name.to_owned(),
        definition_offset: 0,
        data_offset: base,
        context_name_hash,
        context_type_hash,
        root: root_reference,
        declared_branches_including_root: count,
        branches: Vec::new(),
        records: Vec::new(),
    };
    branches(&mut body, root.body_offset, &mut graph, None, 0)?;
    ensure!(
        graph.branches.len() + 1 == count as usize,
        "FIG branch count mismatch: declared {count}, decoded {} plus root",
        graph.branches.len()
    );
    resolve_paths(&mut graph)?;
    Ok(graph)
}

pub fn load(data: &[u8], chunks: &[Chunk]) -> Result<Vec<Graph>> {
    let mut graphs = Vec::new();
    for (index, definition) in chunks
        .iter()
        .enumerate()
        .filter(|(_, c)| c.id == FIGHT_DEFINITION)
    {
        let mut c = Cursor::new(definition.payload(data));
        let name = c.string8()?;
        ensure!(c.u16()? == 1, "unsupported FightDefinition version");
        let type_name = c.string8()?;
        c.u32()?; // Unresolved definition token, retained outside the executable graph.
        ensure!(c.pos == c.data.len(), "FightDefinition trailing bytes");
        let bodies: Vec<_> = chunks
            .iter()
            .filter(|b| b.parent == Some(index) && b.id == FIGHT_DATA)
            .collect();
        ensure!(
            bodies.len() == 1,
            "FightDefinition {name} must have one data child"
        );
        let chunk = bodies[0];
        let mut c = Cursor::new(chunk.payload(data));
        let len = c.u32()? as usize;
        let bytes = c.take(len)?;
        ensure!(c.pos == c.data.len(), "FightData length mismatch");
        let mut graph = parse_body(bytes, chunk.offset + 16, &name)
            .with_context(|| format!("fight context {name} at 0x{:x}", chunk.offset))?;
        ensure!(
            graph.context_type_hash == name_hash(&type_name),
            "FIG context type mismatch"
        );
        graph.definition_offset = definition.offset;
        graphs.push(graph);
    }
    ensure!(!graphs.is_empty(), "no supported fight contexts found");
    Ok(graphs)
}
