# Current status / handoff — 2026-10-08

Project: **prototype2-rust**, separate repository under uiojkl09. Original MIT source,
native Rust readers and an optional Bevy 0.18.1 viewer. Start with README.md and the
format/research/validation notes; do not repeat the archive investigation.

## Working

- Pinned x64 Rust build, build/test/packaging scripts, default-deny Git allowlist,
  publication audit and Windows CI configuration.
- Identification of Steam build 19788008 and exact executable/DLL hashes.
- All 12 RCF indexes / 26,385 entries validated against the installation.
- RZ/P3D reader; full scan of 2,969 cells.rcf files, 811,757 chunks passed.
- Real merged geometry: default yellow-zone Cell_29, 38 meshes, 185,586 triangles.
- Ground triangle decoding: 20,075 faces; geometric probes and optional visual overlay.
- Native viewer, free flight, bounded inspection commands and screenshot smoke tests.
- Native Xbox/XInput viewer controls, analog speed/dead zones, preferences, focus
  handling and connection changes; owner reported working controls and no drift.
- Checked metadata-envelope/reference inspector locating a collision-capsule factory
  in the character package; selective native inspection recovered its capsule schema.
- Checked AlexPhysicsFactory reader and `capsule` command: centre/axis, segment
  half-length/radius and material/intersection-properties references.
- Independent geometric swept-capsule query and `sweep` command: face/edge/vertex
  contact, overlap reporting and retained unknown tags; real ground-patch test passed.
- Synthetic parser/query tests and opt-in retail-data test.
- Checked `fig0` reader: six character contexts, 2,930 branches, 8,981 property
  records; selected LocoSteer/capsule tracks decoded with native corroboration.
- Isolated LocoSteer target-speed, acceleration and turning rules in Rust; single
  capsule-action capture/interpolation/restore. These do not yet form gameplay.
- Original skeleton/packed rotation/translation/scale readers, compressed key blocks
  and checked skin references. Nine Heller skin meshes deform using original clips.
- `animate` plays run, idle, walk, sprint, jump, bare punch and bare kick;
  Xbox A/B select clips, X pauses,
  Y restarts, sticks orbit/zoom and Menu exits. Fixed-pose GPU captures and continuous
  playback verified. Animation-specific physical controls have not been owner-tested.
- Native frame-count-minus-one timing and an isolated three-stage locomotion phase
  rule: adjacent velocity weights, candidate trimming, weighted durations and speed
  scaling, checked sync offsets and frame mapping. Local idle/walk/run pose blending
  now reaches the renderer; RB/left stick inspect it in place. State scheduling and
  native quaternion approximation parity remain separate work, documented in ANIMATION.md.
- Recovered single-clip root displacement/rotation extraction: zero/one inferred
  cycle crossing, segment-relative translation and reverse flags. `root-motion`
  reports explicit frame endpoints; five real root tracks measured in both spaces.
  This is authored clip motion, with no actor/contact integration or scheduler.
- Heller's installed collar/shoulder pose-fixup configuration now updates corrective
  joints before skinning in single-clip and blended previews. Eleven clips on both
  skeletons tested; visual overlap still remains. Broader state/rig policies are
  unsupported, documented in ANIMATION.md.
- Wider character-package coverage: all 741 skeletal patterns decode and sample,
  including 580 authored scale channels across 35 clips and five clips with disabled
  channel metadata. Camera/expression patterns remain unsupported. Compatible actor
  rigs and graph reachability are not established by sampling on the Heller rigs.
- Bare punch/kick, blade activation and shield fixed-pose captures were inspected;
  continuous punch playback advances. Weapon meshes and complete power/model/cloth
  assembly remain incomplete. Controller-bank switching passed actual-window automation.

## Incomplete / unknown

- Materials/textures, instance transforms, local prop placement, visibility, full
  scene assembly, cell streaming and full model/morph/cloth assembly.
- Animation state transitions, layered/additive blending/events, root-motion integration and native
  interpolation parity. Current playback loops clips in place using a preview clock.
- Collision tag/filter semantics, 232,810 selected ground-tail bytes, box/convex
  primitives, active player hull/transforms, retail contact response and acceleration
  structure. The decoded default factory is not confirmed as the walking hull.
- Every gameplay system, including movement, parkour, powers, combat, missions,
  progression, AI and audio. No faithful movement slice or original-game comparison.
- Retail timing, input/component interpretation and complete movement rules.
  Selective Ghidra analysis has now run; main LocoSteer values and isolated scalar
  rules are recovered. State execution, root motion and contacts remain incomplete.
- Windows 10, 4K performance, other CPU configurations and long-run compatibility.
- Interactive 4K/resizing/long-run playtest; controller input has been owner-tested,
  but automated capture alone is not a full playtest result.

## Requested milestone accounting

| Step | Status |
| --- | --- |
| Separate project / build | Implemented; source publication and delivery tracked in release metadata |
| Genuine Prototype 2 data | Implemented and tested in place |
| Small real world section | Merged geometry displayed; untextured and incomplete scene dressing |
| Collision for traversing the section | Asset capsule and geometric translation queries work; retail filters, active hull and response remain incomplete |
| Initial movement and comparison | Blocked by recovered contacts/hull/filter rules and measured original movement |
| Runnable build / source / instructions | Packaging workflow established; release contains exact source and notices |

## Next work, in order

1. **Animation before textures**, following the owner's priority. Continue from
   docs/ANIMATION.md: original clips now play. Recover animation/action references,
   transitions, blending and the root-motion driver, then connect them to state.
2. Continue from docs/FIGHT.md: graph serialization and selected LocoSteer/capsule
   schemas are implemented. Resolve conditions/references, sprint/jump drivers and
   action scheduling. Recover active hull/transforms, filtering and contact response
   before interpreting the query as player collision.
3. Relate a reachable original-game patch to loaded coordinates and recover/measure
   original input, timing, velocity and camera behavior. Follow docs/MOVEMENT.md.
4. Implement an explicit simulation step and initial walking/jumping slice from that
   evidence, then compare repeatable traces. Perfect recreation remains the target;
   guessed mechanics or inspection speeds must not stand in for original behavior.
5. Recover scene instance/material references, then textures and props.
6. Continue compatibility checks with docs/PLAYTEST.md and docs/CONTROLLER.md.
   Basic owner Xbox playtest passed; retain diagnostics outside source.

Private files are in a sibling `Rust Rewrite.local` directory in the owner's
workspace, outside this repository: downloaded reference snapshots, archive-name
indexes, selected binary payloads, probe scripts, scan logs, screenshots, core target
cache and release staging. Do not publish them. The outer modding repository's local
Git exclude prevents accidental tracking of the nested project/research directories.
Retail installation paths are supplied by --game / PROTOTYPE2_GAME; never commit
the full Steam manifest or private machine configuration.
