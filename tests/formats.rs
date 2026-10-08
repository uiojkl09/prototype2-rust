use flate2::{Compression, write::ZlibEncoder};
use prototype2_rust::{
    p3d,
    rcf::{Archive, name_hash},
    scene::{CollisionMesh, raycast},
};
use std::{
    io::Write,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

fn root(children: &[u8]) -> Vec<u8> {
    let mut b = Vec::new();
    for x in [p3d::MAGIC, 12, 12 + children.len() as u32] {
        b.extend(x.to_le_bytes());
    }
    b.extend(children);
    b
}
fn node(id: u32, payload: &[u8], children: &[u8]) -> Vec<u8> {
    let mut b = Vec::new();
    for x in [
        id,
        12 + payload.len() as u32,
        12 + payload.len() as u32 + children.len() as u32,
    ] {
        b.extend(x.to_le_bytes());
    }
    b.extend(payload);
    b.extend(children);
    b
}
fn rz(data: &[u8]) -> Vec<u8> {
    let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
    z.write_all(data).unwrap();
    let mut b = b"RZ\0\0\0\0\0\0".to_vec();
    b.extend((data.len() as u32).to_le_bytes());
    b.extend(0u32.to_le_bytes());
    b.extend(z.finish().unwrap());
    b
}
#[test]
fn retail_hash_includes_punctuation_transform() {
    assert_eq!(name_hash("A0\\B"), 2_970_549);
    assert_eq!(name_hash("\\A0\\B"), 2_970_549);
    assert_ne!(name_hash("A0\\B"), name_hash("A0/B"));
}
#[test]
fn p3d_tracks_parents_and_siblings_without_losing_order() {
    let a = node(1, b"data", &node(2, &[], &[]));
    let b = node(3, &[], &[]);
    let d = root(&[a, b].concat());
    let c = p3d::parse(&d).unwrap();
    assert_eq!(
        c.iter().map(|x| x.id).collect::<Vec<_>>(),
        vec![p3d::MAGIC, 1, 2, 3]
    );
    assert_eq!(
        c.iter().map(|x| x.parent).collect::<Vec<_>>(),
        vec![None, Some(0), Some(1), Some(0)]
    );
}
#[test]
fn p3d_rejects_zero_size_truncation_and_excess_nesting() {
    let valid = root(&node(1, &[], &[]));
    for n in 0..valid.len() {
        assert!(p3d::parse(&valid[..n]).is_err());
    }
    let mut bad = valid.clone();
    bad[16..20].copy_from_slice(&0u32.to_le_bytes());
    assert!(p3d::parse(&bad).is_err());
    let mut child = Vec::new();
    for _ in 0..130 {
        child = node(1, &[], &child);
    }
    assert!(p3d::parse(&root(&child)).is_err());
}
#[test]
fn rz_checks_size_checksum_and_trailing_data() {
    let d = root(&node(1, b"synthetic fixture", &[]));
    let encoded = rz(&d);
    assert_eq!(p3d::decode(&encoded).unwrap(), d);
    let mut bad = encoded.clone();
    bad[8..12].copy_from_slice(&1u32.to_le_bytes());
    assert!(p3d::decode(&bad).is_err());
    let mut bad = encoded.clone();
    let last = bad.len() - 1;
    bad[last] ^= 1;
    assert!(p3d::decode(&bad).is_err());
    let mut bad = encoded.clone();
    bad.push(0);
    assert!(p3d::decode(&bad).is_err());
    let mut bad = encoded;
    bad[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(p3d::decode(&bad).is_err());
}
struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn temporary(bytes: &[u8]) -> Temp {
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "prototype2-rust-synthetic-{}-{}.rcf",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(&p, bytes).unwrap();
    Temp(p)
}
fn archive() -> Vec<u8> {
    let mut b = vec![0; 2052];
    b[..23].copy_from_slice(b"ATG CORE CEMENT LIBRARY");
    b[32..36].copy_from_slice(&[2, 1, 0, 1]);
    let values = [60, 12, 128, 37, 0, 1, name_hash("Test0.bin"), 2048, 4];
    for (i, v) in values.iter().enumerate() {
        b[36 + i * 4..40 + i * 4].copy_from_slice(&v.to_le_bytes());
    }
    b[128..132].copy_from_slice(&2048u32.to_le_bytes());
    b[140..144].copy_from_slice(&2048u32.to_le_bytes());
    b[148..152].copy_from_slice(&10u32.to_le_bytes());
    b[152..162].copy_from_slice(b"Test0.bin\0");
    b[2048..2052].copy_from_slice(b"test");
    b
}
#[test]
fn rcf_resolves_name_hash_and_reads_only_selected_payload() {
    let file = temporary(&archive());
    let a = Archive::open(&file.0).unwrap();
    assert_eq!(a.entries.len(), 1);
    assert_eq!(a.read(a.find("test0.bin").unwrap()).unwrap(), b"test");
}
#[test]
fn rcf_rejects_wrong_hash_out_of_bounds_and_bad_metadata() {
    for (offset, value) in [
        (60, 0),
        (64, 2049),
        (68, u32::MAX),
        (148, u32::MAX),
        (36, u32::MAX),
        (44, u32::MAX),
    ] {
        let mut b = archive();
        b[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        let file = temporary(&b);
        assert!(Archive::open(&file.0).is_err(), "offset {offset}");
    }
    let mut b = archive();
    b[161] = b'X';
    assert!(Archive::open(&temporary(&b).0).is_err());
}
#[test]
fn geometric_collision_picks_nearest_surface_and_accepts_both_windings() {
    let m = CollisionMesh {
        chunk_offset: 0,
        positions: vec![
            [0., 0., 0.],
            [1., 0., 0.],
            [0., 0., 1.],
            [0., -2., 0.],
            [1., -2., 0.],
            [0., -2., 1.],
        ],
        faces: vec![[0, 1, 2, 7], [3, 5, 4, 0]],
        unknown_tail_bytes: 0,
    };
    assert_eq!(
        raycast(std::slice::from_ref(&m), [0.2, 1., 0.2], [0., -1., 0.]),
        Some(1.)
    );
    assert_eq!(
        raycast(std::slice::from_ref(&m), [0.2, -1., 0.2], [0., -1., 0.]),
        Some(1.)
    );
    assert_eq!(raycast(&[m], [2., 1., 2.], [0., -1., 0.]), None);
}
