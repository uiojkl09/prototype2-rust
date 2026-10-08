use prototype2_rust::controller::{self, RawGamepad, Settings, Tracker};

fn slots(raw: RawGamepad) -> [Option<RawGamepad>; 4] {
    [Some(raw), None, None, None]
}

#[test]
fn idle_drift_is_removed_without_destroying_analog_control() {
    let zone = Settings::default().left_dead_zone;
    assert_eq!(controller::stick(-232, 159, zone), [0.0, 0.0]);
    let half = controller::stick(16384, 0, zone);
    assert!(half[0] > 0.3 && half[0] < 0.4);
    assert_eq!(half[1], 0.0);
    assert_eq!(controller::stick(i16::MIN, 0, zone), [-1.0, 0.0]);
    assert_eq!(controller::stick(i16::MAX, 0, zone), [1.0, 0.0]);
    let diagonal = controller::stick(i16::MAX, i16::MAX, zone);
    assert!((diagonal[0].hypot(diagonal[1]) - 1.0).abs() < 1e-6);
}

#[test]
fn toggles_require_new_presses_not_connection_or_focus_changes() {
    let mut tracker = Tracker::default();
    let settings = Settings::default();
    let held = RawGamepad {
        buttons: controller::X | controller::Y | controller::START | controller::A | controller::B,
        ..Default::default()
    };
    let first = tracker.sample(slots(held), true, settings);
    assert!(!first.toggle_collision && !first.reset && !first.exit);
    assert!(!first.next_clip && !first.previous_clip);
    tracker.sample(slots(RawGamepad::default()), true, settings);
    let pressed = tracker.sample(slots(held), true, settings);
    assert!(pressed.toggle_collision && pressed.reset && pressed.exit);
    assert!(pressed.next_clip && pressed.previous_clip);
    let repeat = tracker.sample(slots(held), true, settings);
    assert!(!repeat.toggle_collision && !repeat.reset && !repeat.exit);
    assert!(!repeat.next_clip && !repeat.previous_clip);
    tracker.sample(slots(held), false, settings);
    let regained = tracker.sample(slots(held), true, settings);
    assert!(!regained.toggle_collision && !regained.reset && !regained.exit);
    assert!(!regained.next_clip && !regained.previous_clip);
}

#[test]
fn focus_loss_and_disconnect_clear_motion_and_reconnection_is_safe() {
    let mut tracker = Tracker::default();
    let settings = Settings::default();
    let moving = RawGamepad {
        left_y: 20000,
        right_x: 20000,
        buttons: controller::A | controller::LEFT_SHOULDER,
        ..Default::default()
    };
    let input = tracker.sample(slots(moving), true, settings);
    assert!(input.travel[1] > 0.0 && input.travel[2] == 1.0 && input.look[0] < 0.0 && input.fast);
    let unfocused = tracker.sample(slots(moving), false, settings);
    assert_eq!(unfocused.travel, [0.0; 3]);
    assert_eq!(unfocused.look, [0.0; 2]);
    assert!(!unfocused.fast);
    let disconnected = tracker.sample([None; 4], true, settings);
    assert_eq!(disconnected.travel, [0.0; 3]);
    assert_eq!(tracker.slot(), None);
    let new = RawGamepad {
        buttons: controller::START,
        ..Default::default()
    };
    assert!(
        !tracker
            .sample([None, Some(new), None, None], true, settings)
            .exit
    );
    assert_eq!(tracker.slot(), Some(1));
}

#[test]
fn extra_controllers_do_not_steal_control_and_inversion_is_explicit() {
    let mut tracker = Tracker::default();
    let raw = RawGamepad {
        right_y: i16::MAX,
        ..Default::default()
    };
    tracker.sample([None, Some(raw), None, None], true, Settings::default());
    let inverted = Settings {
        invert_y: true,
        ..Settings::default()
    };
    let input = tracker.sample(
        [Some(RawGamepad::default()), Some(raw), None, None],
        true,
        inverted,
    );
    assert_eq!(tracker.slot(), Some(1));
    assert_eq!(input.look[1], -2.0);
    assert!(
        !Settings {
            look_speed: f32::NAN,
            ..inverted
        }
        .valid()
    );
    assert_eq!(std::mem::size_of::<RawGamepad>(), 12);
    assert_eq!(std::mem::align_of::<RawGamepad>(), 2);
}
