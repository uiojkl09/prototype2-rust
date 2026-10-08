//! Native Windows Xbox input and renderer-independent inspection controls.
//! Inspection dead zones and speeds are preferences, not recovered retail rules.
use serde::Serialize;

pub const A: u16 = 0x1000;
pub const B: u16 = 0x2000;
pub const X: u16 = 0x4000;
pub const Y: u16 = 0x8000;
pub const LEFT_SHOULDER: u16 = 0x0100;
pub const START: u16 = 0x0010;

/// Layout matches XINPUT_GAMEPAD, including signed thumbstick endpoints.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct RawGamepad {
    pub buttons: u16,
    pub left_trigger: u8,
    pub right_trigger: u8,
    pub left_x: i16,
    pub left_y: i16,
    pub right_x: i16,
    pub right_y: i16,
}

#[cfg(target_os = "windows")]
pub fn poll(slot: u32) -> Option<RawGamepad> {
    #[repr(C)]
    #[derive(Default)]
    struct State {
        packet: u32,
        gamepad: RawGamepad,
    }
    #[link(name = "xinput")]
    unsafe extern "system" {
        fn XInputGetState(user_index: u32, state: *mut State) -> u32;
    }
    if slot >= 4 {
        return None;
    }
    let mut state = State::default();
    // SAFETY: C-layout state has the documented DWORD + 12-byte gamepad layout;
    // its initialized storage is writable for the synchronous OS call. Slot is 0..4.
    let result = unsafe { XInputGetState(slot, &mut state) };
    (result == 0).then_some(state.gamepad)
}

#[cfg(not(target_os = "windows"))]
pub fn poll(_slot: u32) -> Option<RawGamepad> {
    None
}

pub fn connected() -> [Option<RawGamepad>; 4] {
    std::array::from_fn(|slot| poll(slot as u32))
}

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub left_dead_zone: f32,
    pub right_dead_zone: f32,
    pub look_speed: f32,
    pub invert_y: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            left_dead_zone: 7849.0 / 32767.0,
            right_dead_zone: 8689.0 / 32767.0,
            look_speed: 2.0,
            invert_y: false,
        }
    }
}
impl Settings {
    pub fn valid(&self) -> bool {
        [self.left_dead_zone, self.right_dead_zone]
            .iter()
            .all(|x| x.is_finite() && (0.0..=0.9).contains(x))
            && self.look_speed.is_finite()
            && (0.1..=10.0).contains(&self.look_speed)
    }
}

fn signed_axis(value: i16) -> f32 {
    value as f32 / if value < 0 { 32768.0 } else { 32767.0 }
}

/// Radial dead zone with continuous rescaling; preserves analog magnitude.
pub fn stick(x: i16, y: i16, dead_zone: f32) -> [f32; 2] {
    let x = signed_axis(x);
    let y = signed_axis(y);
    let length = x.hypot(y);
    if !dead_zone.is_finite() || !(0.0..1.0).contains(&dead_zone) || length <= dead_zone {
        return [0.0; 2];
    }
    let magnitude = (length.min(1.0) - dead_zone) / (1.0 - dead_zone);
    [x / length * magnitude, y / length * magnitude]
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct Controls {
    /// Right, forward, up. Unit-bounded by the viewer after combining devices.
    pub travel: [f32; 3],
    /// Left yaw, upward pitch, in radians per second.
    pub look: [f32; 2],
    pub fast: bool,
    pub reset: bool,
    pub toggle_collision: bool,
    pub exit: bool,
}

#[derive(Default)]
pub struct Tracker {
    slot: Option<usize>,
    previous_buttons: u16,
    was_focused: bool,
}
impl Tracker {
    pub fn slot(&self) -> Option<usize> {
        self.slot
    }
    /// Retain the selected controller until it disconnects. A new connection or
    /// focus regain establishes button baselines so held Menu/X/Y do not fire.
    pub fn sample(
        &mut self,
        pads: [Option<RawGamepad>; 4],
        focused: bool,
        settings: Settings,
    ) -> Controls {
        let old_slot = self.slot;
        self.slot = self
            .slot
            .filter(|&i| pads[i].is_some())
            .or_else(|| pads.iter().position(Option::is_some));
        let Some(raw) = self.slot.and_then(|i| pads[i]) else {
            self.previous_buttons = 0;
            self.was_focused = focused;
            return Controls::default();
        };
        let edges = if focused && self.was_focused && old_slot == self.slot {
            raw.buttons & !self.previous_buttons
        } else {
            0
        };
        self.previous_buttons = raw.buttons;
        self.was_focused = focused;
        if !focused || !settings.valid() {
            return Controls::default();
        }
        let left = stick(raw.left_x, raw.left_y, settings.left_dead_zone);
        let right = stick(raw.right_x, raw.right_y, settings.right_dead_zone);
        Controls {
            travel: [
                left[0],
                left[1],
                f32::from(raw.buttons & A != 0) - f32::from(raw.buttons & B != 0),
            ],
            look: [
                -right[0] * settings.look_speed,
                right[1] * settings.look_speed * if settings.invert_y { -1.0 } else { 1.0 },
            ],
            fast: raw.buttons & LEFT_SHOULDER != 0,
            reset: edges & Y != 0,
            toggle_collision: edges & X != 0,
            exit: edges & START != 0,
        }
    }
}
