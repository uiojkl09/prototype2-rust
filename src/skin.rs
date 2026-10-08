//! Selected character drawables and four-influence CPU skinning for animation inspection.
use crate::{animation::Skeleton, binary::Cursor, p3d::Chunk, scene};
use anyhow::{Context, Result, ensure};
use glam::{Mat4, Vec3};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SkinMesh {
    pub name: String,
    pub skeleton: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub weights: Vec<[f32; 4]>,
    pub joints: Vec<[u16; 4]>,
    pub indices: Vec<u32>,
}
pub struct Deformed {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
}
impl SkinMesh {
    pub fn deform(&self, skeleton: &Skeleton, world: &[Mat4]) -> Result<Deformed> {
        ensure!(
            self.skeleton == skeleton.name
                && world.len() == skeleton.joints.len()
                && world.len() == skeleton.inverse_bind.len(),
            "skin skeleton/pose mismatch"
        );
        let matrices: Vec<_> = world
            .iter()
            .zip(&skeleton.inverse_bind)
            .map(|(w, b)| *w * *b)
            .collect();
        ensure!(
            matrices.iter().all(|m| m.is_finite()),
            "non-finite skin transform"
        );
        let n = self.positions.len();
        ensure!(
            self.normals.len() == n && self.weights.len() == n && self.joints.len() == n,
            "skin stream count mismatch"
        );
        let mut positions = Vec::with_capacity(n);
        let mut normals = Vec::with_capacity(n);
        for i in 0..n {
            let mut p = Vec3::ZERO;
            let mut normal = Vec3::ZERO;
            for k in 0..4 {
                let weight = self.weights[i][k];
                ensure!(weight.is_finite() && weight >= 0., "invalid skin weight");
                if weight == 0. {
                    continue;
                }
                let m = matrices
                    .get(self.joints[i][k] as usize)
                    .context("skin joint out of range")?;
                p += m.transform_point3(Vec3::from_array(self.positions[i])) * weight;
                normal += m.transform_vector3(Vec3::from_array(self.normals[i])) * weight;
            }
            ensure!(
                p.is_finite() && normal.is_finite(),
                "invalid deformed vertex"
            );
            positions.push(p.to_array());
            normals.push(normal.normalize_or_zero().to_array());
        }
        Ok(Deformed { positions, normals })
    }
}
pub fn load(data: &[u8], chunks: &[Chunk], skeletons: &[Skeleton]) -> Result<Vec<SkinMesh>> {
    let mut tree: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, n) in chunks.iter().enumerate() {
        if let Some(p) = n.parent {
            tree.entry(p).or_default().push(i);
        }
    }
    let mut buffers = HashMap::new();
    for (i, n) in chunks
        .iter()
        .enumerate()
        .filter(|(_, n)| n.id == 0x10040 || n.id == 0x10041)
    {
        let name = scene::buffer_name(n, data)?;
        let mut attrs = None;
        let mut found = None;
        for &j in tree.get(&i).context("skin buffer lacks children")? {
            match chunks[j].id {
                0x10043 => {
                    ensure!(attrs.is_none(), "duplicate skin descriptor");
                    attrs = Some(scene::descriptors(&chunks[j], data)?);
                }
                0x10042 => {
                    ensure!(found.is_none(), "multiple raw skin streams unsupported");
                    found = Some(scene::buffer(&chunks[j], data)?);
                }
                _ => {}
            }
        }
        ensure!(
            buffers
                .insert(
                    name,
                    (
                        attrs.context("missing skin descriptor")?,
                        found.context("missing raw skin buffer")?
                    )
                )
                .is_none(),
            "duplicate skin buffer name"
        );
    }
    let mut meshes = Vec::new();
    for node in chunks.iter().filter(|n| n.id == 0x25002) {
        let primitive = node.parent.context("skin reference lacks primitive")?;
        ensure!(
            chunks[primitive].id == 0x25001,
            "unexpected skin reference parent"
        );
        let drawable = chunks[primitive]
            .parent
            .context("skin primitive lacks drawable")?;
        ensure!(
            chunks[drawable].id == 0x25000,
            "unexpected skin drawable parent"
        );
        let mut c = Cursor::new(chunks[drawable].payload(data));
        ensure!(c.u32()? == 0, "unsupported skin drawable version");
        c.string8()?;
        let skeleton_name = c.string8()?;
        let Some(skeleton) = skeletons.iter().find(|s| s.name == skeleton_name) else {
            continue;
        };
        let mut c = Cursor::new(node.payload(data));
        ensure!(c.u32()? == 0, "unsupported skin reference version");
        c.string8()?;
        ensure!(c.u32()? == 0, "unsupported skin reference flags");
        let index_name = c.string8()?;
        let vertex_name = c.string8()?;
        let skin_name = c.string8()?;
        ensure!(
            c.take(1)? == [0] && c.pos == c.data.len(),
            "unsupported skin reference trailer"
        );
        let (attrs, raw) = buffers
            .get(&skin_name)
            .context("referenced skin stream missing")?;
        let attr = |name: &str, encoding, components, bytes| -> Result<&scene::Attribute> {
            let a = attrs
                .iter()
                .find(|a| a.name == name)
                .context("required skin attribute missing")?;
            ensure!(
                a.encoding == encoding
                    && a.format == components
                    && a.stride == 80
                    && a.count.checked_mul(a.stride) == Some(raw.len())
                    && a.offset.checked_add(bytes).is_some_and(|n| n <= a.stride),
                "unsupported skin attribute format"
            );
            Ok(a)
        };
        let position = attr("position", 0, 4, 16)?;
        let normal = attr("normal", 0, 4, 16)?;
        let weights = attr("weights", 0, 4, 16)?;
        let joints = attr("indices", 3, 4, 8)?;
        ensure!(
            position.count == normal.count
                && normal.count == weights.count
                && weights.count == joints.count,
            "skin attribute count mismatch"
        );
        let mut mesh = SkinMesh {
            name: vertex_name,
            skeleton: skeleton.name.clone(),
            positions: Vec::new(),
            normals: Vec::new(),
            weights: Vec::new(),
            joints: Vec::new(),
            indices: Vec::new(),
        };
        for v in raw.chunks_exact(80) {
            let mut p = Cursor::new(&v[position.offset..position.offset + 16]);
            let xyz = p.vec3()?;
            ensure!((p.f32()? - 1.).abs() < 1e-6, "unsupported skin position W");
            mesh.positions.push(xyz);
            let mut n = Cursor::new(&v[normal.offset..normal.offset + 16]);
            mesh.normals.push(n.vec3()?);
            // All inspected normal float4 records store 1 in the fourth lane;
            // it is retained as a checked layout marker, not used as position W.
            ensure!(
                (n.f32()? - 1.).abs() < 1e-6,
                "unsupported skin normal fourth lane"
            );
            let mut w = Cursor::new(&v[weights.offset..weights.offset + 16]);
            let weights = [w.f32()?, w.f32()?, w.f32()?, w.f32()?];
            ensure!(
                weights.iter().all(|w| *w >= 0.)
                    && (weights.iter().sum::<f32>() - 1.).abs() < 0.001,
                "invalid skin weights"
            );
            mesh.weights.push(weights);
            let mut j = Cursor::new(&v[joints.offset..joints.offset + 8]);
            let joints = [j.u16()?, j.u16()?, j.u16()?, j.u16()?];
            ensure!(
                joints.iter().all(|j| (*j as usize) < skeleton.joints.len()),
                "skin joint index out of range"
            );
            mesh.joints.push(joints);
        }
        let (attrs, raw) = buffers
            .get(&index_name)
            .context("referenced skin index stream missing")?;
        ensure!(attrs.len() == 1, "unsupported skin index descriptors");
        let a = &attrs[0];
        ensure!(
            a.name == "indices"
                && a.encoding == 3
                && a.format == 1
                && a.offset == 0
                && a.stride == 2
                && a.count.checked_mul(2) == Some(raw.len())
                && a.count % 3 == 0,
            "unsupported skin index format"
        );
        let mut c = Cursor::new(raw);
        for _ in 0..a.count {
            let i = u32::from(c.u16()?);
            ensure!(
                (i as usize) < mesh.positions.len(),
                "skin vertex index out of range"
            );
            mesh.indices.push(i);
        }
        meshes.push(mesh);
    }
    ensure!(!meshes.is_empty(), "no selected character skin drawables");
    Ok(meshes)
}
