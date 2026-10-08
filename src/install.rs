use anyhow::{Context, Result, ensure};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

#[derive(Serialize)]
pub struct BinaryIdentity {
    pub name: String,
    pub size: u64,
    pub sha256: String,
    pub pe_machine: String,
    pub pe_timestamp: u32,
}
#[derive(Serialize)]
pub struct Identity {
    pub steam_app_id: &'static str,
    pub steam_build_id: Option<String>,
    pub binaries: Vec<BinaryIdentity>,
}
pub fn fingerprint(game: &Path) -> Result<Identity> {
    let mut binaries = Vec::new();
    for name in ["prototype2.exe", "prototype2engine.dll"] {
        let path = game.join(name);
        let mut f = File::open(&path)
            .with_context(|| format!("missing {name}; --game must point to Prototype 2"))?;
        let size = f.metadata()?.len();
        let mut prefix = [0u8; 4096];
        f.read_exact(&mut prefix)?;
        ensure!(&prefix[..2] == b"MZ", "invalid PE DOS signature");
        let pe = u32::from_le_bytes(prefix[60..64].try_into()?) as usize;
        ensure!(
            pe <= prefix.len() - 12 && &prefix[pe..pe + 4] == b"PE\0\0",
            "invalid PE header"
        );
        let machine = u16::from_le_bytes(prefix[pe + 4..pe + 6].try_into()?);
        let timestamp = u32::from_le_bytes(prefix[pe + 8..pe + 12].try_into()?);
        let mut hash = Sha256::new();
        hash.update(prefix);
        let mut buf = [0; 65536];
        loop {
            let n = f.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hash.update(&buf[..n]);
        }
        binaries.push(BinaryIdentity {
            name: name.to_owned(),
            size,
            sha256: format!("{:x}", hash.finalize()),
            pe_machine: format!("0x{machine:04x}"),
            pe_timestamp: timestamp,
        });
    }
    let steam_build_id = game
        .parent()
        .and_then(Path::parent)
        .and_then(|p| fs::read_to_string(p.join("appmanifest_115320.acf")).ok())
        .and_then(|s| {
            s.lines().find_map(|line| {
                let fields: Vec<_> = line.split('"').collect();
                (fields.get(1) == Some(&"buildid"))
                    .then(|| fields.get(3).map(|x| x.to_string()))
                    .flatten()
            })
        });
    Ok(Identity {
        steam_app_id: "115320",
        steam_build_id,
        binaries,
    })
}
