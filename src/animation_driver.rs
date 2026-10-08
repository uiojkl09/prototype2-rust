//! Recovered three-stage, nondirectional locomotion phase and local pose blending.
//! This is not action selection, layered pose blending, root integration or a tick.
use crate::animation::{Clip, Skeleton, world_matrices};
use anyhow::{Result, ensure};
use glam::Mat4;

#[derive(Clone, Copy, Debug)]
pub struct PhaseStep {
    pub phase: f32,
    pub weights: [f32; 3],
    pub cycle_seconds: f32,
}

/// Sample the recovered adjacent-stage blend into joint world matrices.
/// Weights are normalized candidates produced by `advance_phase`.
/// Missing channels use the supplied skeleton's bind components. Partitions,
/// layers, aliases and additive blends are outside this standalone contract.
pub fn sample_locomotion(
    skeleton: &Skeleton,
    clips: [&Clip; 3],
    phase: f32,
    weights: [f32; 3],
    sync_frames: [f32; 3],
) -> Result<Vec<Mat4>> {
    ensure!(
        weights
            .iter()
            .all(|w| w.is_finite() && (0. ..=1.).contains(w))
            && (weights.iter().sum::<f32>() - 1.).abs() < 0.00001
            && !(weights[0] > 0. && weights[2] > 0.),
        "invalid adjacent locomotion weights"
    );
    ensure!(
        clips[0].info.name != clips[1].info.name
            && clips[1].info.name != clips[2].info.name
            && clips[0].info.name != clips[2].info.name,
        "aliased locomotion clips are unsupported"
    );
    let mut order = [0, 1, 2];
    order.sort_by(|a, b| weights[*b].total_cmp(&weights[*a]));
    let first = order[0];
    let frame = |i: usize| frame_at_phase(phase, clips[i].info.last_frame(), sync_frames[i]);
    let mut pose = clips[first].sample_local(skeleton, frame(first)?)?;
    let mut accumulated = weights[first];
    for &stage in &order[1..] {
        if weights[stage] == 0. {
            continue;
        }
        accumulated += weights[stage];
        let fraction = weights[stage] / accumulated;
        let incoming = clips[stage].sample_local(skeleton, frame(stage)?)?;
        for (base, next) in pose.iter_mut().zip(incoming) {
            base.translation = base.translation.lerp(next.translation, fraction);
            base.scale = base.scale.lerp(next.scale, fraction);
            // The native skeletal blender chooses the incoming hemisphere at dot <= 0.
            let rotation = if base.rotation.dot(next.rotation) <= 0. {
                -next.rotation
            } else {
                next.rotation
            };
            base.rotation = base.rotation.slerp(rotation, fraction).normalize();
        }
    }
    world_matrices(
        &skeleton.joints,
        &pose.iter().map(|p| p.matrix()).collect::<Vec<_>>(),
    )
}

/// Native nondirectional driver maps shared phase through the wrapper's sync
/// offset, then wraps over count minus one. This does not execute sync events.
pub fn frame_at_phase(phase: f32, last_frame: f32, sync_frame: f32) -> Result<f32> {
    ensure!(
        phase.is_finite()
            && (0. ..1.).contains(&phase)
            && last_frame.is_finite()
            && last_frame >= 0.
            && sync_frame.is_finite(),
        "invalid animation phase/frame mapping"
    );
    let frame = sync_frame + phase * last_frame;
    ensure!(frame.is_finite(), "animation frame mapping overflow");
    if frame < 0. || frame >= last_frame {
        if last_frame <= 0.00001 {
            return Ok(0.);
        }
        let wrapped = frame.rem_euclid(last_frame);
        return Ok(
            if wrapped <= 0.00001 || (wrapped - last_frame).abs() <= 0.00001 {
                0.
            } else {
                wrapped
            },
        );
    }
    Ok(frame)
}

/// Advance three distinct available idle/walk/run clips using explicit stage
/// velocities and cycle durations. Missing/aliased clips and directional stages
/// require additional native policies and are deliberately outside this primitive.
pub fn advance_phase(
    phase: f32,
    speed: f32,
    velocities: [f32; 3],
    cycle_seconds: [f32; 3],
    seconds: f32,
) -> Result<PhaseStep> {
    ensure!(
        phase.is_finite() && (0. ..1.).contains(&phase),
        "invalid locomotion animation phase"
    );
    ensure!(
        speed.is_finite() && speed >= 0. && seconds.is_finite() && seconds >= 0.,
        "invalid locomotion animation step"
    );
    ensure!(
        velocities[0] == 0.
            && velocities.iter().all(|v| v.is_finite())
            && velocities.windows(2).all(|p| p[0] < p[1])
            && cycle_seconds.iter().all(|v| v.is_finite() && *v >= 0.),
        "unsupported locomotion animation stages"
    );
    let lower = velocities.iter().rposition(|v| *v <= speed).unwrap_or(0);
    let mut weights = [0.; 3];
    if lower == 2 {
        weights[lower] = 1.;
    } else {
        let blend = (speed - velocities[lower]) / (velocities[lower + 1] - velocities[lower]);
        weights[lower] = 1. - blend;
        weights[lower + 1] = blend;
    }
    // Native update sorts candidates, drops weights below 0.01 and renormalizes.
    // With distinct nondirectional stages the surviving result is order-independent.
    for weight in &mut weights {
        if *weight < 0.01 {
            *weight = 0.;
        }
    }
    let total = weights.iter().sum::<f32>();
    for weight in &mut weights {
        *weight /= total;
    }
    let duration = weights
        .iter()
        .zip(cycle_seconds)
        .map(|(w, t)| w * t)
        .sum::<f32>();
    let reference_speed = weights
        .iter()
        .zip(velocities)
        .map(|(w, v)| w * v)
        .sum::<f32>();
    ensure!(
        duration.is_finite() && reference_speed.is_finite(),
        "animation stage overflow"
    );
    let scaled_seconds = if reference_speed > 0. {
        seconds * (speed / reference_speed)
    } else {
        seconds
    };
    ensure!(scaled_seconds.is_finite(), "animation timestep overflow");
    let mut next = if duration > 0. {
        phase + scaled_seconds / duration
    } else {
        phase
    };
    ensure!(next.is_finite(), "animation phase overflow");
    if !(0. ..1.).contains(&next) {
        next = next.rem_euclid(1.);
        if next <= 0.00001 || (next - 1.).abs() <= 0.00001 {
            next = 0.;
        }
    }
    Ok(PhaseStep {
        phase: next,
        weights,
        cycle_seconds: duration,
    })
}
