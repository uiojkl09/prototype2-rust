//! Confirmed byte structures, experimental interpretation. No retail engine is called.
use crate::{binary::Cursor, p3d::Chunk};
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct MeshData {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}
#[derive(Clone, Debug, Serialize)]
pub struct CollisionMesh {
    pub chunk_offset: usize,
    pub positions: Vec<[f32; 3]>,
    pub faces: Vec<[u16; 4]>,
    pub unknown_tail_bytes: usize,
}
#[derive(Debug, Serialize)]
pub struct Summary {
    pub world_meshes: usize,
    pub world_vertices: usize,
    pub world_triangles: usize,
    pub ground_collision_meshes: usize,
    pub ground_collision_triangles: usize,
    pub collision_unknown_tail_bytes: usize,
    pub bounds: [[f32; 3]; 2],
    pub omitted_local_vertex_buffers: usize,
    pub limitations: Vec<&'static str>,
}
pub struct Scene {
    pub meshes: Vec<MeshData>,
    pub collision: Vec<CollisionMesh>,
    pub summary: Summary,
}

#[derive(Debug)]
struct Attribute {
    name: String,
    encoding: u32,
    format: u32,
    offset: usize,
    stride: usize,
    count: usize,
}
fn descriptors(chunk: &Chunk, data: &[u8]) -> Result<Vec<Attribute>> {
    let mut c = Cursor::new(chunk.payload(data));
    let count = c.u32()? as usize;
    ensure!(count <= 32, "too many vertex attributes");
    let mut attrs = Vec::new();
    for _ in 0..count {
        let name = c.string8()?;
        let encoding = c.u32()?;
        let format = c.u32()?;
        let offset = c.u32()? as usize;
        let stride = c.u32()? as usize;
        let count = c.u32()? as usize;
        // Colour attributes have a nonzero trailer. They are parsed but not decoded.
        c.take(8)?;
        attrs.push(Attribute {
            name,
            encoding,
            format,
            offset,
            stride,
            count,
        });
    }
    ensure!(
        c.pos == c.data.len(),
        "attribute descriptor length mismatch"
    );
    Ok(attrs)
}
fn buffer<'a>(chunk: &Chunk, data: &'a [u8]) -> Result<&'a [u8]> {
    let mut c = Cursor::new(chunk.payload(data));
    let n = c.u32()? as usize;
    let raw = c.take(n)?;
    ensure!(c.pos == c.data.len(), "buffer length mismatch");
    Ok(raw)
}
fn buffer_name(chunk: &Chunk, data: &[u8]) -> Result<String> {
    let mut c = Cursor::new(chunk.payload(data));
    ensure!(c.u32()? == 0, "unsupported buffer version");
    c.string8()
}
fn parent_physics<'a>(node: &Chunk, chunks: &'a [Chunk]) -> Option<&'a Chunk> {
    let mut parent = node.parent;
    while let Some(i) = parent {
        if chunks[i].id == 0x07020000 {
            return Some(&chunks[i]);
        }
        parent = chunks[i].parent;
    }
    None
}
pub fn load(data: &[u8], chunks: &[Chunk]) -> Result<Scene> {
    let mut vertices: HashMap<String, Vec<[f32; 3]>> = HashMap::new();
    let mut indices: HashMap<String, Vec<u32>> = HashMap::new();
    let mut omitted = 0;
    let mut children: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, c) in chunks.iter().enumerate() {
        if let Some(p) = c.parent {
            children.entry(p).or_default().push(i);
        }
    }
    for (i, node) in chunks
        .iter()
        .enumerate()
        .filter(|(_, c)| c.id == 0x10040 || c.id == 0x10041)
    {
        let name = buffer_name(node, data)?;
        // Only the merged buffers have inspected, baked world coordinates. Prop instances need transforms.
        if !name.starts_with("mergedDrawableRoot") {
            if node.id == 0x10040 {
                omitted += 1;
            }
            continue;
        }
        let suffix = if node.id == 0x10040 {
            "_vertices"
        } else {
            "_indices"
        };
        let base = name
            .strip_suffix(suffix)
            .context("unexpected merged buffer suffix")?
            .to_owned();
        let child_nodes = children.get(&i).context("buffer has no child chunks")?;
        let mut pending = None;
        let mut found = false;
        for &j in child_nodes {
            let child = &chunks[j];
            if child.id == 0x10043 {
                pending = Some(descriptors(child, data)?);
            }
            if child.id != 0x10042 {
                continue;
            }
            let attrs = pending.take().context("raw buffer lacks descriptor")?;
            let raw = buffer(child, data)?;
            if node.id == 0x10040 {
                if let Some(a) = attrs.iter().find(|a| a.name == "position") {
                    ensure!(
                        a.encoding == 0
                            && a.format == 3
                            && a.stride >= 12
                            && a.offset + 12 <= a.stride,
                        "unsupported position format"
                    );
                    ensure!(
                        a.count.checked_mul(a.stride) == Some(raw.len()),
                        "vertex count/stride mismatch"
                    );
                    let mut positions = Vec::with_capacity(a.count);
                    for v in raw.chunks_exact(a.stride) {
                        positions.push(Cursor::new(&v[a.offset..a.offset + 12]).vec3()?);
                    }
                    ensure!(
                        vertices.insert(base.clone(), positions).is_none(),
                        "duplicate position stream"
                    );
                    found = true;
                }
            } else {
                let a = attrs
                    .iter()
                    .find(|a| a.name == "indices")
                    .context("index descriptor missing")?;
                ensure!(
                    a.encoding == 3
                        && a.format == 1
                        && a.stride == 2
                        && a.offset == 0
                        && a.count * 2 == raw.len(),
                    "unsupported index format"
                );
                let mut c = Cursor::new(raw);
                let idx = (0..a.count)
                    .map(|_| c.u16().map(u32::from))
                    .collect::<Result<Vec<_>>>()?;
                ensure!(idx.len() % 3 == 0, "non-triangle index buffer");
                ensure!(
                    indices.insert(base.clone(), idx).is_none(),
                    "duplicate index stream"
                );
                found = true;
            }
        }
        ensure!(found, "no supported stream in {name}");
    }
    let mut meshes = Vec::new();
    for (name, positions) in vertices {
        let idx = indices
            .remove(&name)
            .with_context(|| format!("no index buffer for {name}"))?;
        ensure!(
            idx.iter().all(|&i| (i as usize) < positions.len()),
            "mesh index out of range"
        );
        meshes.push(MeshData {
            name,
            positions,
            indices: idx,
        });
    }
    ensure!(indices.is_empty(), "unpaired merged index buffer");
    meshes.sort_by(|a, b| a.name.cmp(&b.name));
    ensure!(
        !meshes.is_empty(),
        "no merged world geometry in this entry; select a populated cell"
    );
    let mut collision = Vec::new();
    for node in chunks.iter().filter(|c| c.id == 0x07021007) {
        let Some(physics) = parent_physics(node, chunks) else {
            continue;
        };
        if Cursor::new(physics.payload(data)).string8()? != "ground" {
            continue;
        }
        let mut c = Cursor::new(node.payload(data));
        ensure!(c.u32()? == 0, "unsupported triangle collision version");
        let low = c.vec3()?;
        let high = c.vec3()?;
        ensure!(
            (0..3).all(|i| low[i] <= high[i]),
            "inverted collision bounds"
        );
        let n = c.u32()? as usize;
        ensure!(
            n <= 65536 && n * 12 <= c.data.len() - c.pos,
            "invalid collision vertex count"
        );
        let mut positions = Vec::with_capacity(n);
        for _ in 0..n {
            let p = c.vec3()?;
            ensure!(
                (0..3).all(|i| p[i] >= low[i] - 0.05 && p[i] <= high[i] + 0.05),
                "collision vertex outside bounds"
            );
            positions.push(p);
        }
        let n = c.u32()? as usize;
        ensure!(
            n * 8 <= c.data.len() - c.pos,
            "invalid collision face count"
        );
        let mut faces = Vec::with_capacity(n);
        for _ in 0..n {
            let face = [c.u16()?, c.u16()?, c.u16()?, c.u16()?];
            ensure!(
                face[..3].iter().all(|&i| (i as usize) < positions.len()),
                "collision index out of range"
            );
            faces.push(face);
        }
        collision.push(CollisionMesh {
            chunk_offset: node.offset,
            positions,
            faces,
            unknown_tail_bytes: c.data.len() - c.pos,
        });
    }
    let mut bounds = [[f32::INFINITY; 3], [f32::NEG_INFINITY; 3]];
    for p in meshes.iter().flat_map(|m| &m.positions) {
        for i in 0..3 {
            bounds[0][i] = bounds[0][i].min(p[i]);
            bounds[1][i] = bounds[1][i].max(p[i]);
        }
    }
    let summary = Summary {
        world_meshes: meshes.len(),
        world_vertices: meshes.iter().map(|m| m.positions.len()).sum(),
        world_triangles: meshes.iter().map(|m| m.indices.len() / 3).sum(),
        ground_collision_meshes: collision.len(),
        ground_collision_triangles: collision.iter().map(|m| m.faces.len()).sum(),
        collision_unknown_tail_bytes: collision.iter().map(|m| m.unknown_tail_bytes).sum(),
        bounds,
        omitted_local_vertex_buffers: omitted,
        limitations: vec![
            "Untextured merged world geometry only; local props/animated objects omitted",
            "Collision face tags, spatial-tree tail and filtering are unknown; triangle queries are geometric experiments",
            "Free camera is a viewer control, not Prototype 2 character movement",
        ],
    };
    Ok(Scene {
        meshes,
        collision,
        summary,
    })
}

/// Two-sided nearest intersection. This does not implement the retail collision filter or player hull.
pub fn raycast(meshes: &[CollisionMesh], origin: [f32; 3], direction: [f32; 3]) -> Option<f32> {
    fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
    }
    fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }
    fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }
    let mut nearest = f32::INFINITY;
    for m in meshes {
        for f in &m.faces {
            let a = m.positions[f[0] as usize];
            let b = m.positions[f[1] as usize];
            let c = m.positions[f[2] as usize];
            let ab = sub(b, a);
            let ac = sub(c, a);
            let p = cross(direction, ac);
            let det = dot(ab, p);
            if det.abs() < 1e-7 {
                continue;
            }
            let inv = det.recip();
            let s = sub(origin, a);
            let u = dot(s, p) * inv;
            if !(0.0..=1.0).contains(&u) {
                continue;
            }
            let q = cross(s, ab);
            let v = dot(direction, q) * inv;
            if v < 0.0 || u + v > 1.0 {
                continue;
            }
            let t = dot(ac, q) * inv;
            if t >= 0.0 {
                nearest = nearest.min(t);
            }
        }
    }
    nearest.is_finite().then_some(nearest)
}
