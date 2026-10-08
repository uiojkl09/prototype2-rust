use prototype2_rust::{meta, p3d};

fn chunk(id: u32, payload: &[u8], children: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for n in [
        id,
        12 + payload.len() as u32,
        12 + payload.len() as u32 + children.len() as u32,
    ] {
        out.extend(n.to_le_bytes());
    }
    out.extend(payload);
    out.extend(children);
    out
}
fn fixture(declared_length: u32) -> Vec<u8> {
    let mut data = declared_length.to_le_bytes().to_vec();
    data.extend(b"METAExampleFactory");
    let child = chunk(0x07f00001, &data, &[]);
    let mut def = Vec::new();
    for name in ["example:body", "body", "ExampleType"] {
        def.push(name.len() as u8);
        def.extend(name.as_bytes());
    }
    def.extend([1, 0, 0, 0]);
    def.extend(123_u32.to_le_bytes());
    chunk(p3d::MAGIC, &[], &chunk(0x07f00000, &def, &child))
}
#[test]
fn metadata_search_locates_references_without_guessing_properties() {
    let data = fixture(18);
    let chunks = p3d::parse(&data).unwrap();
    let found = meta::inspect(&data, &chunks, "examplefactory").unwrap();
    assert_eq!(found.len(), 1);
    let object = &found[0];
    assert_eq!(object.short_name, "body");
    assert_eq!(object.unknown_u32, 123);
    assert!(object.has_meta_signature);
    assert_eq!(object.matched_reference_offsets, [object.body_offset + 4]);
    assert!(
        meta::inspect(&data, &chunks, "unrelated")
            .unwrap()
            .is_empty()
    );
    for bad_length in [17, 19, u32::MAX] {
        let malformed = fixture(bad_length);
        let chunks = p3d::parse(&malformed).unwrap();
        assert!(meta::inspect(&malformed, &chunks, "").is_err());
    }
}
