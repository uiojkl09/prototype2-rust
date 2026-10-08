//! Inspected single-clip root delta extraction, independent of actor integration.
use crate::animation::{JointTrack, TrackSample};
use anyhow::{Result, ensure};
use glam::{Quat, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Interval {
    pub previous: f32,
    pub current: f32,
    pub first: f32,
    pub last: f32,
    pub relative_translation: bool,
    pub reverse: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct RootDelta {
    pub translation: Vec3,
    pub rotation: Quat,
}
/// Extract zero or one cycle crossing, inferred from endpoint ordering as in the
/// native consumer. Equal endpoints mean zero delta, not a complete cycle.
/// This returns authored track motion; it does not choose the root, scale velocity,
/// blend drivers, apply contacts or move a character.
pub fn delta(track: Option<&JointTrack>, interval: Interval) -> Result<RootDelta> {
    let Interval {
        previous,
        current,
        first,
        last,
        relative_translation,
        reverse,
    } = interval;
    ensure!(
        [previous, current, first, last]
            .iter()
            .all(|f| f.is_finite())
            && first >= 0.
            && last >= first
            && (first..=last).contains(&previous)
            && (first..=last).contains(&current),
        "invalid root-motion frame interval"
    );
    let sample = |frame| -> Result<TrackSample> {
        match track {
            Some(track) => track.sample(frame),
            None => Ok(TrackSample::default()),
        }
    };
    let (start, end) = if reverse {
        (current, previous)
    } else {
        (previous, current)
    };
    let segment = |from: TrackSample, to: TrackSample| {
        let orientation = from.rotation.unwrap_or(Quat::IDENTITY);
        let raw = to.translation.unwrap_or(Vec3::ZERO) - from.translation.unwrap_or(Vec3::ZERO);
        RootDelta {
            translation: if relative_translation {
                orientation.conjugate() * raw
            } else {
                raw
            },
            rotation: orientation.conjugate() * to.rotation.unwrap_or(Quat::IDENTITY),
        }
    };
    let start_pose = sample(start)?;
    let end_pose = sample(end)?;
    let mut result = if end < start {
        let tail = segment(start_pose, sample(last)?);
        let head = segment(sample(first)?, end_pose);
        // Native row-convention multiply maps to head * tail in glam's convention.
        RootDelta {
            translation: tail.translation + head.translation,
            rotation: head.rotation * tail.rotation,
        }
    } else {
        segment(start_pose, end_pose)
    };
    if reverse {
        result.translation = -result.translation;
        result.rotation = result.rotation.conjugate();
    }
    ensure!(
        result.translation.is_finite() && result.rotation.is_finite(),
        "root-motion delta overflow"
    );
    Ok(result)
}
