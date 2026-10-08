//! Selected Heller pose fixups, independent of render timing and gameplay state.
use crate::{animation::Skeleton, binary::Cursor, meta, p3d::Chunk};
use anyhow::{Context, Result, ensure};
use glam::{Mat4, Vec3};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Collar {
    pub left: String,
    pub right: String,
    pub chin: String,
    pub chin_offset: [f32; 3],
    pub displacement_power: f32,
    pub maximum_displacement: f32,
}
#[derive(Clone, Debug, Serialize)]
pub enum Strategy {
    Collar(Collar),
    Shoulder { pairs: Vec<[String; 2]> },
}
#[derive(Debug, Serialize)]
pub struct Fixups {
    pub definition_offset: usize,
    pub body_offset: usize,
    pub strategies: Vec<Strategy>,
}
fn string(c: &mut Cursor<'_>) -> Result<String> {
    let n = c.u32()? as usize;
    ensure!((1..=128).contains(&n), "invalid fixup string length");
    let text = c.take(n)?;
    ensure!(
        text.iter().all(|b| b.is_ascii_graphic()),
        "invalid fixup name"
    );
    Ok(std::str::from_utf8(text)?.to_owned())
}
/// Only the observed v1 nested layouts and base markers are supported.
pub fn decode_body(body: &[u8]) -> Result<Vec<Strategy>> {
    let mut c = Cursor::new(body);
    ensure!(c.take(4)? == b"META", "missing pose-fixup signature");
    let count = c.u32()? as usize;
    ensure!((1..=32).contains(&count), "invalid fixup strategy count");
    let mut strategies = Vec::with_capacity(count);
    let mut labels = std::collections::HashSet::new();
    for _ in 0..count {
        ensure!(labels.insert(string(&mut c)?), "duplicate fixup label");
        let kind = string(&mut c)?;
        ensure!(c.u16()? == 1, "unsupported fixup version");
        let token = c.u32()?;
        let strategy = match kind.as_str() {
            "PoseStrategyCollarRig" => {
                ensure!(token == 0x08c2bd93, "unsupported collar layout token");
                let collar = Collar {
                    left: string(&mut c)?,
                    right: string(&mut c)?,
                    chin: string(&mut c)?,
                    chin_offset: c.vec3()?,
                    displacement_power: c.f32()?,
                    maximum_displacement: c.f32()?,
                };
                ensure!(
                    collar.displacement_power >= 0. && collar.maximum_displacement >= 0.,
                    "unsupported collar parameters"
                );
                Strategy::Collar(collar)
            }
            "PoseStrategyShoulderCon" => {
                ensure!(token == 0x1a80a495, "unsupported shoulder layout token");
                let count = c.u32()? as usize;
                ensure!((1..=64).contains(&count), "invalid shoulder pair count");
                let mut pairs = Vec::with_capacity(count);
                for _ in 0..count {
                    pairs.push([string(&mut c)?, string(&mut c)?]);
                }
                Strategy::Shoulder { pairs }
            }
            _ => anyhow::bail!("unsupported pose-fixup strategy {kind}"),
        };
        ensure!(
            c.u32()? == 1 && c.u32()? == 1,
            "unsupported fixup base markers"
        );
        strategies.push(strategy);
    }
    ensure!(c.pos == body.len(), "unexpected pose-fixup tail");
    Ok(strategies)
}
pub fn load(data: &[u8], chunks: &[Chunk]) -> Result<Fixups> {
    let objects = meta::inspect(data, chunks, "HellerPoseFixupProperties")?;
    let selected: Vec<_> = objects
        .iter()
        .filter(|o| o.short_name == "HellerPoseFixupProperties")
        .collect();
    ensure!(selected.len() == 1, "expected one Heller pose-fixup object");
    let object = selected[0];
    ensure!(
        object.type_name == "PoseFixupProperties"
            && object.unknown_u16 == [1, 0]
            && object.unknown_u32 == 1106286942,
        "unsupported Heller pose-fixup envelope"
    );
    let end = object
        .body_offset
        .checked_add(object.body_bytes)
        .context("pose-fixup range overflow")?;
    let body = data
        .get(object.body_offset..end)
        .context("pose-fixup body out of range")?;
    Ok(Fixups {
        definition_offset: object.definition_offset,
        body_offset: object.body_offset,
        strategies: decode_body(body)?,
    })
}

enum Operation {
    Collar {
        indices: [usize; 3],
        parameters: Collar,
    },
    Copy {
        source: usize,
        target: usize,
    },
}
pub struct BoundFixups {
    skeleton: String,
    hierarchy: Vec<(String, Option<usize>)>,
    operations: Vec<Operation>,
}
/// Bind only the inspected leaf targets and same-parent shoulder pairs. Broader
/// rigs require the native descendant/update policy, rather than stale matrices.
impl Fixups {
    pub fn bind(&self, skeleton: &Skeleton) -> Result<BoundFixups> {
        ensure!(
            skeleton
                .joints
                .iter()
                .enumerate()
                .all(|(i, j)| j.parent.is_none_or(|p| p < i)),
            "invalid fixup skeleton hierarchy"
        );
        let index = |name: &str| {
            skeleton
                .joints
                .iter()
                .position(|joint| joint.name == name)
                .with_context(|| format!("pose-fixup joint {name} missing"))
        };
        let mut targets = std::collections::HashSet::new();
        let mut target = |i: usize| -> Result<()> {
            ensure!(
                skeleton.joints[i].parent.is_some()
                    && !skeleton.joints.iter().any(|j| j.parent == Some(i)),
                "pose-fixup target must be a non-root leaf"
            );
            ensure!(targets.insert(i), "duplicate pose-fixup target");
            Ok(())
        };
        let mut operations = Vec::new();
        for strategy in &self.strategies {
            match strategy {
                Strategy::Collar(parameters) => {
                    ensure!(
                        parameters.chin_offset.iter().all(|x| x.is_finite())
                            && parameters.displacement_power.is_finite()
                            && parameters.displacement_power >= 0.
                            && parameters.maximum_displacement.is_finite()
                            && parameters.maximum_displacement >= 0.,
                        "invalid collar parameters"
                    );
                    let indices = [
                        index(&parameters.left)?,
                        index(&parameters.right)?,
                        index(&parameters.chin)?,
                    ];
                    target(indices[0])?;
                    target(indices[1])?;
                    ensure!(
                        !indices[..2].contains(&indices[2])
                            && skeleton.joints[indices[2]].parent.is_some()
                            && skeleton.joints[indices[0]].parent
                                == skeleton.joints[indices[1]].parent,
                        "unsupported collar hierarchy"
                    );
                    operations.push(Operation::Collar {
                        indices,
                        parameters: parameters.clone(),
                    });
                }
                Strategy::Shoulder { pairs } => {
                    for [source, destination] in pairs {
                        let source = index(source)?;
                        let destination = index(destination)?;
                        target(destination)?;
                        ensure!(
                            source != destination
                                && skeleton.joints[source].parent
                                    == skeleton.joints[destination].parent,
                            "unsupported shoulder hierarchy"
                        );
                        operations.push(Operation::Copy {
                            source,
                            target: destination,
                        });
                    }
                }
            }
        }
        Ok(BoundFixups {
            skeleton: skeleton.name.clone(),
            hierarchy: skeleton
                .joints
                .iter()
                .map(|j| (j.name.clone(), j.parent))
                .collect(),
            operations,
        })
    }
}
impl BoundFixups {
    /// Apply after pose evaluation, before skin matrices and preview root removal.
    /// Native power/inverse arithmetic is approximated, not bitwise reproduced.
    pub fn apply(&self, skeleton: &Skeleton, world: &mut [Mat4]) -> Result<()> {
        ensure!(
            skeleton.name == self.skeleton
                && skeleton.joints.len() == self.hierarchy.len()
                && skeleton
                    .joints
                    .iter()
                    .zip(&self.hierarchy)
                    .all(|(j, (name, parent))| j.name == *name && j.parent == *parent)
                && world.len() == skeleton.joints.len()
                && skeleton.bind_world.len() == world.len()
                && skeleton.inverse_bind.len() == world.len()
                && world.iter().all(|m| m.is_finite()),
            "pose-fixup skeleton/pose mismatch"
        );
        for operation in &self.operations {
            match operation {
                Operation::Copy { source, target } => world[*target] = world[*source],
                Operation::Collar {
                    indices: [left, right, chin],
                    parameters,
                } => {
                    let parent = skeleton.joints[*left]
                        .parent
                        .context("missing collar parent")?;
                    let chin_parent = skeleton.joints[*chin]
                        .parent
                        .context("missing chin parent")?;
                    let point = skeleton.inverse_bind[chin_parent]
                        .transform_vector3(Vec3::from_array(parameters.chin_offset));
                    let rest = skeleton.inverse_bind[parent]
                        .transform_point3(skeleton.bind_world[*chin].transform_point3(point))
                        .z;
                    ensure!(
                        world[parent].determinant().abs() > 1e-8,
                        "singular collar parent pose"
                    );
                    let inverse_parent = world[parent].inverse();
                    let current = inverse_parent
                        .transform_point3(world[*chin].transform_point3(point))
                        .z;
                    let displacement = current - rest;
                    let gain = (1. + f64::from(displacement).abs())
                        .powf(f64::from(parameters.displacement_power))
                        - 1.;
                    let base_left = skeleton.joints[*left].bind_local.w_axis.truncate();
                    let base_right = skeleton.joints[*right].bind_local.w_axis.truncate();
                    let (left_shift, right_shift) = if displacement > 0. {
                        (
                            0.,
                            ((f64::from(base_right.z) - f64::from(rest)) * gain)
                                .min(f64::from(parameters.maximum_displacement)),
                        )
                    } else {
                        (
                            (-(f64::from(base_left.z) - f64::from(rest)) * gain)
                                .min(f64::from(parameters.maximum_displacement)),
                            0.,
                        )
                    };
                    ensure!(
                        displacement.is_finite()
                            && gain.is_finite()
                            && left_shift.is_finite()
                            && right_shift.is_finite(),
                        "collar displacement overflow"
                    );
                    for (joint, mut base, shift) in [
                        (*left, base_left, -left_shift),
                        (*right, base_right, right_shift),
                    ] {
                        let mut local = inverse_parent * world[joint];
                        base.z += shift as f32;
                        local.w_axis = base.extend(1.);
                        world[joint] = world[parent] * local;
                    }
                }
            }
        }
        ensure!(
            world.iter().all(|m| m.is_finite()),
            "pose-fixup matrix overflow"
        );
        Ok(())
    }
}
