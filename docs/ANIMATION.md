# Original character animation playback

Original Heller clips now deform the original character meshes in the independent
Rust runtime. This is inspection playback: it does not yet execute the game's
animation state machine, action scheduling, blending, root-motion driver or contacts.
Animation integration takes priority over textures.

## Run and controller controls

```powershell
.\prototype2-rust.exe animate --game 'F:\SteamLibrary\steamapps\common\Prototype 2'
```

The default clip is `heller_loco_run_n`. Five validated clips are loaded: run,
idle (`alex_amb_stand`), walk (`heller_loco_walk_n`), sprint
(`heller_loco_run_sprint_n`) and jump (`heller_loco_jump_from_idle`). A/B select
the next/previous clip. Right stick or left-stick X orbit; left-stick Y zooms.
X pauses, Y restarts, LB slows playback and Menu exits. Focus loss suppresses
input; connection/focus changes do not create button edges. These are inspection
controls, not the final retail gameplay mapping.

`animations --filter heller --limit 40` lists bounded headers without rendering.
`animate --clip <exact-name>` chooses a clip; unsupported channel layouts fail.
`--sample-frame 10.5` fixes a pose and disables clip switching. `--frames 180
--screenshot <path>` captures a bounded run. Keyboard fallback is Space, R,
Left/Right and Escape. `scripts/Animate.ps1` launches playback and saves a log.

## Corpus and evidence

Steam app 115320, build **19788008**, engine SHA-256
`c3d51d6355387963a63cbac058381b8fd1ce39f51088fa50276a49285ee94174`.
Native addresses below are preferred virtual addresses, base `0x10000000`,
for that exact x86 binary. The runtime never loads it. Private native analysis,
decoded payloads, logs and screenshots stay outside source and packages.

`boot.rcf / art\alex\alex.p3d` contains 931 animation headers and three
92-joint skeletons. Run is at decoded chunk offset **0x77176e**, end frame 21,
30 fps, with 62 joint groups. Walk is at **0x796c9f**, end frame 37, 30 fps;
`heller_amb_stand` is at **0x667b70** and exercises multiple compressed-data blocks.
Jump is at **0x75a1d6**, sprint at **0x775ef6**. These offsets identify this corpus.
The main bare LocoSteer record at **0x51495** in `alex_fig.p3d` resolves idle,
walk and run hashes to `alex_amb_stand` (**0x3f343**), `heller_loco_walk_n` and
`heller_loco_run_n`. Playback loads that trio from the actual graph references.
`heller_amb_stand` is also independently validated and selectable by `--clip`.
The main track stores phase -1, all three sync frames -1, blend-in approximately
0.133333 seconds and blend-out 0.3 seconds. Native begin **0x101e37f0** resolves
the referenced animations and constructs/reuses a locomotion driver; its phase,
sync and update policies still require recovery before implementing transitions.

`art.rcf / art\alex\alex_model_main.p3d` supplies selected body/arm geometry.
Nine supported skin primitives contain **7,978 vertices / 12,463 triangles**.
Body and arm skeletons are joined by exact drawable references; a separate
left-arm skeleton/drawable is not selected. This is not complete model assembly.

## Checked layouts

| Chunk ID | Implemented interpretation |
| --- | --- |
| 0x23000 / 0x23001 | Skeleton v1 header and joints: names, ordered parent indices, 16-float affine local bind matrix, twelve additional finite floats, u16/u32 tail |
| 0x121000 | Animation v0 name, kind, positive finite end frame/fps, cyclic flag |
| 0x121001 | Joint group v0 name, joint index and direct channel count |
| 0x2f00000 | Version 0 ZLIB blob with declared decoded/stored lengths; complete checked stream, maximum 64 MiB |
| 0x121010 | Version 0 block table, inspected 8192 marker, checked cumulative block sizes covering the blob |
| 0x121121 | Version 0 referenced key count, offset and block index; frames followed by values aligned to four-byte absolute blob position |
| 0x121112 / 0x121114 | ROT v1 signed i16/i8 XYZ, reconstructed positive W |
| 0x121104 / 0x121119 | TRAN v0 float3 / IEEE binary16 triple |
| 0x121102 | TRAN v0 scalar replacing one component of a supplied base vector |
| 0x121103 / 0x121118 | TRAN v0 float2 / binary16 pair replacing the other two components |
| 0x25000 / 0x25001 / 0x25002 | Drawable/skeleton, primitive and named shader/index/vertex/skin references |

Inline key arrays have no referenced-blob alignment padding. Key frame indices
are u16, strictly increasing and within the clip. Count/offset arithmetic, bounds,
versions, complete known payloads, unique joints/channels and finite values are
checked. Unknown transform channel formats fail with context. Non-transform phase
or event channels are not executed. Secondary skeleton fields remain opaque.

Native corroboration: skeleton loader **0x1080cdc0**, animation loader
**0x10812230**, group loader **0x10811b10**, compressed blocks **0x10810e90**,
block layout **0x108110f0**, channel/type **0x10844280**, referenced key layout
**0x108412d0**, rotation reader **0x10843bd0**, translation reader **0x10843620**.
Packed rotations use f32 bits **0x38000100** and **0x3c010204** respectively,
corroborated by unpackers **0x1080e2b0 / 0x1080e3f0**. W uses the square root of
the nonnegative clamped remainder. XYZ/W ordering is corroborated by matrix
conversion **0x107e9790**; older format documentation alone was insufficient.
The two-component axis table at **0x10d91690** selects (Y,Z), (X,Z), (X,Y).
Half expansion is corroborated by **0x10845370 / 0x1080eab0**.

Skin streams use inspected 80-byte records: float4 positions/normals/weights and
u16x4 joint indices. Both position and normal fourth lanes are **1** in these
streams. Weight sums, joint/index bounds and descriptor byte ranges are checked.
Bind-pose deformation reproduces input positions within 0.0001 units.

## Sampling and visible limits

The core samples an explicit finite frame, clamps outside individual key ranges,
linearly interpolates translation and uses normalized shortest-arc quaternion
SLERP. Missing tracks preserve bind transforms. Parent composition produces joint
world matrices; CPU skinning uses `sampled_world * inverse_bind_world`. Synthetic
two-joint fixtures independently verify a known halfway rotation and deformed point.
The native interpolator uses approximations different from glam: bitwise matching
and full blending behavior have not been demonstrated.

The viewer cancels the sampled `Motion_Root` world transform and retains the bind
root, so clips can be inspected upright in place. Raw root transforms remain in
the decoded clip. This preview policy is not recovered gameplay root motion.
Presentation delta time advances frames at the asset fps, with a 0.1-second cap;
inspection loops even non-cyclic clips. Neither that clock nor looping policy is
the retail scheduler. Asset fps does not identify the simulation tick.

Materials are diagnostic colors. Morph/cloth/expression assembly, constraints,
retail shaders, motion events, transitions, layered blends, grounded movement and
camera behavior remain unresolved. Some shoulder/hood geometry visibly overlaps.
Selected clips passing does not establish support for all 931 headers.

Default playback contains five clips; choosing an additional supported clip adds
it to the inspection bank. Six clips have now been sampled successfully, including
both idle references. This does not prove the main bare branch's retail reachability.

Run frame 0 and 10.5 captures showed distinct deformed poses. A continuous walk
capture reached frame 14.959668, demonstrating advancement rather than a static
pose. XInput slot 0 was detected; synthetic edge tests cover A/B and held/focus
behavior. Additional fixed-frame idle, sprint and jump captures were visually
inspected, with distinct standing/running/airborne poses and unresolved cloth parts.
The owner has not physically tested the new animation-specific mapping.
Automated keyboard scan-code input in the actual focused viewer verified next and
previous clips, a stable paused frame, reset to frame zero and successful exit.
That test does not simulate or establish physical Xbox input.
See VALIDATION.md for the combined test record and STATUS.md for next work.
