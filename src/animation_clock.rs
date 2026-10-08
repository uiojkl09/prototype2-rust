//! Cropped single-clip driver math; action selection and synchronization are separate.
use crate::animation::ClipInfo;
use anyhow::{Result, ensure};
use serde::Serialize;

const EPSILON: f32 = 0.00001;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ClipTiming {
    first_frame: f32,
    last_frame: f32,
    frames_per_second: f32,
    speed: f32,
}
impl ClipTiming {
    /// Configure the inspected skeletal driver. Negative end offsets use COUNT,
    /// then the wrapper clamps; negative start values clamp to zero. Other native
    /// component paths have different rules and are outside this contract.
    /// `cyclic_at_setup` is driver state when setup runs, not the clip's metadata:
    /// the original begin path can apply its cyclic policy afterwards.
    pub fn configure(
        clip: &ClipInfo,
        start: f32,
        end: f32,
        duration: f32,
        requested_speed: f32,
        cyclic_at_setup: bool,
    ) -> Result<Self> {
        ensure!(
            clip.frame_count.is_finite()
                && (1. ..=32768.).contains(&clip.frame_count)
                && clip.frames_per_second.is_finite()
                && (0. ..=1000.).contains(&clip.frames_per_second)
                && clip.frames_per_second > 0.
                && [start, end, duration, requested_speed]
                    .iter()
                    .all(|v| v.is_finite()),
            "invalid single-clip timing configuration"
        );
        let first_frame = start.clamp(0., clip.last_frame());
        let end = if end < 0. {
            clip.frame_count + end
        } else {
            end
        };
        ensure!(end.is_finite(), "animation end offset overflow");
        let last_frame = if end < 0. {
            clip.last_frame()
        } else {
            end.clamp(first_frame, clip.last_frame())
        };
        let mut timing = Self {
            first_frame,
            last_frame,
            frames_per_second: clip.frames_per_second,
            speed: requested_speed,
        };
        if duration > EPSILON && !cyclic_at_setup {
            timing.speed = 1.;
            // Native duration lookup uses -1 for near-zero fps/speed. Preserve
            // that arithmetic even for a supplied unusual, very-low-fps header.
            let cycle = timing.cycle_seconds()?.unwrap_or(-1.);
            let sign = if requested_speed < 0. {
                -1.
            } else if requested_speed > 0. {
                1.
            } else {
                0.
            };
            timing.speed = sign / duration * cycle;
            ensure!(timing.speed.is_finite(), "animation speed overflow");
        }
        Ok(timing)
    }
    pub fn first_frame(self) -> f32 {
        self.first_frame
    }
    pub fn last_frame(self) -> f32 {
        self.last_frame
    }
    pub fn speed(self) -> f32 {
        self.speed
    }
    /// The skeletal completion refresh suppresses completion for cyclic/hold
    /// drivers; otherwise it compares the current frame with the directional end.
    /// This is not action removal, graph completion or an event dispatch.
    pub fn finished(self, frame: f32, cyclic: bool, hold_end_frame: bool) -> Result<bool> {
        ensure!(frame.is_finite(), "invalid animation completion frame");
        Ok(!cyclic
            && !hold_end_frame
            && if self.speed < 0. {
                frame <= self.first_frame
            } else {
                frame >= self.last_frame
            })
    }
    /// None represents the native -1 sentinel for near-zero speed or fps.
    pub fn cycle_seconds(self) -> Result<Option<f32>> {
        if self.speed.abs() <= EPSILON || self.frames_per_second.abs() <= EPSILON {
            return Ok(None);
        }
        let seconds = ((f64::from(self.last_frame) - f64::from(self.first_frame))
            / (f64::from(self.frames_per_second) * f64::from(self.speed)))
        .abs() as f32;
        ensure!(seconds.is_finite(), "animation duration overflow");
        Ok(Some(seconds))
    }
    pub fn frame_delta(self, seconds: f32) -> Result<f32> {
        ensure!(
            seconds.is_finite() && seconds >= 0.,
            "invalid animation elapsed time"
        );
        let delta =
            (f64::from(self.frames_per_second) * f64::from(self.speed) * f64::from(seconds)) as f32;
        ensure!(delta.is_finite(), "animation frame delta overflow");
        Ok(delta)
    }
    /// Clamp a noncyclic frame or periodically map a cyclic frame. Native endpoint
    /// epsilon applies to wrapped results only; an in-range near-end frame stays.
    /// Floating-point instruction parity with the original x87 math is unproven.
    pub fn map_frame(self, frame: f32, cyclic: bool) -> Result<f32> {
        ensure!(frame.is_finite(), "invalid animation frame");
        if !cyclic {
            return Ok(frame.clamp(self.first_frame, self.last_frame));
        }
        if frame >= self.first_frame && frame < self.last_frame {
            return Ok(frame);
        }
        let width = self.last_frame - self.first_frame;
        if width <= EPSILON {
            return Ok(self.first_frame);
        }
        let mapped = (f64::from(self.first_frame)
            + (f64::from(frame) - f64::from(self.first_frame)).rem_euclid(f64::from(width)))
            as f32;
        ensure!(mapped.is_finite(), "animation frame mapping overflow");
        Ok(
            if (mapped - self.first_frame).abs() <= EPSILON
                || (mapped - self.last_frame).abs() <= EPSILON
            {
                self.first_frame
            } else {
                mapped
            },
        )
    }
    /// Explicit caller-time step, without a scheduler, event dispatch or actor tick.
    pub fn advance(self, frame: f32, seconds: f32, cyclic: bool) -> Result<f32> {
        let frame = self.map_frame(frame, cyclic)? + self.frame_delta(seconds)?;
        ensure!(frame.is_finite(), "animation frame step overflow");
        self.map_frame(frame, cyclic)
    }
}
