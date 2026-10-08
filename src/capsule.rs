//! The observed AlexPhysicsFactory v1 asset layout, not a locomotion-state hull.
use crate::{binary::Cursor, meta, p3d::Chunk};
use anyhow::{Context, Result, ensure};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Capsule {
    pub centre: [f32; 3],
    pub axis: [f32; 3],
    pub extent: f32,
    pub radius: f32,
}
impl Capsule {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.centre.iter().chain(&self.axis).all(|x| x.is_finite()),
            "non-finite capsule vector"
        );
        let length2: f64 = self.axis.iter().map(|&x| f64::from(x).powi(2)).sum();
        ensure!(
            (length2 - 1.).abs() < 1e-5,
            "capsule axis must be unit length"
        );
        ensure!(
            self.extent.is_finite() && self.extent >= 0.,
            "invalid capsule extent"
        );
        ensure!(
            self.radius.is_finite() && self.radius > 0.,
            "invalid capsule radius"
        );
        Ok(())
    }
    pub fn endpoints(&self, origin: [f64; 3]) -> [[f64; 3]; 2] {
        [
            std::array::from_fn(|i| {
                origin[i] + f64::from(self.centre[i])
                    - f64::from(self.axis[i]) * f64::from(self.extent)
            }),
            std::array::from_fn(|i| {
                origin[i]
                    + f64::from(self.centre[i])
                    + f64::from(self.axis[i]) * f64::from(self.extent)
            }),
        ]
    }
}
#[derive(Debug, Serialize)]
pub struct Factory {
    pub definition_offset: usize,
    pub body_offset: usize,
    pub material_reference: String,
    pub intersection_properties_reference: String,
    pub shape: Capsule,
    pub unknown_tail_bytes: usize,
}

/// Fail closed on any body identity/layout other than the inspected factory.
pub fn load(data: &[u8], chunks: &[Chunk]) -> Result<Factory> {
    let objects = meta::inspect(data, chunks, "AlexPhysicsFactory")?;
    let found: Vec<_> = objects
        .iter()
        .filter(|o| o.short_name == "AlexPhysicsFactory")
        .collect();
    ensure!(
        found.len() == 1,
        "expected one AlexPhysicsFactory, found {}",
        found.len()
    );
    let o = found[0];
    ensure!(
        o.type_name == "ravenphysics::SimplePhysicsObjectFactory"
            && o.unknown_u16 == [1, 0]
            && o.unknown_u32 == 3711084174,
        "unsupported AlexPhysicsFactory envelope"
    );
    let body = data
        .get(
            o.body_offset
                ..o.body_offset
                    .checked_add(o.body_bytes)
                    .context("capsule range overflow")?,
        )
        .context("capsule body out of range")?;
    let (material_reference, intersection_properties_reference, shape, unknown_tail_bytes) =
        decode_body(body)?;
    Ok(Factory {
        definition_offset: o.definition_offset,
        body_offset: o.body_offset,
        material_reference,
        intersection_properties_reference,
        shape,
        unknown_tail_bytes,
    })
}

pub fn decode_body(body: &[u8]) -> Result<(String, String, Capsule, usize)> {
    let mut c = Cursor::new(body);
    ensure!(c.take(4)? == b"META", "missing capsule META signature");
    let n = c.u32()? as usize;
    ensure!(
        n == 37 && c.take(n)? == b"ravenphysics::CollisionCapsuleFactory",
        "unsupported capsule factory type"
    );
    ensure!(
        c.u16()? == 1 && c.u32()? == 0x9ca40a36,
        "unsupported capsule factory version/token"
    );
    let material = c.string8()?;
    let group = c.string8()?;
    let shape = Capsule {
        centre: c.vec3()?,
        axis: c.vec3()?,
        extent: c.f32()?,
        radius: c.f32()?,
    };
    shape.validate()?;
    // SimplePhysicsObjectFactory fields beyond its nested volume are not decoded.
    ensure!(
        c.data.len() - c.pos == 13,
        "unsupported simple physics factory tail length"
    );
    c.take(13)?;
    Ok((material, group, shape, 13))
}
