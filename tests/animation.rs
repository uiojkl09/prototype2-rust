use flate2::{Compression, write::ZlibEncoder};
use glam::{Mat4, Vec3};
use prototype2_rust::{
    animation::{self, Joint, Skeleton},
    p3d,
    skin::SkinMesh,
};
use std::io::Write;
fn node(id: u32, payload: &[u8], children: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for x in [
        id,
        12 + payload.len() as u32,
        12 + payload.len() as u32 + children.len() as u32,
    ] {
        out.extend(x.to_le_bytes());
    }
    out.extend(payload);
    out.extend(children);
    out
}
fn string(out: &mut Vec<u8>, s: &str) {
    out.push(s.len() as u8);
    out.extend(s.as_bytes());
}
fn fixture(referenced: bool) -> Vec<u8> {
    let mut header = 0u32.to_le_bytes().to_vec();
    string(&mut header, "synthetic");
    header.extend(b"PTRN");
    header.extend(11f32.to_le_bytes());
    header.extend(30f32.to_le_bytes());
    header.extend(1u32.to_le_bytes());
    let mut values = Vec::new();
    for f in [0u16, 10] {
        values.extend(f.to_le_bytes());
    }
    for xyz in [[0i16; 3], [0, 0, 32767]] {
        for v in xyz {
            values.extend(v.to_le_bytes());
        }
    }
    let mut channel = 1u32.to_le_bytes().to_vec();
    channel.extend(b"ROT\0");
    channel.extend(if referenced { 0u32 } else { 2u32 }.to_le_bytes());
    let mut children = Vec::new();
    let mut clip_children = Vec::new();
    if referenced {
        let mut reference = Vec::new();
        for v in [0u32, 2, 0] {
            reference.extend(v.to_le_bytes());
        }
        reference.extend(0u16.to_le_bytes());
        children = node(0x121121, &reference, &[]);
        let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
        z.write_all(&values).unwrap();
        let compressed = z.finish().unwrap();
        let mut table = Vec::new();
        for v in [0u32, 8192, 1, values.len() as u32] {
            table.extend(v.to_le_bytes());
        }
        clip_children.extend(node(0x121006, &[], &node(0x121010, &table, &[])));
        let mut blob = 0u32.to_le_bytes().to_vec();
        blob.extend(b"ZLIB");
        blob.extend((values.len() as u32).to_le_bytes());
        blob.extend((compressed.len() as u32).to_le_bytes());
        blob.extend(compressed);
        clip_children.extend(node(0x2f00000, &blob, &[]));
    } else {
        channel.extend(values);
    }
    let mut group = 0u32.to_le_bytes().to_vec();
    string(&mut group, "tip");
    group.extend(1u32.to_le_bytes());
    group.extend(1u32.to_le_bytes());
    clip_children.extend(node(
        0x121002,
        &[],
        &node(0x121001, &group, &node(0x121112, &channel, &children)),
    ));
    node(p3d::MAGIC, &[], &node(0x121000, &header, &clip_children))
}
fn skeleton() -> Skeleton {
    let tip = Mat4::from_translation(Vec3::X);
    Skeleton {
        name: "synthetic".into(),
        joints: vec![
            Joint {
                name: "root".into(),
                parent: None,
                bind_local: Mat4::IDENTITY,
            },
            Joint {
                name: "tip".into(),
                parent: Some(0),
                bind_local: tip,
            },
        ],
        bind_world: vec![Mat4::IDENTITY, tip],
        inverse_bind: vec![Mat4::IDENTITY, tip.inverse()],
    }
}
fn clip_with_channels(template: &[u8], channels: &[Vec<u8>]) -> Vec<u8> {
    let chunks = p3d::parse(template).unwrap();
    let clip = chunks.iter().find(|n| n.id == 0x121000).unwrap();
    let wrapper = chunks.iter().find(|n| n.id == 0x121002).unwrap();
    let group = chunks.iter().find(|n| n.id == 0x121001).unwrap();
    let mut payload = group.payload(template).to_vec();
    let len = payload.len();
    payload[len - 4..].copy_from_slice(&(channels.len() as u32).to_le_bytes());
    let mut children = template[clip.offset + clip.data_size..wrapper.offset].to_vec();
    children.extend(node(
        0x121002,
        &[],
        &node(0x121001, &payload, &channels.concat()),
    ));
    node(
        p3d::MAGIC,
        &[],
        &node(0x121000, clip.payload(template), &children),
    )
}
fn scale_channel(id: u32) -> Vec<u8> {
    let mut payload = 0u32.to_le_bytes().to_vec();
    payload.extend(b"SCL\0");
    if matches!(id, 0x121102 | 0x121103 | 0x121118) {
        payload.extend(0u16.to_le_bytes()); // X is authored for scalar, constant for 2D.
        for value in [1f32, 1., 1.] {
            payload.extend(value.to_le_bytes());
        }
    }
    payload.extend(2u32.to_le_bytes());
    for frame in [0u16, 10] {
        payload.extend(frame.to_le_bytes());
    }
    match id {
        0x121119 => {
            for bits in [0x3c00u16, 0x3c00, 0x3c00, 0x4200, 0x3c00, 0x3c00] {
                payload.extend(bits.to_le_bytes());
            }
        }
        0x121104 => {
            for value in [1f32, 1., 1., 3., 1., 1.] {
                payload.extend(value.to_le_bytes());
            }
        }
        0x121102 => {
            for value in [1f32, 3.] {
                payload.extend(value.to_le_bytes());
            }
        }
        0x121118 => {
            for bits in [0x3c00u16, 0x3c00, 0x4200, 0x4500] {
                payload.extend(bits.to_le_bytes());
            }
        }
        0x121103 => {
            for value in [1f32, 1., 3., 5.] {
                payload.extend(value.to_le_bytes());
            }
        }
        _ => unreachable!(),
    }
    node(id, &payload, &[])
}
#[test]
fn authored_scale_changes_the_rotating_skin_and_absence_keeps_bind_scale() {
    let template = fixture(false);
    let chunks = p3d::parse(&template).unwrap();
    let rotation = chunks.iter().find(|n| n.id == 0x121112).unwrap();
    let rotation = template[rotation.offset..rotation.offset + rotation.total_size].to_vec();
    let rig = skeleton();
    let skin = SkinMesh {
        name: "tip".into(),
        skeleton: "synthetic".into(),
        positions: vec![[2., 0., 0.]],
        normals: vec![[1., 0., 0.]],
        weights: vec![[1., 0., 0., 0.]],
        joints: vec![[1; 4]],
        indices: vec![],
    };
    for id in [0x121119, 0x121104, 0x121102, 0x121118, 0x121103] {
        let data = clip_with_channels(&template, &[rotation.clone(), scale_channel(id)]);
        let clip = animation::load_clip(&data, &p3d::parse(&data).unwrap(), "synthetic").unwrap();
        let expected_scale = if matches!(id, 0x121118 | 0x121103) {
            Vec3::new(1., 2., 3.)
        } else {
            Vec3::new(2., 1., 1.)
        };
        assert_eq!(
            clip.sample_local(&rig, 5.).unwrap()[1].scale,
            expected_scale
        );
        let pose = skin.deform(&rig, &clip.sample(&rig, 5.).unwrap()).unwrap();
        assert!(
            (Vec3::from_array(pose.positions[0]) - Vec3::new(1., expected_scale.x, 0.)).length()
                < 1e-5
        );
        assert_eq!(clip.sample_local(&rig, -10.).unwrap()[1].scale, Vec3::ONE);
        let mut bad = clip.clone();
        bad.tracks
            .get_mut("tip")
            .unwrap()
            .scale
            .as_mut()
            .unwrap()
            .values[0]
            .x = f32::NAN;
        assert!(bad.sample(&rig, 5.).is_err());
    }
    let mut scaled_rig = rig;
    scaled_rig.joints[1].bind_local =
        Mat4::from_scale_rotation_translation(Vec3::new(4., 5., 6.), glam::Quat::IDENTITY, Vec3::X);
    let clip = animation::load_clip(&template, &chunks, "synthetic").unwrap();
    assert_eq!(
        clip.sample_local(&scaled_rig, 5.).unwrap()[1].scale,
        Vec3::new(4., 5., 6.)
    );
    assert!(clip.tracks["tip"].sample(5.).unwrap().scale.is_none());
}
#[test]
fn scale_encoding_rejects_duplicates_nonfinite_axes_versions_and_truncation() {
    let template = fixture(false);
    let channel = scale_channel(0x121119);
    let data = clip_with_channels(&template, &[channel.clone(), channel.clone()]);
    assert!(animation::load_clip(&data, &p3d::parse(&data).unwrap(), "synthetic").is_err());
    for (id, relative, bytes) in [
        (0x121119, 0, 1u32.to_le_bytes().to_vec()),
        (0x121119, 16, 0x7c00u16.to_le_bytes().to_vec()),
        (0x121102, 8, 3u16.to_le_bytes().to_vec()),
        (0x121118, 8, 3u16.to_le_bytes().to_vec()),
        (0x121104, 16, f32::INFINITY.to_le_bytes().to_vec()),
    ] {
        let mut channel = scale_channel(id);
        channel[12 + relative..12 + relative + bytes.len()].copy_from_slice(&bytes);
        let data = clip_with_channels(&template, &[channel]);
        assert!(animation::load_clip(&data, &p3d::parse(&data).unwrap(), "synthetic").is_err());
    }
    let data = clip_with_channels(&template, &[channel]);
    let chunks = p3d::parse(&data).unwrap();
    let index = chunks.iter().position(|n| n.id == 0x121119).unwrap();
    for len in 0..chunks[index].data_size - 12 {
        let mut short = chunks.clone();
        short[index].data_size = 12 + len;
        assert!(animation::load_clip(&data, &short, "synthetic").is_err());
    }
}
#[test]
fn disabled_channel_metadata_preserves_keys_and_rejects_unimplemented_flags() {
    for referenced in [false, true] {
        let template = fixture(referenced);
        let chunks = p3d::parse(&template).unwrap();
        let channel = chunks.iter().find(|n| n.id == 0x121112).unwrap();
        let original_children =
            &template[channel.offset + channel.data_size..channel.offset + channel.total_size];
        let with_metadata = |metadata: &[u8]| {
            let children = [original_children, metadata].concat();
            clip_with_channels(
                &template,
                &[node(0x121112, channel.payload(&template), &children)],
            )
        };
        let empty = node(0x121110, &[0; 8], &[]);
        let data = with_metadata(&empty);
        let clip = animation::load_clip(&data, &p3d::parse(&data).unwrap(), "synthetic").unwrap();
        assert!(
            (clip.sample(&skeleton(), 5.).unwrap()[1].transform_vector3(Vec3::X) - Vec3::Y)
                .length()
                < 1e-5
        );
        let mut invalid = vec![
            node(0x121110, &[1, 0, 0, 0, 0, 0, 0, 0], &[]),
            node(0x121110, &[0, 0, 0, 0, 1, 0, 0, 0], &[]),
            node(0x121110, &[0; 8], &node(0x121101, &[0; 8], &[])),
            [empty.clone(), empty].concat(),
        ];
        for len in (0..8).chain([9]) {
            invalid.push(node(0x121110, &vec![0; len], &[]));
        }
        for metadata in invalid {
            let data = with_metadata(&metadata);
            assert!(animation::load_clip(&data, &p3d::parse(&data).unwrap(), "synthetic").is_err());
        }
    }
}
#[test]
fn sync_frame_child_is_checked_and_absence_uses_native_zero_default() {
    let data = fixture(false);
    let chunks = p3d::parse(&data).unwrap();
    assert_eq!(
        animation::clips(&data, &chunks).unwrap()[0].default_sync_frame,
        0.
    );
    let clip = chunks.iter().find(|n| n.id == 0x121000).unwrap();
    let with_sync = |extra: &[u8]| {
        let mut children =
            data[clip.offset + clip.data_size..clip.offset + clip.total_size].to_vec();
        children.extend(extra);
        node(
            p3d::MAGIC,
            &[],
            &node(0x121000, clip.payload(&data), &children),
        )
    };
    let payload = [0u32.to_le_bytes(), 3.5f32.to_le_bytes()].concat();
    let child = node(0x121402, &payload, &[]);
    let good = with_sync(&child);
    assert_eq!(
        animation::clips(&good, &p3d::parse(&good).unwrap()).unwrap()[0].default_sync_frame,
        3.5
    );
    for payload in [
        [1u32.to_le_bytes(), 0f32.to_le_bytes()].concat(),
        [0u32.to_le_bytes(), f32::NAN.to_le_bytes()].concat(),
        vec![0; 7],
        vec![0; 9],
    ] {
        let bad = with_sync(&node(0x121402, &payload, &[]));
        assert!(animation::clips(&bad, &p3d::parse(&bad).unwrap()).is_err());
    }
    let bad = with_sync(&[child.clone(), child].concat());
    assert!(animation::clips(&bad, &p3d::parse(&bad).unwrap()).is_err());
}
#[test]
fn packed_clip_and_blob_produce_known_rotating_skin_pose() {
    for referenced in [false, true] {
        let data = fixture(referenced);
        let nodes = p3d::parse(&data).unwrap();
        let clip = animation::load_clip(&data, &nodes, "synthetic").unwrap();
        assert_eq!(clip.info.frame_count, 11.);
        assert_eq!(clip.info.last_frame(), 10.);
        let skeleton = skeleton();
        let skin = SkinMesh {
            name: "tip".into(),
            skeleton: "synthetic".into(),
            positions: vec![[2., 0., 0.]],
            normals: vec![[1., 0., 0.]],
            weights: vec![[1., 0., 0., 0.]],
            joints: vec![[1; 4]],
            indices: vec![],
        };
        let half = skin
            .deform(&skeleton, &clip.sample(&skeleton, 5.).unwrap())
            .unwrap();
        assert!(
            (half.positions[0][0] - 1.).abs() < 1e-5 && (half.positions[0][1] - 1.).abs() < 1e-5
        );
        assert!(half.normals[0][0].abs() < 1e-5 && (half.normals[0][1] - 1.).abs() < 1e-5);
        let final_pose = skin
            .deform(&skeleton, &clip.sample(&skeleton, 100.).unwrap())
            .unwrap();
        assert!(final_pose.positions[0].iter().all(|v| v.abs() < 1e-5));
        assert!(clip.sample(&skeleton, f32::NAN).is_err());
        let mut bad = clip.clone();
        bad.tracks
            .get_mut("tip")
            .unwrap()
            .rotation
            .as_mut()
            .unwrap()
            .frames
            .clear();
        assert!(bad.sample(&skeleton, 0.).is_err());
    }
}
#[test]
fn animation_rejects_reference_counts_offsets_and_truncated_keys() {
    let good = fixture(true);
    let nodes = p3d::parse(&good).unwrap();
    let reference = nodes.iter().find(|n| n.id == 0x121121).unwrap();
    for (relative, value) in [(4, 0u32), (4, 1000000), (8, u32::MAX)] {
        let mut bad = good.clone();
        let p = reference.offset + 12 + relative;
        bad[p..p + 4].copy_from_slice(&value.to_le_bytes());
        assert!(animation::load_clip(&bad, &nodes, "synthetic").is_err());
    }
    let mut bad = good.clone();
    let p = reference.offset + 24;
    bad[p..p + 2].copy_from_slice(&256u16.to_le_bytes());
    assert!(animation::load_clip(&bad, &nodes, "synthetic").is_err());
    let inline = fixture(false);
    let nodes = p3d::parse(&inline).unwrap();
    let channel = nodes.iter().find(|n| n.id == 0x121112).unwrap();
    for len in 0..channel.data_size - 12 {
        let mut nodes = nodes.clone();
        nodes
            .iter_mut()
            .find(|n| n.id == 0x121112)
            .unwrap()
            .data_size = 12 + len;
        assert!(animation::load_clip(&inline, &nodes, "synthetic").is_err());
    }
    let mut bad = inline.clone();
    let p = channel.offset + 24;
    bad[p..p + 4].copy_from_slice(&[10, 0, 0, 0]);
    assert!(animation::load_clip(&bad, &nodes, "synthetic").is_err());
}
#[test]
fn half_expansion_handles_sign_subnormal_and_nonfinite_encodings() {
    assert_eq!(animation::half(0x3c00), 1.);
    assert_eq!(animation::half(0xc000), -2.);
    assert_eq!(animation::half(1), 2f32.powi(-24));
    assert_eq!(animation::half(0x8000).to_bits(), (-0f32).to_bits());
    assert!(animation::half(0x7c00).is_infinite());
    assert!(animation::half(0x7e00).is_nan());
}
#[test]
fn invalid_joint_order_and_skin_influences_fail_without_panicking() {
    let mut invalid = skeleton();
    invalid.joints[1].parent = Some(1);
    assert!(animation::world_matrices(&invalid.joints, &invalid.bind_world).is_err());
    let skeleton = skeleton();
    let skin = SkinMesh {
        name: "bad".into(),
        skeleton: "synthetic".into(),
        positions: vec![[0.; 3]],
        normals: vec![[0., 1., 0.]],
        weights: vec![[1., 0., 0., 0.]],
        joints: vec![[100; 4]],
        indices: vec![],
    };
    assert!(skin.deform(&skeleton, &skeleton.bind_world).is_err());
}

#[test]
fn skeleton_reader_checks_version_parents_counts_and_complete_joint_payloads() {
    let mut header = Vec::new();
    string(&mut header, "synthetic");
    for v in [1u32, 2, 0, 0] {
        header.extend(v.to_le_bytes());
    }
    let mut children = Vec::new();
    for (name, matrix) in [
        ("root", Mat4::IDENTITY),
        ("tip", Mat4::from_translation(Vec3::X)),
    ] {
        let mut joint = Vec::new();
        string(&mut joint, name);
        joint.extend(0u32.to_le_bytes());
        for value in matrix.to_cols_array() {
            joint.extend(value.to_le_bytes());
        }
        joint.extend([0; 48]);
        joint.extend(0u16.to_le_bytes());
        joint.extend(0u32.to_le_bytes());
        children.extend(node(0x23001, &joint, &[]));
    }
    let good = node(p3d::MAGIC, &[], &node(0x23000, &header, &children));
    let nodes = p3d::parse(&good).unwrap();
    let result = animation::skeleton(&good, &nodes, "synthetic").unwrap();
    assert_eq!(result.joints.len(), 2);
    assert_eq!(result.bind_world[1].transform_point3(Vec3::ZERO), Vec3::X);
    let root = nodes.iter().find(|n| n.id == 0x23000).unwrap();
    let joints: Vec<_> = nodes.iter().filter(|n| n.id == 0x23001).collect();
    for (offset, value) in [
        (root.offset + 12 + 10, 2u32),
        (root.offset + 12 + 14, 4097),
        (joints[0].offset + 12 + 5, 1),
        (joints[1].offset + 12 + 4, 1),
    ] {
        let mut bad = good.clone();
        bad[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(animation::skeleton(&bad, &nodes, "synthetic").is_err());
    }
    for len in 0..joints[1].data_size - 12 {
        let mut truncated = nodes.clone();
        truncated
            .iter_mut()
            .find(|n| n.offset == joints[1].offset)
            .unwrap()
            .data_size = 12 + len;
        assert!(animation::skeleton(&good, &truncated, "synthetic").is_err());
    }
}

#[test]
fn animation_references_resolve_exact_names_and_reject_missing_or_ambiguous_hashes() {
    let data = fixture(false);
    let nodes = p3d::parse(&data).unwrap();
    let clips = animation::clips(&data, &nodes).unwrap();
    assert_eq!(
        animation::resolve_clip(&clips, prototype2_rust::fight::name_hash("synthetic"))
            .unwrap()
            .name,
        "synthetic"
    );
    assert!(
        animation::resolve_clip(&clips, prototype2_rust::fight::name_hash("Synthetic")).is_err()
    );
    let duplicates = vec![clips[0].clone(), clips[0].clone()];
    assert!(
        animation::resolve_clip(&duplicates, prototype2_rust::fight::name_hash("synthetic"))
            .is_err()
    );
}
