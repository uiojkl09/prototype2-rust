use prototype2_rust::fight::{self, MAX_DEPTH, name_hash};

fn u32(out: &mut Vec<u8>, value: u32) {
    out.extend(value.to_le_bytes());
}
fn u64(out: &mut Vec<u8>, value: u64) {
    out.extend(value.to_le_bytes());
}
fn string(out: &mut Vec<u8>, value: &str) {
    u32(out, value.len() as u32);
    out.extend(value.as_bytes());
    out.resize(out.len() + (4 - value.len() % 4) % 4, 0);
}
fn record(key: &str, bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    u64(&mut out, name_hash(key));
    u32(&mut out, bytes.len() as u32);
    out.extend(bytes);
    u64(&mut out, 0);
    out
}
fn branch(children: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    u64(&mut out, name_hash("test"));
    string(&mut out, "test");
    u32(&mut out, 1);
    out.extend(children);
    record("node", &out)
}
fn graph(children: &[u8], count: u32) -> Vec<u8> {
    let mut body = Vec::new();
    u32(&mut body, 1);
    u64(&mut body, name_hash("synthetic"));
    u64(&mut body, name_hash("Motion"));
    string(&mut body, "/");
    u32(&mut body, u32::MAX);
    u32(&mut body, count);
    body.extend(children);
    u64(&mut body, 0);
    let mut out = b"fig0".to_vec();
    u32(&mut out, 5);
    u32(&mut out, 0);
    out.extend(record("chunk", &body));
    out
}
#[test]
fn bounded_unknown_actions_keep_their_identity_and_provenance() {
    let action = record("unknownSyntheticAction", &[1, 2, 3, 4, 5]);
    let mut group = Vec::new();
    u32(&mut group, 1);
    group.extend(action);
    let bytes = graph(&branch(&record("tracks", &group)), 2);
    let mut decoded = fight::parse_body(&bytes, 100, "synthetic").unwrap();
    assert_eq!(decoded.branches.len(), 1);
    assert_eq!(decoded.branches[0].derived_path, "test");
    let action = &decoded.records[0];
    assert_eq!(action.type_hash, name_hash("unknownSyntheticAction"));
    assert_eq!(action.body_bytes, 5);
    assert_eq!(
        &bytes[action.body_offset - 100..action.body_offset - 95],
        &[1, 2, 3, 4, 5]
    );
    assert!(action.capsule.is_none() && action.steer.is_none());
    decoded.branches[0].parent = Some(0);
    assert!(!decoded.branch_matches(0, "absent"));
    decoded.branches[0].parent = Some(999);
    assert!(!decoded.branch_matches(0, "absent"));
}
#[test]
fn truncated_ranges_trailers_counts_and_unknown_branches_fail_closed() {
    let bytes = graph(&branch(&[]), 2);
    for len in 0..bytes.len() {
        assert!(
            fight::parse_body(&bytes[..len], 0, "synthetic").is_err(),
            "accepted prefix {len}"
        );
    }
    let mut bad = bytes.clone();
    *bad.last_mut().unwrap() = 1;
    assert!(fight::parse_body(&bad, 0, "synthetic").is_err());
    assert!(fight::parse_body(&graph(&branch(&[]), 3), 0, "synthetic").is_err());
    assert!(
        fight::parse_body(
            &graph(&record("unknownBranch", &[0; 16]), 2),
            0,
            "synthetic"
        )
        .is_err()
    );
    let mut properties = Vec::new();
    u32(&mut properties, 2);
    properties.extend(record("unknownAction", &[0; 4]));
    assert!(
        fight::parse_body(
            &graph(&branch(&record("tracks", &properties)), 2),
            0,
            "synthetic"
        )
        .is_err()
    );
}
#[test]
fn depth_and_declared_limits_reject_resource_exhaustion() {
    let mut nested = branch(&[]);
    for _ in 0..=MAX_DEPTH {
        nested = branch(&nested);
    }
    assert!(fight::parse_body(&graph(&nested, MAX_DEPTH as u32 + 3), 0, "synthetic").is_err());
    assert!(fight::parse_body(&graph(&[], u32::MAX), 0, "synthetic").is_err());
    assert!(fight::parse_body(&graph(&[], 1), usize::MAX, "synthetic").is_err());
}
#[test]
fn capsule_track_requires_boolean_flags_and_finite_fields() {
    let mut bytes = vec![0; 136];
    bytes[..4].copy_from_slice(&(-1i32).to_le_bytes());
    bytes[12..16].copy_from_slice(&(-1f32).to_le_bytes());
    assert!(fight::capsule_track(&bytes).is_ok());
    bytes[24..28].copy_from_slice(&2u32.to_le_bytes());
    assert!(fight::capsule_track(&bytes).is_err());
    bytes[24..28].fill(0);
    bytes[32..36].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(fight::capsule_track(&bytes).is_err());
    let mut sprint = vec![0; 208];
    sprint[..4].copy_from_slice(&(-1i32).to_le_bytes());
    assert!(fight::sprint_track(&sprint).is_ok());
    sprint[164..168].copy_from_slice(&2u32.to_le_bytes());
    assert!(fight::sprint_track(&sprint).is_err());
    assert!(fight::sprint_track(&sprint[..207]).is_err());
}
