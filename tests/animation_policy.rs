use prototype2_rust::animation_policy::{CyclePolicy, SyncPhasePolicy};

#[test]
fn original_enum_hashes_preserve_order_and_unknown_values_fail() {
    use CyclePolicy::*;
    let expected = [
        (FromAnimation, 0xa43298b89a5bf3c6),
        (NotCyclic, 0x8b3e16d80559e678),
        (Cyclic, 0x25f64ae2e13c4a85),
        (HoldEndFrame, 0x1b729cb6529f2e87),
        (NotCyclicBackward, 0xe429a83e9b02b748),
        (CyclicBackward, 0xbe01ae4856d01e7b),
        (EndEarlyForTimeSlicing, 0xb13765344a10b881),
    ];
    for (index, (policy, hash)) in expected.into_iter().enumerate() {
        assert_eq!(CyclePolicy::from_hash(hash).unwrap(), policy);
        assert_eq!(policy.native_index(), index as u8);
    }
    assert_eq!(
        SyncPhasePolicy::from_hash(0x2e7c5ad2600aeb45).unwrap(),
        SyncPhasePolicy::Legacy
    );
    assert_eq!(
        SyncPhasePolicy::from_hash(0xd907d85ac26f2f87).unwrap(),
        SyncPhasePolicy::FromPuppetPhase
    );
    assert_eq!(SyncPhasePolicy::Legacy.native_index(), 0);
    assert_eq!(SyncPhasePolicy::FromPuppetPhase.native_index(), 1);
    for hash in [
        0,
        u64::MAX,
        prototype2_rust::fight::name_hash("cyclic"),
        0x2e7c5ad2600aeb45,
    ] {
        assert!(CyclePolicy::from_hash(hash).is_err());
    }
    assert!(SyncPhasePolicy::from_hash(0xa43298b89a5bf3c6).is_err());
}
#[test]
fn driver_flags_use_asset_cyclic_only_for_from_animation() {
    use CyclePolicy::*;
    for asset in [false, true] {
        assert_eq!(FromAnimation.driver_flags(asset).cyclic, asset);
        for policy in [Cyclic, CyclicBackward] {
            let flags = policy.driver_flags(asset);
            assert!(flags.cyclic && !flags.hold_end_frame);
        }
        for policy in [NotCyclic, NotCyclicBackward, EndEarlyForTimeSlicing] {
            let flags = policy.driver_flags(asset);
            assert!(!flags.cyclic && !flags.hold_end_frame);
        }
        let flags = HoldEndFrame.driver_flags(asset);
        assert!(!flags.cyclic && flags.hold_end_frame);
    }
}
