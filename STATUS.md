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
  in the character package; its parameter schema and retail semantics remain unknown.
- Synthetic parser/query tests and opt-in retail-data test.

## Incomplete / unknown

- Materials/textures, instance transforms, local prop placement, visibility, full
  scene assembly, cell streaming and animation.
- Collision tag/filter semantics, 232,810 selected ground-tail bytes, box/convex
  primitives, player hull, contacts and acceleration structure.
- Every gameplay system, including movement, parkour, powers, combat, missions,
  progression, AI and audio. No faithful movement slice or original-game comparison.
- Retail timing, numerical constants and native routines. Ghidra has not been run.
- Windows 10, 4K performance, other CPU configurations and long-run compatibility.
- Interactive 4K/resizing/long-run playtest; controller input has been owner-tested,
  but automated capture alone is not a full playtest result.

## Requested milestone accounting

| Step | Status |
| --- | --- |
| Separate project / build | Implemented; source publication and delivery tracked in release metadata |
| Genuine Prototype 2 data | Implemented and tested in place |
| Small real world section | Merged geometry displayed; untextured and incomplete scene dressing |
| Collision for traversing the section | Triangle geometry/query foundation works; character collision is incomplete |
| Initial movement and comparison | Blocked by recovered contacts/hull/filter rules and measured original movement |
| Runnable build / source / instructions | Packaging workflow established; release contains exact source and notices |

## Next work, in order

1. Continue viewer compatibility checks using docs/PLAYTEST.md and docs/CONTROLLER.md.
   Basic owner controller playtest passed; retain diagnostics outside source.
2. Choose a simple reachable patch in the original game and relate it to a loaded
   cell's coordinates. Avoid claiming a location from asset names alone.
3. Decode the located character META/fight schemas, collision filtering and player
   hull/contact semantics. Exact offsets and queries are in docs/MOVEMENT.md. Use
   selective native analysis where file structures cannot answer a specific rule.
4. Recover scene instance/material references, then textures and simple props.
5. Implement an explicit simulation step and initial walking slice from measured
   original behavior. Follow docs/MOVEMENT.md; do not use viewer speed as gameplay.

Private files are in a sibling `Rust Rewrite.local` directory in the owner's
workspace, outside this repository: downloaded reference snapshots, archive-name
indexes, selected binary payloads, probe scripts, scan logs, screenshots, core target
cache and release staging. Do not publish them. The outer modding repository's local
Git exclude prevents accidental tracking of the nested project/research directories.
Retail installation paths are supplied by --game / PROTOTYPE2_GAME; never commit
the full Steam manifest or private machine configuration.
