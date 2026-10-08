//! Original readers for the inspected character skeleton and animation chunks.
//! Playback is an inspection facility, not the retail action scheduler.
use crate::{binary::Cursor, p3d::Chunk};
use anyhow::{Context, Result, ensure};
use glam::{Mat4, Quat, Vec3};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::io::Read;

#[derive(Clone, Debug)]
pub struct Joint {
    pub name: String,
    pub parent: Option<usize>,
    pub bind_local: Mat4,
}
#[derive(Clone, Debug)]
pub struct Skeleton {
    pub name: String,
    pub joints: Vec<Joint>,
    pub bind_world: Vec<Mat4>,
    pub inverse_bind: Vec<Mat4>,
}
#[derive(Clone, Debug, Serialize)]
pub struct ClipInfo {
    pub name: String,
    pub offset: usize,
    pub kind: [u8; 4],
    pub end_frame: f32,
    pub frames_per_second: f32,
    pub cyclic: bool,
}
/// Resolve a graph's case-preserving animation hash against actual clip names.
/// Missing or ambiguous references fail rather than selecting a similar label.
pub fn resolve_clip(clips: &[ClipInfo], hash: u64) -> Result<&ClipInfo> {
    let mut matches = clips
        .iter()
        .filter(|c| crate::fight::name_hash(&c.name) == hash);
    let selected = matches
        .next()
        .with_context(|| format!("unresolved animation hash 0x{hash:016x}"))?;
    ensure!(
        matches.next().is_none(),
        "ambiguous animation hash 0x{hash:016x}"
    );
    Ok(selected)
}
fn complete(c: &Cursor<'_>) -> Result<()> {
    ensure!(
        c.pos == c.data.len(),
        "animation payload has unexpected trailing bytes"
    );
    Ok(())
}
fn children(chunks: &[Chunk]) -> HashMap<usize, Vec<usize>> {
    let mut out = HashMap::new();
    for (i, n) in chunks.iter().enumerate() {
        if let Some(p) = n.parent {
            out.entry(p).or_insert_with(Vec::new).push(i);
        }
    }
    out
}
pub fn skeleton(data: &[u8], chunks: &[Chunk], wanted: &str) -> Result<Skeleton> {
    let tree = children(chunks);
    let mut selected = None;
    for (i, node) in chunks.iter().enumerate().filter(|(_, n)| n.id == 0x23000) {
        let mut c = Cursor::new(node.payload(data));
        let name = c.string8()?;
        if name != wanted {
            continue;
        }
        ensure!(selected.is_none(), "duplicate skeleton {wanted}");
        ensure!(c.u32()? == 1, "unsupported skeleton version");
        let count = c.u32()? as usize;
        ensure!(count > 0 && count <= 4096, "invalid skeleton joint count");
        c.u32()?;
        c.u32()?;
        complete(&c)?;
        let mut joints = Vec::new();
        let mut names = HashSet::new();
        for &j in tree.get(&i).context("skeleton has no children")? {
            if chunks[j].id != 0x23001 {
                continue;
            }
            let mut c = Cursor::new(chunks[j].payload(data));
            let name = c.string8()?;
            ensure!(names.insert(name.clone()), "duplicate skeleton joint");
            let p = c.u32()? as usize;
            // The inspected root stores parent 0; subsequent parents precede the joint.
            let parent = if joints.is_empty() {
                ensure!(p == 0, "unsupported skeleton root parent");
                None
            } else {
                ensure!(p < joints.len(), "invalid joint parent ordering");
                Some(p)
            };
            let mut values = [0.; 16];
            for x in &mut values {
                *x = c.f32()?;
            }
            let bind_local = Mat4::from_cols_array(&values);
            ensure!(
                values[3].abs() < 1e-6
                    && values[7].abs() < 1e-6
                    && values[11].abs() < 1e-6
                    && (values[15] - 1.).abs() < 1e-6,
                "non-affine joint matrix"
            );
            ensure!(
                bind_local.determinant().abs() > 1e-6,
                "singular joint matrix"
            );
            // Native v1 loader consumes twelve floats, a u16 and a u32. Their
            // secondary-physics interpretation remains outside this reader.
            for _ in 0..12 {
                c.f32()?;
            }
            c.u16()?;
            c.u32()?;
            complete(&c)?;
            joints.push(Joint {
                name,
                parent,
                bind_local,
            });
        }
        ensure!(joints.len() == count, "skeleton joint count mismatch");
        let bind_world = world_matrices(
            &joints,
            &joints.iter().map(|j| j.bind_local).collect::<Vec<_>>(),
        )?;
        let inverse_bind = bind_world.iter().map(|m| m.inverse()).collect();
        selected = Some(Skeleton {
            name,
            joints,
            bind_world,
            inverse_bind,
        });
    }
    selected.with_context(|| format!("skeleton {wanted} not found"))
}
pub fn clips(data: &[u8], chunks: &[Chunk]) -> Result<Vec<ClipInfo>> {
    chunks
        .iter()
        .filter(|n| n.id == 0x121000)
        .map(|n| {
            let mut c = Cursor::new(n.payload(data));
            ensure!(c.u32()? == 0, "unsupported animation version");
            let name = c.string8()?;
            let kind = c.take(4)?.try_into()?;
            let end_frame = c.f32()?;
            let frames_per_second = c.f32()?;
            ensure!(
                end_frame > 0.
                    && end_frame <= 32767.
                    && frames_per_second > 0.
                    && frames_per_second <= 1000.,
                "invalid animation timing"
            );
            let cyclic = c.u32()?;
            ensure!(cyclic <= 1, "invalid animation cyclic flag");
            complete(&c)?;
            Ok(ClipInfo {
                name,
                offset: n.offset,
                kind,
                end_frame,
                frames_per_second,
                cyclic: cyclic != 0,
            })
        })
        .collect()
}
pub fn world_matrices(joints: &[Joint], local: &[Mat4]) -> Result<Vec<Mat4>> {
    ensure!(joints.len() == local.len(), "joint pose count mismatch");
    let mut out: Vec<Mat4> = Vec::with_capacity(joints.len());
    for (i, (joint, matrix)) in joints.iter().zip(local).enumerate() {
        ensure!(matrix.is_finite(), "non-finite joint pose");
        out.push(if let Some(p) = joint.parent {
            ensure!(p < i, "invalid joint parent ordering");
            out[p] * *matrix
        } else {
            *matrix
        });
    }
    Ok(out)
}

#[derive(Clone, Debug)]
pub struct Keys<T> {
    pub frames: Vec<u16>,
    pub values: Vec<T>,
}
impl<T> Keys<T> {
    fn validate(&self) -> Result<()> {
        ensure!(
            !self.frames.is_empty()
                && self.frames.len() == self.values.len()
                && self.frames.windows(2).all(|w| w[0] < w[1]),
            "invalid animation key arrays"
        );
        Ok(())
    }
    fn interval(&self, frame: f32) -> (usize, usize, f32) {
        let upper = self.frames.partition_point(|&f| f32::from(f) <= frame);
        if upper == 0 {
            return (0, 0, 0.);
        }
        if upper == self.frames.len() {
            return (upper - 1, upper - 1, 0.);
        }
        let lower = upper - 1;
        let fraction = (frame - f32::from(self.frames[lower]))
            / f32::from(self.frames[upper] - self.frames[lower]);
        (lower, upper, fraction)
    }
}
#[derive(Clone, Debug, Default)]
pub struct JointTrack {
    pub rotation: Option<Keys<Quat>>,
    pub translation: Option<Keys<Vec3>>,
}
#[derive(Clone, Debug)]
pub struct Clip {
    pub info: ClipInfo,
    pub tracks: HashMap<String, JointTrack>,
}
impl Clip {
    /// Explicit frame sampling allows deterministic checks independent of render dt.
    /// Time wrapping and state transitions are the caller's responsibility.
    pub fn sample(&self, skeleton: &Skeleton, frame: f32) -> Result<Vec<Mat4>> {
        ensure!(frame.is_finite(), "non-finite animation frame");
        let mut local = Vec::with_capacity(skeleton.joints.len());
        for joint in &skeleton.joints {
            let (scale, mut rotation, mut translation) =
                joint.bind_local.to_scale_rotation_translation();
            if let Some(track) = self.tracks.get(&joint.name) {
                if let Some(keys) = &track.rotation {
                    keys.validate()?;
                    ensure!(
                        keys.values
                            .iter()
                            .all(|q| q.is_finite() && q.is_normalized()),
                        "invalid quaternion keys"
                    );
                    let (a, b, t) = keys.interval(frame);
                    rotation = keys.values[a].slerp(keys.values[b], t);
                }
                if let Some(keys) = &track.translation {
                    keys.validate()?;
                    ensure!(
                        keys.values.iter().all(|v| v.is_finite()),
                        "invalid translation keys"
                    );
                    let (a, b, t) = keys.interval(frame);
                    translation = keys.values[a].lerp(keys.values[b], t);
                }
            }
            local.push(Mat4::from_scale_rotation_translation(
                scale,
                rotation,
                translation,
            ));
        }
        world_matrices(&skeleton.joints, &local)
    }
}

/// Decode selected skeletal clips. Unsupported transform channels are errors;
/// unrelated phase/event channels are not installed as skeletal transforms.
pub fn load_clip(data: &[u8], chunks: &[Chunk], wanted: &str) -> Result<Clip> {
    let info = clips(data, chunks)?
        .into_iter()
        .find(|c| c.name == wanted)
        .with_context(|| format!("animation {wanted} not found"))?;
    ensure!(info.kind == *b"PTRN", "animation is not a skeletal pattern");
    let index = chunks
        .iter()
        .position(|n| n.offset == info.offset)
        .context("missing clip chunk")?;
    let root = &chunks[index];
    let tree = children(chunks);
    let mut blob = None;
    let mut block_sizes = None;
    for node in chunks.iter().filter(|n| {
        n.id == 0x121010 && n.offset > root.offset && n.offset < root.offset + root.total_size
    }) {
        ensure!(
            block_sizes.is_none(),
            "duplicate animation block-size table"
        );
        let mut c = Cursor::new(node.payload(data));
        ensure!(
            c.u32()? == 0 && c.u32()? == 8192,
            "unsupported animation block table"
        );
        let n = c.u32()? as usize;
        ensure!(n > 0 && n <= 256, "invalid animation block count");
        let mut sizes = Vec::with_capacity(n);
        for _ in 0..n {
            sizes.push(c.u32()? as usize);
        }
        complete(&c)?;
        block_sizes = Some(sizes);
    }
    for &i in tree.get(&index).context("clip has no children")? {
        if chunks[i].id != 0x2f00000 {
            continue;
        }
        ensure!(blob.is_none(), "multiple animation blobs unsupported");
        let mut c = Cursor::new(chunks[i].payload(data));
        ensure!(
            c.u32()? == 0 && c.take(4)? == b"ZLIB",
            "unsupported animation blob"
        );
        let decoded = c.u32()? as usize;
        let stored = c.u32()? as usize;
        ensure!(decoded <= 64 * 1024 * 1024, "animation blob exceeds limit");
        let compressed = c.take(stored)?;
        complete(&c)?;
        let mut reader = flate2::read::ZlibDecoder::new(compressed);
        let mut bytes = Vec::new();
        (&mut reader)
            .take(decoded as u64 + 1)
            .read_to_end(&mut bytes)?;
        ensure!(
            bytes.len() == decoded && reader.total_in() as usize == compressed.len(),
            "animation blob length/trailer mismatch"
        );
        blob = Some(bytes);
    }
    let mut blocks = Vec::new();
    if let Some(blob) = &blob {
        let mut offset = 0usize;
        for size in block_sizes.context("animation blob lacks block table")? {
            let end = offset
                .checked_add(size)
                .context("animation block range overflow")?;
            ensure!(
                size > 0 && size % 4 == 0 && end <= blob.len(),
                "invalid animation block range"
            );
            blocks.push(offset..end);
            offset = end;
        }
        ensure!(
            offset == blob.len(),
            "animation block sizes do not cover blob"
        );
    }
    let mut tracks = HashMap::new();
    for (i, node) in chunks.iter().enumerate().filter(|(_, n)| {
        n.id == 0x121001 && n.offset > root.offset && n.offset < root.offset + root.total_size
    }) {
        let mut c = Cursor::new(node.payload(data));
        ensure!(c.u32()? == 0, "unsupported animation group version");
        let name = c.string8()?;
        c.u32()?;
        let channel_count = c.u32()? as usize;
        complete(&c)?;
        ensure!(channel_count <= 32, "too many animation group channels");
        let mut track = JointTrack::default();
        let mut observed = 0;
        for &j in tree.get(&i).map(Vec::as_slice).unwrap_or(&[]) {
            let channel = &chunks[j];
            ensure!(
                (0x121100..=0x121119).contains(&channel.id),
                "unsupported animation group child"
            );
            observed += 1;
            let mut c = Cursor::new(channel.payload(data));
            let version = c.u32()?;
            let kind = c.take(4)?;
            if kind != b"ROT\0" && kind != b"TRAN" {
                continue;
            }
            let axis = if matches!(channel.id, 0x121102 | 0x121103 | 0x121118) {
                Some(c.u16()? as usize)
            } else {
                None
            };
            let base = if axis.is_some() {
                Some(c.vec3()?)
            } else {
                None
            };
            let count = c.u32()? as usize;
            let mut reference = None;
            for &child in tree.get(&j).map(Vec::as_slice).unwrap_or(&[]) {
                ensure!(
                    chunks[child].id == 0x121121,
                    "unsupported animation channel child"
                );
                ensure!(reference.is_none(), "duplicate animation channel reference");
                let mut r = Cursor::new(chunks[child].payload(data));
                ensure!(r.u32()? == 0, "unsupported animation reference version");
                let count = r.u32()? as usize;
                let offset = r.u32()? as usize;
                let block = r.u16()? as usize;
                complete(&r)?;
                reference = Some((count, offset, block));
            }
            let mut keys = if let Some((n, offset, block)) = reference {
                ensure!(
                    count == 0,
                    "animation channel has both inline and referenced keys"
                );
                complete(&c)?;
                let bytes = blob
                    .as_ref()
                    .context("channel reference lacks animation blob")?;
                let range = blocks
                    .get(block)
                    .context("animation reference block out of range")?;
                let offset = range
                    .start
                    .checked_add(offset)
                    .context("animation reference range overflow")?;
                let raw = bytes
                    .get(offset..range.end)
                    .context("animation reference offset out of range")?;
                (n, Cursor::new(raw), Some(offset))
            } else {
                (count, c, None)
            };
            ensure!(keys.0 > 0 && keys.0 <= 32768, "invalid animation key count");
            let mut frames = Vec::with_capacity(keys.0);
            for _ in 0..keys.0 {
                let f = keys.1.u16()?;
                ensure!(
                    f32::from(f) <= info.end_frame && frames.last().is_none_or(|last| *last < f),
                    "invalid animation key order/range"
                );
                frames.push(f);
            }
            if let Some(offset) = keys.2 {
                let pad = (4 - (offset + keys.1.pos) % 4) % 4;
                keys.1.take(pad)?;
            }
            if kind == b"ROT\0" {
                ensure!(
                    version == 1 && matches!(channel.id, 0x121112 | 0x121114),
                    "unsupported rotation encoding/version"
                );
                ensure!(track.rotation.is_none(), "duplicate joint rotation channel");
                let mut values = Vec::with_capacity(keys.0);
                for _ in 0..keys.0 {
                    let xyz = if channel.id == 0x121112 {
                        [
                            keys.1.u16()? as i16 as f32,
                            keys.1.u16()? as i16 as f32,
                            keys.1.u16()? as i16 as f32,
                        ]
                        .map(|v| v * f32::from_bits(0x38000100))
                    } else {
                        let v = keys.1.take(3)?;
                        [v[0] as i8 as f32, v[1] as i8 as f32, v[2] as i8 as f32]
                            .map(|v| v * f32::from_bits(0x3c010204))
                    };
                    let w = (1. - xyz.iter().map(|v| v * v).sum::<f32>()).max(0.).sqrt();
                    let q = Quat::from_xyzw(xyz[0], xyz[1], xyz[2], w);
                    ensure!(
                        q.is_finite() && q.length_squared() > 0.5,
                        "invalid compressed quaternion"
                    );
                    // Quantization can overshoot the unit sphere. Native unpack clamps
                    // the radicand; normalize for the inspection math library.
                    values.push(q.normalize());
                }
                track.rotation = Some(Keys { frames, values });
            } else {
                ensure!(version == 0, "unsupported translation version");
                ensure!(
                    track.translation.is_none(),
                    "duplicate joint translation channel"
                );
                let mut values = Vec::with_capacity(keys.0);
                for _ in 0..keys.0 {
                    let v = match channel.id {
                        0x121104 => keys.1.vec3()?,
                        0x121119 => [
                            half(keys.1.u16()?),
                            half(keys.1.u16()?),
                            half(keys.1.u16()?),
                        ],
                        0x121103 | 0x121118 => {
                            let axis = axis.context("missing translation constant axis")?;
                            ensure!(axis < 3, "invalid translation constant axis");
                            let mut v = base.context("missing translation base")?;
                            let components = [[1, 2], [0, 2], [0, 1]][axis];
                            for i in components {
                                v[i] = if channel.id == 0x121118 {
                                    half(keys.1.u16()?)
                                } else {
                                    keys.1.f32()?
                                };
                            }
                            v
                        }
                        0x121102 => {
                            let axis = axis.context("missing translation axis")?;
                            ensure!(axis < 3, "invalid translation axis");
                            let mut v = base.context("missing translation base")?;
                            v[axis] = keys.1.f32()?;
                            v
                        }
                        _ => anyhow::bail!(
                            "unsupported translation channel 0x{:x} in {name}",
                            channel.id
                        ),
                    };
                    ensure!(
                        v.iter().all(|v| v.is_finite()),
                        "non-finite animation translation"
                    );
                    values.push(Vec3::from_array(v));
                }
                track.translation = Some(Keys { frames, values });
            }
            if keys.2.is_none() {
                complete(&keys.1)?;
            }
        }
        ensure!(
            observed == channel_count,
            "animation channel count mismatch"
        );
        ensure!(
            tracks.insert(name, track).is_none(),
            "duplicate animation joint group"
        );
    }
    ensure!(!tracks.is_empty(), "animation has no joint tracks");
    Ok(Clip { info, tracks })
}
/// IEEE binary16 expansion, including subnormals; non-finite values are rejected
/// by the channel reader. No retail code or GPU codec is required.
pub fn half(bits: u16) -> f32 {
    let sign = u32::from(bits & 0x8000) << 16;
    let exponent = (bits >> 10) & 31;
    let fraction = u32::from(bits & 1023);
    if exponent == 0 {
        if fraction == 0 {
            return f32::from_bits(sign);
        }
        let n = (fraction as f32) * 2f32.powi(-24);
        return if sign == 0 { n } else { -n };
    }
    let e = if exponent == 31 {
        255
    } else {
        u32::from(exponent) + 112
    };
    f32::from_bits(sign | (e << 23) | (fraction << 13))
}
