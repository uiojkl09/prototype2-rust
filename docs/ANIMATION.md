# Original character animation playback

Original Heller clips now deform the original character meshes in the independent
Rust runtime. This is inspection playback: it does not yet execute the game's
animation state machine, action scheduling, layered blending, root-motion driver or contacts.
The recovered standalone idle/walk/run blend now also deforms the character.
Animation integration takes priority over textures.

## Run and controller controls

```powershell
.\prototype2-rust.exe animate --game 'F:\SteamLibrary\steamapps\common\Prototype 2'
```

The default clip is `heller_loco_run_n`. Seven validated clips are loaded: run,
idle (`alex_amb_stand`), walk (`heller_loco_walk_n`), sprint
(`heller_loco_run_sprint_n`), jump (`heller_loco_jump_from_idle`), bare punch
(`heller_bare_punch_1`) and bare kick (`heller_bare_kick_1`). A/B select
the next/previous clip. Right stick or left-stick X orbit; left-stick Y zooms.
X pauses, Y restarts, LB slows playback and Menu exits. Focus loss suppresses
input; connection/focus changes do not create button edges. These are inspection
controls, not the final retail gameplay mapping.

RB toggles a locomotion blend preview. In that mode the left stick's magnitude
selects an inspection speed from zero to the main graph's run velocity (4.5),
and the right stick orbits. Shared phase and velocity-weighted local poses follow
the recovered standalone driver structure. This stick mapping is an inspection
control; it does not implement the game's steering/mode selection or actor travel.
A/B returns to single-clip mode; X pauses, Y resets phase and LB slows the preview.
Keyboard L toggles the same mode. `--loco-speed 3` starts a fixed inspection speed
with equal walk/run weights. This cannot be combined with `--sample-frame`; RB
discards the fixed-speed override. `Animate.ps1 -LocomotionSpeed 3` also supports it.

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
92-joint skeletons. Run is at decoded chunk offset **0x77176e**, frame count 21
(last frame 20),
30 fps, with 62 joint groups. Walk is at **0x796c9f**, frame count 37
(last frame 36), 30 fps;
`heller_amb_stand` is at **0x667b70** and exercises multiple compressed-data blocks.
Jump is at **0x75a1d6**, sprint at **0x775ef6**. These offsets identify this corpus.
The main bare LocoSteer record at **0x51495** in `alex_fig.p3d` resolves idle,
walk and run hashes to `alex_amb_stand` (**0x3f343**), `heller_loco_walk_n` and
`heller_loco_run_n`. Playback loads that trio from the actual graph references.
`heller_amb_stand` is also independently validated and selectable by `--clip`.
The main track stores phase -1, all three sync frames -1, blend-in approximately
0.133333 seconds and blend-out 0.3 seconds. Native begin **0x101e37f0** resolves
the referenced animations and constructs/reuses a locomotion driver; its phase,
sync and update policies beyond the standalone subset below still require recovery
before implementing transitions.

The complete header inventory is **741 PTRN skeletal patterns, 166 CAM camera
patterns and 24 EXP expression patterns**. All 741 skeletal patterns now decode;
every authored track and both selected rigs were sampled at frame zero, midpoint
and last frame. Some patterns belong to other actors/props and share few or no
joint names. This check establishes reader/sampler coverage, not correct actor
binding, gameplay reachability or camera/expression playback.

Bare punch and kick chunks are **0x6a361b / 0x68d6a9**, last frames 88 / 125.
Blade activation is **0x7a62bf**, last frame 192; devastator attack is **0x6c8989**,
last frame 200. These can be inspected independently of the action scheduler;
weapon/power drawables are not assembled in this character preview.

`art.rcf / art\alex\alex_model_main.p3d` supplies selected body/arm geometry.
Nine supported skin primitives contain **7,978 vertices / 12,463 triangles**.
Body and arm skeletons are joined by exact drawable references; a separate
left-arm skeleton/drawable is not selected. This is not complete model assembly.

## Checked layouts

| Chunk ID | Implemented interpretation |
| --- | --- |
| 0x23000 / 0x23001 | Skeleton v1 header and joints: names, ordered parent indices, 16-float affine local bind matrix, twelve additional finite floats, u16/u32 tail |
| 0x121000 | Animation v0 name, kind, positive finite frame count/fps, cyclic flag |
| 0x121001 | Joint group v0 name, joint index and direct channel count |
| 0x121402 | Direct v0 default sync-frame child; finite f32 offset, native zero default when absent |
| 0x2f00000 | Version 0 ZLIB blob with declared decoded/stored lengths; complete checked stream, maximum 64 MiB |
| 0x121010 | Version 0 block table, inspected 8192 marker, checked cumulative block sizes covering the blob |
| 0x121121 | Version 0 referenced key count, offset and block index; frames followed by values aligned to four-byte absolute blob position |
| 0x121112 / 0x121114 | ROT v1 signed i16/i8 XYZ, reconstructed positive W |
| 0x121104 / 0x121119 | TRAN or SCL v0 float3 / IEEE binary16 triple |
| 0x121102 | TRAN or SCL v0 scalar replacing one component of a supplied base vector |
| 0x121103 / 0x121118 | TRAN or SCL v0 float2 / binary16 pair replacing the other two components |
| 0x121110 | Inspected disabled channel metadata: v0, zero flag, exactly eight bytes and no children; active/nested metadata remains unsupported |
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
Half expansion is corroborated by **0x10845370 / 0x1080eab0**. The vector reader
**0x10843620** retains the channel kind, so these layouts also carry `SCL\0`.
Native ordinary pose blending **0x1062f600** routes both translation and scale
through the linear component blend **0x1062dfe0**.

v0.1.6 and earlier skipped authored scale channels. v0.1.7 decodes and samples
them, retains bind scale only when an authored channel is absent, and includes
scale in local pose blending before parent composition. The corpus contains 580
scale channels across 35 skeletal clips: 540 half3, 37 scalar and three half2.
For example, `heller_shield_brace2idle` at **0x7bd7f4** supplies constant scale
**0.01000213623046875** on `Index_L` and other hand/arm joints. Shrinking those
joints is authored behavior; missing shield/weapon assembly is a separate limit.
Hammerfist acquisition also carries scale tracks on power-specific joints outside
the two selected base rigs. Such channels are decoded but cannot deform absent rigs.

Five skeletal clips previously failed on direct child **0x121110**. Its inspected
v0/zero-flag form leaves native channel flag bit 0x40 clear, as corroborated by
**0x108421a0**; it does not replace or erase the referenced keys. This empty form
is accepted with strict size/version/flag/child/uniqueness checks. Active flags,
nested values and channel-name overrides remain unimplemented and fail.

Skin streams use inspected 80-byte records: float4 positions/normals/weights and
u16x4 joint indices. Both position and normal fourth lanes are **1** in these
streams. Weight sums, joint/index bounds and descriptor byte ranges are checked.
Bind-pose deformation reproduces input positions within 0.0001 units.

## Sampling and visible limits

The core samples an explicit finite frame, clamps outside individual key ranges,
linearly interpolates translation/scale and uses normalized shortest-arc quaternion
SLERP. Missing tracks preserve bind transforms. Parent composition produces joint
world matrices; CPU skinning uses `sampled_world * inverse_bind_world`. Synthetic
two-joint fixtures independently verify a known halfway rotation, nonuniform scale
and deformed point; a child position verifies scale blending before composition.
The native interpolator uses approximations different from glam: bitwise matching
and full blending behavior have not been demonstrated.

The viewer cancels the sampled `Motion_Root` world transform and retains the bind
root, so clips can be inspected upright in place. Raw root transforms remain in
the decoded clip. This preview policy is not recovered gameplay root motion.
Presentation delta time advances frames at the asset fps, with a 0.1-second cap;
inspection loops even non-cyclic clips. Neither that clock nor looping policy is
the retail scheduler. Asset fps does not identify the simulation tick.

Native wrapper bind **0x10621730** copies the serialized frame count and fps,
then sets its last frame to count minus one. v0.1.3 labeled the count as an end
frame and looped over that longer interval; v0.1.4 corrects the field, key bounds,
fixed-pose range and preview interval. Run's cycle is **20/30 seconds** and walk's
is **36/30 seconds**, before any locomotion-driver rate adjustment.

The independent `animation_driver::advance_phase` primitive now implements the
inspected update **0x1062a350** for three distinct, available, nondirectional
idle/walk/run stages. It selects adjacent velocity stages, linearly weights them,
drops candidates below 0.01 and normalizes remaining weights. Cycle duration and
reference velocity are weighted from the surviving stages. Positive reference
velocity scales supplied dt by current/reference speed; phase advances by that
scaled dt divided by the weighted cycle duration. Zero duration leaves phase
unchanged; out-of-cycle phase wraps with the native 0.00001 boundary tolerance.
The 0.01/0.00001 constants occur at **0x10aa7678 / 0x10aa7674** in this binary.
Constructor **0x1062a1a0**, stage registration **0x10629f40**, candidate insertion
**0x10629da0** and clip wrapper bind/getters **0x10621730 / 0x10317410 /
0x1025b140** corroborate the stage and duration contracts.

Tests cover idle/walk boundaries, halfway walk/run weights, faster-than-final-stage
playback, candidate trimming, cycle boundaries and invalid input. An installed-data
check joins the main graph's actual three clips and verifies a 0.12-second walk
step reaches phase 0.1. This primitive accepts explicit seconds and velocities;
it does not choose an input mode or tick or execute states.
Aliases/missing clips, directional candidates, sync/event channels, layers and
retargeting remain separate policies. Native arithmetic is not bitwise reproduced.

`frame_at_phase` implements the inspected phase mapping **0x1062ab30**: default
sync offset plus phase times last frame, wrapped over [0,last frame) with the
0.00001 boundary tolerance from **0x100690d0 / 0x109f02fc**. The native clip
constructor **0x108120e0** defaults sync to zero; loader **0x10812230** overrides
it from direct child 0x121402. All 931 inspected headers have one v0 child containing
zero. Main idle/walk/run child offsets are **0x40822 / 0x7993ea / 0x773b39**.
The preview requires the main track's all-negative sync array and uses asset
defaults. Explicit track overrides and event-driven rephasing remain unsupported.
Tests cover nonzero/negative offsets, wrap boundaries, zero length, overflow and
malformed/duplicate sync children. The installed walk check maps phase 0.1 to
frame 3.6. These are explicit data/math primitives, not gameplay transitions.

`sample_locomotion` now samples adjacent idle/walk/run stages at that shared phase.
Native candidate ordering **0x10628490 / 0x10628360 / 0x10628210** puts greater
weights first and preserves equal weights for this small candidate list.
Evaluator **0x1062aee0** accumulates weight and blends each following pose by its
weight divided by the accumulated total. Command writer **0x1062f910** and skeletal
dispatch **0x1062de40 / 0x1062f600** connect that coefficient to local-component
blending. Translation/scale helper **0x1062dfe0** is linear; rotation helper
**0x1062e110** uses a normalized spherical approximation and flips the incoming
hemisphere for dot product <= 0. Missing-channel fallback **0x1062bbc0** uses
base components. This standalone Rust contract supplies bind components as base,
uses glam SLERP with the inspected hemisphere rule, and composes parents afterward.
Native SIMD polynomial approximation differences remain; this is not bitwise parity.
Aliases, nonadjacent weights, partitions, additive layers and retargeting are outside
the primitive. Independent fixtures check a known blended child position, endpoints,
antipodal rotations, zero-dot behavior and overflowing composition. Installed-data
checks sample six speeds on both 92-joint skeletons and reproduce the walk endpoint.

## Authored root-motion extraction

`root-motion --clip heller_loco_run_n` now reports the original `Motion_Root`
track's displacement/rotation from frame zero to last frame. Explicit endpoints
use `--previous-frame` and `--current-frame`; `--relative-root true` chooses
segment-start orientation coordinates, and `--reverse true` applies the native
endpoint-swap/output-reversal policy. The command is available in core-only builds.
It does not infer a tick, integrate a character or use a contact solver.

The independent `root_motion::delta` primitive follows translation consumers
**0x1062cc30 / 0x1062ccd0**, rotation consumer **0x1062cb00** and dispatcher
**0x1062ced0**. For ordered endpoints it subtracts authored translation and computes
inverse-start times end rotation in glam's convention. A decreasing endpoint
crosses the loop boundary once: combine the previous-to-last and first-to-current
segments. Relative translation rotates each segment by that segment's inverse
starting orientation. Rotation combines the second segment times the first;
reverse swaps endpoints before this calculation, then negates translation and
conjugates rotation. Equal endpoints mean zero motion, not a complete cycle;
endpoint pairs cannot represent multiple crossings. Clip/scheduler policy remains
responsible for deciding the actual interval and flags.

Quaternion product **0x107e93f0** uses a row convention that corresponds to reversed
Hamilton product order; relative helper **0x107e9fe0** conjugates its second input.
Vector rotation **0x107e9ca0** corroborates the orientation-coordinate conversion.
Noncommuting synthetic rotations check this order independently by their action
on basis vectors. Wrapped/cropped/reversed intervals, missing channels, malformed
keys, finite bounds and overflow are tested. Missing authored channels remain
distinct from skeleton bind fallback. Native key samplers return the first value
directly at zero interpolation fraction; the Rust channel sampler now does too,
avoiding an unused overflowing difference at a valid extreme endpoint.

Measured full-clip translation in the installed corpus (asset units):

| Clip | Raw XYZ | Segment-relative XYZ |
| --- | --- | --- |
| `alex_amb_stand` | (0,0,0) | (0,0,0) |
| `heller_loco_walk_n` | (0,0,-1.659999967) | same |
| `heller_loco_run_n` | (0,0,3.975730419) | (0,0,-3.975730419) |
| `heller_loco_run_sprint_n` | (0,0,-4.999793053) | same |
| `heller_loco_jump_from_idle` | (0.061584473,1.041015625,-1.624023438) | same |

Rotation deltas are identity within 0.00001 for these endpoint pairs. Run's
authored root orientation causes its raw/relative Z sign difference; changing
quaternion ordering to hide it would be incorrect. Clip distance divided by
duration is not a confirmed gameplay speed: the main graph supplies explicit
walk/run velocity references (1.5/4.5). Jump's full-clip endpoint and duration do
not establish gameplay jump height or jump timing. Playback still uses the
root-neutral preview; actual interval selection, blended root-driver evaluation,
actor integration, contacts and original-game trace comparison remain incomplete.

## Heller corrective pose strategies

The preview now reads `boot.rcf / art\alex\alex_tod.p3d` and applies the two
strategies in `HellerPoseFixupProperties` after clip/blend evaluation and before
root removal/skinning. The object definition is at **0x36a**, META body at
**0x3f1**, 233 bytes. `TransformationDrawable` and `PrototypeTemplate` reference
this object at **0x78e / 0x7235** respectively. These are selected object references,
not proof that the runtime executes the full original behaviour/state lifecycle.

The nested v1 `PoseStrategyCollarRig` layout token is **0x08c2bd93**. Fields are
left/right collar and chin names, chin offset float3, displacement power and maximum
projected displacement. The installed values are `Collar_L`, `Collar_R`, `Jaw`,
(0,0,-0.1), 5 and 1. `PoseStrategyShoulderCon`, token **0x1a80a495**, supplies two
source/target pairs: `Shoulder_L` to `Shoulder_Con_L`, and the corresponding right
pair. Both nested entries have the inspected trailing markers 1/1. Their broader
activation policy is outside this preview: different markers fail, and these
configured operations are always applied during inspection.

Field registration **0x1016efe0 / 0x1016f290 / 0x101709c0**, joint binding
**0x1016e2b0 / 0x1016e320**, strategy updates **0x1016f8c0 / 0x1016e920** and
pose writer **0x106b2550** corroborate the layouts and operations. Behaviour dispatch
**0x10154210** iterates configured strategies; its actor activation/state gating
is not reproduced. Vector transformation **0x107e5ec0** excludes translation;
matrix copy/product **0x100019f0 / 0x107e7990** corroborate local/world composition.

Shoulder correction copies the source local pose into the corrective joint and
updates its world/skin pose under the target parent. The tested pairs share a
parent and have leaf targets, so the resulting world matrix equals the source.
Collar correction transforms the configured chin offset as a vector through the
chin parent's inverse bind orientation, then projects the current and bind chin
points into the left collar parent's coordinates. Let their Z difference be d,
bind-projected Z be r, and left/right bind translations be l and h. With power p,
the gain is `(1 + abs(d))^p - 1`. For d > 0, add `min(maximum, (h-r)*gain)` to
right bind Z. Otherwise subtract `min(maximum, -(l-r)*gain)` from left bind Z.
The other collar uses bind Z. Both use bind X/Y and retain current local orientation.
The clamp is an upper limit; a negative computed offset is retained.

The independent `pose_fixup` module validates bounded strings/counts/tokens and
complete payloads, binds exact joint names, and rejects unsupported/non-leaf or
different-parent rigs. It does not invent descendant propagation. Singular/nonfinite
poses, stale bindings and overflow fail. Synthetic fixtures verify copy/hierarchy,
both collar directions, upper-limit and negative-offset behavior, rotated chin
offsets, translation reset/orientation preservation and malformed layouts.
An installed-data test evaluates eleven clips on both selected skeletons and deforms
all nine skins. Exact power/inverse/SIMD rounding parity is unproven.

A same-frame run capture at 10.5 was visually compared with the earlier capture:
corrective shoulder geometry changes, but shoulder/hood overlap remains visible.
These two strategies do not complete the model or establish retail appearance.

Materials are diagnostic colors. Morph/cloth/expression assembly, additional constraints,
retail shaders, motion events, transitions, layered blends, grounded movement and
camera behavior remain unresolved. Some shoulder/hood geometry visibly overlaps.
Camera/expression patterns and their consumers remain unsupported.

Default playback contains seven clips; choosing an additional supported clip adds
it to the inspection bank. Eleven selected clips also pass fixup/skin evaluation.
The full skeletal corpus passes reader/sampling checks. Neither check proves the
main bare branch's retail reachability or complete model/power assembly.

Run frame 0 and 10.5 captures showed distinct deformed poses. A continuous walk
capture reached frame 14.959668, demonstrating advancement rather than a static
pose. XInput slot 0 was detected; synthetic edge tests cover A/B and held/focus
behavior. Additional fixed-frame idle, sprint and jump captures were visually
inspected, with distinct standing/running/airborne poses and unresolved cloth parts.
The corrected v0.1.4 continuous walk capture reached frame 15.784990 and the actual
window controls passed again after the loop interval correction.
The owner has not physically tested the new animation-specific mapping.
v0.1.7 additionally captured punch frames 0 / 30, kick 30, blade activation 60,
shield transition 7.5 and continuous punch at frame 15.751035. Distinct poses were
visually inspected; shoulder/hood overlap and missing weapon meshes remain visible.
Actual-window automation exercised the seven-clip bank, including punch/kick and
wrap in both directions, alongside pause/reset/blend-mode behavior and exit.
Automated keyboard scan-code input in the actual focused viewer verified next and
previous clips, a stable paused frame, reset to frame zero and successful exit.
That test does not simulate or establish physical Xbox input.
Two shared-phase blend captures were also inspected: speed 3 reached phase
0.562845 with weights 0/0.5/0.5; speed 0.75 reached phase 0.312983 with weights
0.5/0.5/0. The renderer showed distinct striding and upright poses. Actual-window
automation additionally verified blend-mode entry, phase advancement, paused phase
stability, phase reset and return to clip mode. Synthetic XInput tests cover RB
connection/focus/held-edge behavior. Physical RB/left-stick use remains untested.
See VALIDATION.md for the combined test record and STATUS.md for next work.
