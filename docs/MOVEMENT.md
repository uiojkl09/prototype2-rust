# Initial movement milestone: blocked prerequisites and comparison protocol

No Prototype 2 movement slice is implemented yet. The free camera has arbitrary
inspection speed and passes through walls. It must not be presented as recovered
walking, sprinting, jumping or parkour, even if the viewer looks convincing.

## Exact prerequisites

1. Resolve collision face tags, filtering, transforms and relevant box/convex
   primitives. Decode or replace the observed spatial-tree data with an independently
   validated acceleration structure. Establish winding/contact semantics.
2. Recover player hull dimensions, collision offsets, slope/step limits and grounded
   state transitions. Triangle ray hits alone cannot prevent body penetration.
3. Measure basic walking acceleration, steady speed, deceleration and camera behavior
   from the identified original build. World units are not yet calibrated to metres.
4. Recover or measure original simulation/input timing. Select a runtime fixed tick
   only after that evidence; do not hard-code 60/120 Hz as a claim of original behavior.

The initial character slice should be walking, stopping, turning and falling on a
known ground patch, then sprint/jump. Powers and wall-running follow after contacts
and state transitions are verified. Use native executable analysis selectively if
data/measurement does not answer a specific rule; keep output outside the repository.

## Repeatable comparison

Record binary identity, save/progression/upgrade state, location, frame cap, input
device and a fixed start orientation. Use the same input timeline: 2 seconds idle,
3 seconds straight forward, 2 seconds released, then repeat for reverse, lateral
and diagonal input. Repeat five times at 30, 60 and 120 FPS when supported.

Measure wall-clock input duration, displacement in game coordinates (once a safe
read-only measurement method is established), velocity curve, stopping distance,
camera yaw/pitch and grounded state. Screen-space video alone is insufficient to
infer world speed without camera/unit calibration. Do not label those estimates
confirmed. Jump tests additionally record launch, apex, landing and state changes.

Future Rust replay should use explicit per-tick input and emit CSV with time,
position, velocity, state, contact normal and camera state. Run identical inputs
with presentation capped at 30/60/120/240 FPS and compare simulation outputs.
Document missed-frame handling and do not silently change elapsed simulation time.

Proposed acceptance thresholds, pending calibration: steady travel and stop distance
within 2% of measured original means; state transitions within one established
original simulation tick; no observed penetration or false floor on tested contacts.
These are planned checks, not completed results. Save original and runtime traces
privately; publish only derived measurements and authored test scenarios.
