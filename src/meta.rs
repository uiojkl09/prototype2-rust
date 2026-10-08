//! Checked metadata envelopes; bodies remain opaque until their schemas are recovered.
use crate::{binary::Cursor, p3d::Chunk};
use anyhow::{Result, ensure};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Object {
    pub definition_offset: usize,
    pub long_name: String,
    pub short_name: String,
    pub type_name: String,
    pub unknown_u16: [u16; 2],
    pub unknown_u32: u32,
    pub data_chunk_offset: usize,
    pub body_offset: usize,
    pub body_bytes: usize,
    pub has_meta_signature: bool,
    pub matched_reference_offsets: Vec<usize>,
}

/// Search bounded ASCII references in opaque bodies, without interpreting data as
/// properties or exporting payloads. Match offsets are absolute within decoded P3D.
pub fn inspect(data: &[u8], chunks: &[Chunk], filter: &str) -> Result<Vec<Object>> {
    ensure!(
        filter.is_ascii() && filter.len() <= 128,
        "meta filter must be ASCII, at most 128 bytes"
    );
    let needle = filter.to_ascii_lowercase();
    let mut definitions = std::collections::HashMap::new();
    let mut objects = Vec::new();
    for (index, chunk) in chunks.iter().enumerate() {
        if chunk.id == 0x07f00000 {
            let mut c = Cursor::new(chunk.payload(data));
            let long = c.string8()?;
            let short = c.string8()?;
            let kind = c.string8()?;
            let unknown = [c.u16()?, c.u16()?];
            let hash = c.u32()?;
            ensure!(
                c.pos == c.data.len(),
                "unsupported meta definition tail at 0x{:x}",
                chunk.offset
            );
            definitions.insert(index, (chunk.offset, long, short, kind, unknown, hash));
        } else if chunk.id == 0x07f00001 {
            let Some(definition) = chunk.parent.and_then(|p| definitions.get(&p)) else {
                continue;
            };
            let mut c = Cursor::new(chunk.payload(data));
            let length = c.u32()? as usize;
            let body = c.take(length)?;
            ensure!(
                c.pos == c.data.len(),
                "meta data length/tail mismatch at 0x{:x}",
                chunk.offset
            );
            let body_offset = chunk.offset + 16;
            let mut matches = Vec::new();
            if !needle.is_empty() {
                for (offset, candidate) in body.windows(needle.len()).enumerate() {
                    if candidate.eq_ignore_ascii_case(needle.as_bytes()) {
                        matches.push(body_offset + offset);
                        if matches.len() == 32 {
                            break;
                        }
                    }
                }
            }
            let name_match = [&definition.1, &definition.2, &definition.3]
                .iter()
                .any(|s| s.to_ascii_lowercase().contains(&needle));
            if !needle.is_empty() && !name_match && matches.is_empty() {
                continue;
            }
            objects.push(Object {
                definition_offset: definition.0,
                long_name: definition.1.clone(),
                short_name: definition.2.clone(),
                type_name: definition.3.clone(),
                unknown_u16: definition.4,
                unknown_u32: definition.5,
                data_chunk_offset: chunk.offset,
                body_offset,
                body_bytes: length,
                has_meta_signature: body.starts_with(b"META"),
                matched_reference_offsets: matches,
            });
        }
    }
    Ok(objects)
}
