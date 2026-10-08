# Initial research assessment

Reviewed on 2026-10-08. Upstream revisions below identify what was actually inspected.
No decompiled retail source or database is in this repository. Public references
were downloaded into a private sibling research directory. Tools were evaluated;
they were not blindly installed or treated as recovery evidence.

| Reference / revision | Assessment and decision |
| --- | --- |
| [AI Game Modding Guides](https://github.com/trevaintdead/ai-game-modding-guides/tree/1c8df26a64f3d8604fa05ac2b3d3b65c0dde31ef), guides 3 and 12 | MIT workflow reference. Reviewed Rust rewrite guide and IW4L worked example. Adopted measured milestones, separate assets, provenance notes and explicit limitations; no guide code copied. |
| [IW4L](https://github.com/vladtrc/iw4L/tree/a2f4e0b8573eeb828001f8ee757b5b724eee196f) | Apache-2.0. Studied README, Cargo workspace, build, map-load, render and simulation contracts. Its FastFile/GSC readers are game-specific. Explicit simulation input/time and separation of asset products from rendering are useful architectural references; no code reused. Its documented gameplay remains incomplete. |
| [gamedb](https://github.com/smileybaal/gamedb/tree/7054201291d704d64cdf37a9d3573c5d16a81fcf) | MIT. Search/call-graph index for a future private decompilation corpus. Not needed for current file-format work and not installed. Index status does not prove recovered semantics. |
| [RustPorts toolkit](https://github.com/phoenixfire808/rustports/tree/3961b90140a594c813e7c5b004eff73bf27b0de3/toolkit) | Research-ledger/spreadsheet workflow; native Jcode described as experimental. Useful evidence discipline, not automatic engine reconstruction. No runtime dependency or code reuse. |
| [universal-modder](https://github.com/rehan-remade/universal-modder/tree/8370faa8e114baf33acdb23079aff552a7728c4b) | MIT recon/modding workflow and tool orchestration. Generated art and passthrough routes do not help a faithful standalone runtime. Not installed; no code reused. |
| [GameDecompLibrary](https://github.com/solarfren69420/GameDecompLibrary/tree/0e1357d14b9c5b0807631b43fefe7459e5ae7c09) | Catalog checked for Prototype entries. No relevant Prototype 2 reimplementation was identified in the inspected material. This is a bounded search result, not proof that none exists. |
| [Gibbed.Prototype](https://github.com/gibbed/Gibbed.Prototype/tree/026da397342c4f8a43af575232dccf4b104df2e5) | Permissive zlib-style file notices. Inspected CementFile, metadata, hash helpers, compressor, Pure3D tree, physics and geometry nodes. RCF/RZ/P3D envelope knowledge was directly useful and independently validated against the installation. Its physics class does not recover character collision. |
| [RcfTools](https://github.com/NixsonLai/RcfTools/tree/636904bfcd9d5c136c377a624d89fe92634d336d) | MIT. Cross-checked table offsets, metadata and hash behavior. Native Rust reader reads a selected payload instead of loading whole archives. Preserve attribution; do not inherit unchecked parsing or path extraction. |
| [Prototype 2 Noesis importer](https://github.com/RoadTrain/noesis-plugins-official/blob/master/demonsangel/fmt_pt2_p3d.py) | Investigative pointer to named buffers. No clear reuse license established, so no code copied or ported. Current Rust decoder was derived from observed chunk descriptors and actual buffer bounds; supports only tested encodings. |

## Existing workspace

The neighboring modding workspace contains Lua package-loading helpers and mods,
exported Prototype 2 data, Discord research, a picture-based presentation, and a
Prototype 1 compatibility launcher. Package and motion-state helpers are clues for
later research, not standalone behavior implementations. Local skeleton notes suggest
separate model/skeleton packages; verify them against the current build before use.
No private chat content or neighboring assets were copied into the new repository.

The Prototype 1 launcher describes CPU-affinity, OBS and HID workarounds. It was
read as historical context and was not executed. These descriptions do not establish
Prototype 2 failures or causes. Prototype 1 is available as a comparison installation;
its geometry and behavior must not be substituted for Prototype 2's.

## What still requires investigation

RCF indexing and tested RZ/P3D envelope reading no longer require novel reverse
engineering. Merged vertex/index decoding is implemented for one section. Material
bindings, instanced transforms, animation tracks, collision flags/trees and primitive
types need additional format work. Movement rules, capsule/contact behavior, camera
and mission behavior require original-game measurements and selective native
executable analysis. Capsule field registration/endpoint/bounds routines have now
been inspected; their authored evidence is in CAPSULE.md. No original movement
constants, timing or retail contact response have been accepted.

Ghidra was not needed for this archive/viewer milestone and was not run. When needed,
identify the hashed binary, inspect relevant string/import references and routines,
keep its project private, and publish only original format/behavior descriptions.
Use gamedb if a selective decompilation corpus makes indexing worthwhile.

The capsule prerequisite used already-installed pefile 2024.8.26 and Capstone 5.0.7
for bounded read-only PE/RTTI and native-function inspection outside the repository.
No tool installation, proprietary code port, retail execution or patch was required.
Initialization scripts were structurally inspected privately as Lua 5.1 bytecode
with 32-bit float numbers. Selected script constants identify locomotion setting
accessors; they do not establish walking/jumping values. Fight-track schemas/native
consumers remain the next investigation target. Private Lua/disassembly files are
not runtime dependencies or public deliverables.
