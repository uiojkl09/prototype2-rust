//! New Rust implementation informed by Gibbed.Prototype and RcfTools; see THIRD_PARTY.md.
use crate::binary::{Cursor, text};
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

pub const MAX_ENTRY_BYTES: u64 = 256 * 1024 * 1024;
const MAX_TABLE_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Serialize)]
pub struct Entry {
    pub name: String,
    pub hash: u32,
    pub offset: u64,
    pub size: u64,
    pub type_hash: u32,
    pub alignment: u32,
}
pub struct Archive {
    pub path: PathBuf,
    pub file_size: u64,
    pub entries: Vec<Entry>,
}

/// Retail hash transforms *all* ASCII bytes below 0x61, including punctuation.
/// Ordinary ASCII lowercasing gives the wrong hash for digits and separators.
pub fn name_hash(name: &str) -> u32 {
    name.strip_prefix('\\')
        .unwrap_or(name)
        .bytes()
        .fold(0u32, |v, b| {
            v.wrapping_mul(31)
                .wrapping_add(if b < 0x61 { b as u32 + 32 } else { b as u32 })
        })
}
pub fn normalize(name: &str) -> String {
    name.replace('/', "\\")
        .trim_start_matches('\\')
        .to_ascii_lowercase()
}

fn region(file: &mut File, offset: u64, size: u64, file_size: u64, cap: u64) -> Result<Vec<u8>> {
    ensure!(size <= cap, "read exceeds {cap} byte limit");
    ensure!(
        offset.checked_add(size).is_some_and(|end| end <= file_size),
        "range outside archive"
    );
    file.seek(SeekFrom::Start(offset))?;
    let mut data = vec![0; usize::try_from(size)?];
    file.read_exact(&mut data)?;
    Ok(data)
}
impl Archive {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut file = File::open(path).with_context(|| format!("open {}", path.display()))?;
        let file_size = file.metadata()?.len();
        let header = region(&mut file, 0, 60, file_size, 60)?;
        ensure!(
            &header[..23] == b"ATG CORE CEMENT LIBRARY",
            "invalid RCF magic"
        );
        ensure!(
            header[23..32].iter().all(|&b| b == 0),
            "unexpected signature padding"
        );
        ensure!(
            header[32..36] == [2, 1, 0, 1],
            "unsupported RCF version/endian/valid flag"
        );
        let mut c = Cursor::new(&header[36..]);
        let eo = c.u32()? as u64;
        let el = c.u32()? as u64;
        let mo = c.u32()? as u64;
        let ml = c.u32()? as u64;
        ensure!(c.u32()? == 0, "unexpected RCF header field");
        let count = c.u32()? as usize;
        ensure!(count as u64 * 12 == el, "entry count/table length mismatch");
        ensure!(eo >= 60 && eo + el <= mo, "overlapping RCF tables");
        let entry_bytes = region(&mut file, eo, el, file_size, MAX_TABLE_BYTES)?;
        let metadata = region(&mut file, mo, ml, file_size, MAX_TABLE_BYTES)?;
        let mut c = Cursor::new(&metadata);
        ensure!(
            c.u32()? == 2048 && c.u32()? == 0,
            "unsupported metadata preamble"
        );
        let mut names = HashMap::new();
        for _ in 0..count {
            let type_hash = c.u32()?;
            let alignment = c.u32()?;
            ensure!(alignment.is_power_of_two(), "invalid entry alignment");
            ensure!(c.u32()? == 0, "unexpected metadata field");
            let len = c.u32()? as usize;
            ensure!(len > 0, "empty serialized filename");
            let bytes = c.take(len)?;
            ensure!(bytes.last() == Some(&0), "filename lacks terminator");
            let name = text(bytes)?;
            ensure!(!name.is_empty() && name.is_ascii(), "invalid archive name");
            ensure!(c.take(3)? == [0, 0, 0], "unexpected metadata trailer");
            ensure!(
                names
                    .insert(name_hash(&name), (name, type_hash, alignment))
                    .is_none(),
                "duplicate filename hash"
            );
        }
        ensure!(c.pos == metadata.len(), "trailing metadata bytes");
        let mut c = Cursor::new(&entry_bytes);
        let mut entries = Vec::with_capacity(count);
        let mut hashes = HashSet::new();
        for _ in 0..count {
            let hash = c.u32()?;
            let offset = c.u32()? as u64;
            let size = c.u32()? as u64;
            let (name, type_hash, alignment) = names
                .remove(&hash)
                .context("entry hash has no matching filename")?;
            ensure!(hashes.insert(hash), "duplicate entry hash");
            ensure!(
                offset >= mo + ml && offset.checked_add(size).is_some_and(|e| e <= file_size),
                "invalid payload range: {name}"
            );
            ensure!(
                offset.is_multiple_of(alignment as u64),
                "misaligned payload: {name}"
            );
            entries.push(Entry {
                name,
                hash,
                offset,
                size,
                type_hash,
                alignment,
            });
        }
        let mut ranges: Vec<_> = entries
            .iter()
            .filter(|e| e.size != 0)
            .map(|e| (e.offset, e.offset + e.size))
            .collect();
        ranges.sort_unstable();
        ensure!(
            ranges.windows(2).all(|r| r[0].1 <= r[1].0),
            "overlapping payloads"
        );
        Ok(Self {
            path: path.to_owned(),
            file_size,
            entries,
        })
    }
    pub fn find(&self, name: &str) -> Result<&Entry> {
        let name = normalize(name);
        self.entries
            .iter()
            .find(|e| normalize(&e.name) == name)
            .context("entry not found (use list)")
    }
    pub fn read(&self, entry: &Entry) -> Result<Vec<u8>> {
        let mut file = File::open(&self.path)?;
        region(
            &mut file,
            entry.offset,
            entry.size,
            self.file_size,
            MAX_ENTRY_BYTES,
        )
    }
}
