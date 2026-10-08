//! Recovered single capsule-track shape changes. Joint attachment, rotations,
//! simultaneous action ordering and world collision response remain unresolved.
use crate::{
    capsule::Capsule,
    fight::{Animated, CapsuleTrack},
};
use anyhow::{Result, ensure};

pub struct CapsuleAction {
    baseline: Capsule,
    track: CapsuleTrack,
}
impl CapsuleAction {
    /// Capture the shape before the action. Offsets are relative to this snapshot;
    /// resampling must never accumulate offsets from the preceding frame.
    pub fn begin(baseline: Capsule, track: CapsuleTrack) -> Result<(Self, Capsule)> {
        baseline.validate()?;
        ensure!(
            track.header.time_begin.is_finite() && track.header.time_end.is_finite(),
            "invalid capsule action time bounds"
        );
        ensure!(
            track.attach_to_joint == 0,
            "capsule joint attachment is not recovered"
        );
        ensure!(
            !track.rotation_degrees.enabled,
            "capsule rotation is not recovered"
        );
        let action = Self { baseline, track };
        let initial = action.at_fraction(0.)?;
        Ok((action, initial))
    }
    /// Sample the engine's elapsed-action value before it adds this update's dt.
    /// No scheduler or simulation tick is supplied by this shape primitive.
    pub fn sample(&self, elapsed_seconds: f32) -> Result<Capsule> {
        ensure!(
            elapsed_seconds.is_finite(),
            "non-finite capsule action time"
        );
        if !self.track.animate {
            return self.at_fraction(0.);
        }
        let duration = self.track.header.time_end - self.track.header.time_begin;
        ensure!(duration.is_finite(), "invalid capsule action duration");
        let fraction = if duration > 0.00001 {
            (elapsed_seconds / duration).clamp(0., 1.)
        } else {
            1.
        };
        self.at_fraction(fraction)
    }
    fn at_fraction(&self, fraction: f32) -> Result<Capsule> {
        let scalar = |field: &Animated<f32>| {
            if fraction == 0. {
                field.initial
            } else {
                (field.final_value - field.initial) * fraction + field.initial
            }
        };
        let vector = |field: &Animated<[f32; 3]>| {
            std::array::from_fn::<_, 3, _>(|i| {
                if fraction == 0. {
                    field.initial[i]
                } else {
                    (field.final_value[i] - field.initial[i]) * fraction + field.initial[i]
                }
            })
        };
        let mut shape = self.baseline.clone();
        if self.track.offset.enabled {
            let offset = vector(&self.track.offset);
            shape.centre = std::array::from_fn(|i| self.baseline.centre[i] + offset[i]);
        }
        if self.track.extent.enabled {
            shape.extent = scalar(&self.track.extent);
        }
        if self.track.radius.enabled {
            shape.radius = scalar(&self.track.radius);
        }
        if self.track.axis.enabled {
            shape.axis = vector(&self.track.axis);
        }
        shape.validate()?;
        Ok(shape)
    }
    /// Ending this action restores the snapshot, as observed in the native end
    /// handler. A future scheduler must preserve original overlap/end ordering.
    pub fn end(self) -> Capsule {
        self.baseline
    }
}
