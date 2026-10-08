//! Recovered LocoSteer scalar rules, independent of rendering and input hardware.
//! These primitives do not supply the still-unrecovered tick, camera or contacts.
use anyhow::{Result, ensure};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct SteerSpeeds {
    pub walk: f32,
    pub run: f32,
}
impl SteerSpeeds {
    fn validate(self) -> Result<()> {
        ensure!(
            self.walk.is_finite() && self.run.is_finite() && self.walk >= 0. && self.run >= 0.,
            "unresolved or invalid locomotion speeds"
        );
        Ok(())
    }

    /// Scalar helper in the inspected LocoSteer action. `fixed_midpoint` mirrors
    /// its mode flag; the component that determines that flag is still unresolved.
    /// Input dead zones and upstream intention conversion are separate work.
    pub fn target(self, magnitude: f32, fixed_midpoint: bool) -> Result<f32> {
        self.validate()?;
        ensure!(
            magnitude.is_finite() && magnitude >= 0.,
            "invalid intention magnitude"
        );
        if magnitude == 0. {
            return Ok(0.);
        }
        let split = if !fixed_midpoint && self.run > 0. {
            (self.walk / self.run).min(0.99)
        } else {
            0.5
        };
        let speed = if magnitude <= split {
            (magnitude / split) * self.walk
        } else if magnitude < 1. {
            let blend = (magnitude - split) / (1. - split);
            (1. - blend) * self.walk + self.run * blend
        } else {
            self.run * magnitude
        };
        ensure!(speed.is_finite(), "locomotion speed overflow");
        Ok(speed)
    }
}

/// LocoSteer uses one positive acceleration limit for both rising and falling
/// speed. A nonpositive limit snaps to target. Seconds are supplied explicitly;
/// this function does not choose or claim an original simulation frequency.
pub fn advance_speed(current: f32, target: f32, acceleration: f32, seconds: f32) -> Result<f32> {
    ensure!(
        [current, target, acceleration, seconds]
            .iter()
            .all(|v| v.is_finite()),
        "non-finite locomotion step"
    );
    ensure!(
        current >= 0. && target >= 0. && seconds >= 0.,
        "negative speed or timestep"
    );
    if acceleration <= 0. {
        return Ok(target);
    }
    let change = acceleration * seconds;
    ensure!(change.is_finite(), "locomotion acceleration overflow");
    let difference = target - current;
    Ok(if difference > 0. && change <= difference {
        current + change
    } else if difference <= -change {
        current - change
    } else {
        target
    })
}

fn wrapped_difference(current: f32, desired: f32) -> Result<f32> {
    let delta = desired - current;
    ensure!(delta.is_finite(), "steering angle overflow");
    let lower = -std::f32::consts::PI;
    let upper = std::f32::consts::PI;
    if lower <= delta && delta < upper {
        return Ok(delta);
    }
    let width = f64::from(upper) - f64::from(lower);
    let turns = ((f64::from(delta) - f64::from(lower)) / width).floor();
    let wrapped = f64::from(delta) - turns * f64::from(width as f32);
    if (wrapped - f64::from(lower)).abs() <= f64::from(0.00001f32)
        || (wrapped - f64::from(upper)).abs() <= f64::from(0.00001f32)
    {
        Ok(lower)
    } else {
        Ok(wrapped as f32)
    }
}

/// Recovered zero-angular-acceleration case used by the inspected main LocoSteer
/// track. The engine blends turn-rate limits by current speed / run speed.
/// Includes the inspected shortest-angle wrap and boundary tolerance. Input/camera
/// coordinate transforms remain outside this rule; x87 bit parity is not claimed.
pub fn turn_toward(
    current: f32,
    desired: f32,
    current_speed: f32,
    run_speed: f32,
    walk_rate: f32,
    run_rate: f32,
    seconds: f32,
) -> Result<f32> {
    ensure!(
        [
            current,
            desired,
            current_speed,
            run_speed,
            walk_rate,
            run_rate,
            seconds
        ]
        .iter()
        .all(|v| v.is_finite()),
        "non-finite steering step"
    );
    ensure!(
        current_speed >= 0.
            && run_speed >= 0.
            && walk_rate >= 0.
            && run_rate >= 0.
            && seconds >= 0.,
        "unsupported steering values"
    );
    let blend = if run_speed > 0. {
        (current_speed / run_speed).clamp(0., 1.)
    } else {
        0.
    };
    let rate = (run_rate - walk_rate) * blend + walk_rate;
    let angle = wrapped_difference(current, desired)?;
    let maximum = rate * seconds;
    ensure!(maximum.is_finite(), "steering step overflow");
    Ok(current + angle.clamp(-maximum, maximum))
}
