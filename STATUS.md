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

## Incomplete / unknown

- Materials/textures, instance transforms, local prop placement, visibility, full
  scene assembly, cell streaming and animation.
- Collision tag/filter semantics, 232,810 selected ground-tail bytes, box/convex
  primitives, active player hull/transforms, retail contact response and acceleration
  structure. The decoded default factory is not confirmed as the walking hull.
- Every gameplay system, including movement, parkour, powers, combat, missions,
  progression, AI and audio. No faithful movement slice or original-game comparison.
- Retail timing and movement constants/rules. Selective native capsule schema and
  geometry inspection used PE parsing/Capstone; Ghidra has not been run.
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

1. Decode fight-node serialization, locomotion/capsule tracks and state transitions.
   Native capsule field names and geometry are recovered; exact evidence and queries
   are in docs/CAPSULE.md. Recover active hull/transforms and intersection-property
   filtering/contact response before interpreting the query as player collision.
2. Relate a reachable original-game patch to loaded coordinates and recover/measure
   original input, timing, velocity and camera behavior. Follow docs/MOVEMENT.md.
3. Implement an explicit simulation step and initial walking/jumping slice from that
   evidence, then compare repeatable traces. Perfect recreation remains the target;
   guessed mechanics or inspection speeds must not stand in for original behavior.
4. Recover scene instance/material references, then textures, props and animation.
5. Continue compatibility checks with docs/PLAYTEST.md and docs/CONTROLLER.md.
   Basic owner Xbox playtest passed; retain diagnostics outside source.

Private files are in a sibling `Rust Rewrite.local` directory in the owner's
workspace, outside this repository: downloaded reference snapshots, archive-name
indexes, selected binary payloads, probe scripts, scan logs, screenshots, core target
cache and release staging. Do not publish them. The outer modding repository's local
Git exclude prevents accidental tracking of the nested project/research directories.
Retail installation paths are supplied by --game / PROTOTYPE2_GAME; never commit
the full Steam manifest or private machine configuration.
