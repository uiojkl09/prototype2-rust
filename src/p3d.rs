use crate::binary::Cursor;
use anyhow::{Context, Result, ensure};
use flate2::read::ZlibDecoder;
use serde::Serialize;
use std::{collections::BTreeMap, io::Read};

pub const MAGIC: u32 = 0xff443350;
pub const MAX_DECODED_BYTES: u64 = 256 * 1024 * 1024;
#[derive(Clone, Debug, Serialize)]
pub struct Chunk {
    pub id: u32,
    pub offset: usize,
    pub data_size: usize,
    pub total_size: usize,
    pub depth: usize,
    pub parent: Option<usize>,
}
impl Chunk {
    pub fn payload<'a>(&self, data: &'a [u8]) -> &'a [u8] {
        &data[self.offset + 12..self.offset + self.data_size]
    }
}
pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
    ensure!(
        data.len() as u64 <= MAX_DECODED_BYTES,
        "input exceeds decode limit"
    );
    if !data.starts_with(b"RZ") {
        return Ok(data.to_vec());
    }
    let mut c = Cursor::new(data);
    ensure!(c.take(8)? == b"RZ\0\0\0\0\0\0", "unsupported RZ header");
    let size = c.u32()? as u64;
    ensure!(c.u32()? == 0, "unsupported RZ reserved field");
    ensure!(size <= MAX_DECODED_BYTES, "RZ exceeds decode limit");
    let mut decoder = ZlibDecoder::new(&data[16..]);
    let mut out = Vec::new();
    (&mut decoder)
        .take(size + 1)
        .read_to_end(&mut out)
        .context("invalid zlib stream")?;
    ensure!(out.len() as u64 == size, "RZ decoded length mismatch");
    ensure!(
        decoder.total_in() as usize == data.len() - 16,
        "trailing or incomplete RZ stream"
    );
    Ok(out)
}
pub fn parse(data: &[u8]) -> Result<Vec<Chunk>> {
    let mut c = Cursor::new(data);
    ensure!(c.u32()? == MAGIC, "unsupported Pure3D signature/endian");
    ensure!(
        c.u32()? == 12 && c.u32()? as usize == data.len(),
        "Pure3D root length mismatch"
    );
    let mut chunks = Vec::new();
    // Stack avoids recursion on malformed files. Depth and node count are bounded.
    let mut stack = vec![(0usize, data.len(), 0usize, None)];
    while let Some((mut pos, end, depth, parent)) = stack.pop() {
        ensure!(depth <= 128, "Pure3D nesting exceeds limit");
        while pos < end {
            ensure!(chunks.len() < 1_000_000, "Pure3D chunk count exceeds limit");
            let mut c = Cursor::new(data.get(pos..end).context("invalid child range")?);
            let id = c.u32()?;
            let ds = c.u32()? as usize;
            let ts = c.u32()? as usize;
            ensure!(
                12 <= ds && ds <= ts && ts <= end - pos,
                "invalid chunk 0x{id:08x} at 0x{pos:x}"
            );
            let index = chunks.len();
            chunks.push(Chunk {
                id,
                offset: pos,
                data_size: ds,
                total_size: ts,
                depth,
                parent,
            });
            if ds < ts {
                if pos + ts < end {
                    stack.push((pos + ts, end, depth, parent));
                }
                stack.push((pos + ds, pos + ts, depth + 1, Some(index)));
                break;
            }
            pos += ts;
        }
    }
    Ok(chunks)
}
pub fn histogram(chunks: &[Chunk]) -> BTreeMap<String, usize> {
    let mut h = BTreeMap::new();
    for c in chunks {
        *h.entry(format!("0x{:08x}", c.id)).or_default() += 1;
    }
    h
}
