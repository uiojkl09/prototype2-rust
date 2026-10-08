# Character fight graphs and initial movement rules

Evidence: Steam build 19788008; binary identities are in VALIDATION.md. The owned
`boot.rcf` entry `art\alex\alex_fig.p3d` is 1,199,077 decoded bytes. The Rust reader
loads it directly without extracted fixtures or an editor/runtime dependency.

## Checked serialization

FightDefinition 0x20000701 holds a string8 name, u16 version 1, string8 context type
and an unresolved u32 definition token. Its sole FightData 0x20000702 child holds
a u32 length followed by exactly that many bytes. The inspected bodies begin with
`fig0`, u32 flags 5, u32 zero and a tagged root record. A record is a u64 type token,
u32 payload length, bounded payload and an eight-byte zero trailer. The root token
is 0x616fe21d386330ff, matching the hash of `chunk`.

Root payload: u32 version 1, u64 context-name hash, u64 context-type hash, a root
branch reference, u32 declared branch count including the implicit root, then nested
branches and a zero terminator. A branch starts with a u64 name hash, string32 path
and an observed sibling-count u32. Reference branches add a branch reference;
content branches add a boolean u32. Property groups contain a u32 entry count and
tagged entries. Other supported branch kinds have no additional prefix in this
corpus. Unsupported branch/group kinds fail with token and offset, rather than
silently discarding children. Most condition/action bodies remain opaque spans.

String32 stores a u32 UTF-8 byte length followed by the bytes and zero padding to
a multiple of four. This differs from P3D string8: padding is outside the length.
A branch reference is string32 plus i32 index. Index -1 is observed; reference
resolution and action scheduling are not yet implemented. Record reuse entries
are retained but are not treated as executable actions.

Case-preserving name hashing uses wrapping u64 `h = (h * 65599) XOR byte`, starting
at zero. Labels resolve only from hashes of stored branch/reference components;
unresolved names remain hex tokens. `--filter bare/loco` can match ancestor hashes
without requiring a guessed label dictionary. Names alone do not prove a branch is
reachable or establish condition priority.

The inspected content-branch token is 0xf55e99665d47c7d3; it does not equal the hash
of the descriptive type label `contentNode`. Type tokens must not be synthesized
from all editor class names. Limits cover bytes, strings, nesting, derived paths,
branch/property counts, ranges, padding, trailers and finite numeric fields.

| Context | Data chunk offset | Branches, excluding implicit root | Property records |
| --- | --- | --- | --- |
| disguise | 0x49 | 59 | 220 |
| prototype | 0x4efd | 1,322 | 3,295 |
| prototype_air | 0x6a099 | 463 | 1,649 |
| prototype_consume | 0x95409 | 559 | 2,138 |
| prototype_sprint | 0xf979d | 328 | 983 |
| prototype_whipfist | 0x113265 | 199 | 696 |

All six contexts consume their full spans, including 2,930 branches and 8,981
property records. Opaque records include conditions, commands and reuse references;
this total is not an animation or action count.

## Main movement evidence

The main bare locomotion branch contains a `locoSteer` record at 0x51495, with
132 payload bytes. Native field registration at preferred-image VA 0x10297160
corroborates the ordered property names. Setter/consumer inspection identifies:

| Field | Stored value | Native evidence |
| --- | --- | --- |
| acceleration | 15 | Setter 0x1027f760; update 0x101e3e50 limits speed change by acceleration times dt |
| turningVelocity | 360 degrees/s | Setter 0x102c64d0 converts to radians; turn helper 0x102227b0 |
| turningVelocityRun | 720 degrees/s | Setter 0x10296e50 converts to radians; same turn helper |
| velocityWalk | 1.5 | Ordered property; action initialization 0x101e37f0 and scalar helper 0x101e3370 |
| velocityRun | 4.5 | Same initialization/helper; units remain uncalibrated |
| smoothSteeringAngle | 0 | Ordered property; optional angular smoothing is disabled in this track |
| blendInTime / blendOutTime | approximately 0.133333 / 0.3 seconds | Track values; animation-driver behavior remains incomplete |

The scalar helper has two modes: an internal flag selects a midpoint of 0.5;
otherwise the split is min(walk/run, 0.99) when run is positive. Upstream component
selection and intention processing remain unresolved, so the library accepts this
flag explicitly. Between zero, split and full input, speed follows the inspected
piecewise linear curve. Above full input the helper multiplies run speed by the
magnitude; input clamping belongs upstream. Positive acceleration limits increases
and decreases symmetrically; a nonpositive limit snaps to the target.

For the inspected track's zero-angular-acceleration case, the turn-rate bound blends
from walk to run by clamp(current_speed/run_speed, 0, 1). The shortest signed angle
uses the native [-pi, pi) wrap and 0.00001 boundary tolerance. Functions in
`locomotion.rs` implement these isolated rules with caller-supplied seconds. They
do not choose a retail tick, map Xbox input, integrate position or solve contacts.
Retail x87 intermediate arithmetic and Rust floating-point operations have not been
shown bit-identical; original-game trajectory comparison remains required.

The single `locomotion` record at 0x68aa5 is in `new_feature_testing/locomotion`.
Its 1.4/6 stage speeds and other testing parameters are not vanilla walking values.
Main sprinting uses `LocoSprint`; three 208-byte records are now decoded, with native
field registration corroborated at 0x10296090. The default pose record at 0x107cc1
stores velocity stages 7/10/16, acceleration stages 1.5/0.75/1.25, deceleration 5,
turn-rate limits 360/90 degrees, turn-acceleration limits 4800/1200 degrees, lean
rate 5, and unlockable minimum velocity 9. It enables forceAnimationVelocities.
The human pose stores 8/8/8 and the cautious pose 6/6/6. These are asset parameters;
the action also applies upgrades, actor input modes and turn-dependent slowdown.
They do not establish measured travel speed or a complete sprint implementation.
Native actions begin/update at 0x101e1570/0x101e1c10 provide the next rule evidence.

## Capsule state changes

Fourteen 136-byte `physicsCollisionCapsule` records were decoded across these
contexts. Ordered schema: track reference i32, slave boolean u32, begin/end f32,
joint-name u64, animate boolean, offset flag/initial/final float3, extent
flag/initial/final f32, radius flag/initial/final f32, axis flag/initial/final float3,
rotation flag/initial/final float3. All flags use 0/1 u32 values. Disabled vector
fields may contain zero placeholders; they are not active shape axes.

Native registration 0x103e6ba0 and its setters corroborate the names and flags.
Begin 0x103e5480 captures the previous shape; apply 0x103e4e30 adds offset to its
captured centre, replaces enabled dimensions/axis and writes the shape. Update
0x103e56c0 samples elapsed/duration, clamped to 0..1, then advances elapsed after a
successful apply. Duration at or below 0.00001 selects the final values during
update. End 0x103e5950 restores the captured shape. Rotation setters convert degrees
to radians. Constructor 0x103e5410 requests updates for animation or joint attachment.

| Record offset | State evidence | Stored changes |
| --- | --- | --- |
| 0x519d5 | Main bare locomotion capsule branch | Animated over 0.2s; radius 0.7 to 0.5; centre offset (0,0,-0.1) to zero; extent retained |
| 0x1126d5 | Sprint capsule branch | Static radius 0.25, extent 0.4 and centre offset (0,0,-0.25) |
| 0x8fb4d | Medium landing pose branch | Static radius 0.7 and offset (0,0,-0.15); extent retained |

`capsule_state.rs` implements single-action snapshot/interpolation/restore for
unattached, unrotated shapes. Joint attachment and rotation fail explicitly.
Simultaneous action ordering, actor transforms and the baseline at each retail
state transition remain unresolved. Applying a track to the decoded default factory
in a test verifies this primitive; it does not prove the retail state uses that
factory unchanged. Shape interpolation alone does not recover contact response.

## Reproduce and continue

```powershell
.\prototype2-rust.exe fight --game 'F:\SteamLibrary\steamapps\common\Prototype 2' --filter bare/loco --limit 10
$env:PROTOTYPE2_GAME='F:\SteamLibrary\steamapps\common\Prototype 2'
cargo test --locked --no-default-features --test retail installed_fight_graph -- --ignored --nocapture
```

Selective Ghidra 12.1.4 analysis now supplies private cross-references and native
consumer evidence. All native addresses above use the engine DLL's preferred base
0x10000000, not live ASLR addresses. No engine code is loaded by the Rust program.
Decompiler output and original byte bodies remain outside the public repository.

Next connect resolved conditions/references to input and state selection, recover
simulation timing and actor/world transforms, then retail grounding/contact rules.
Continue through a controller-driven character slice and original-game comparison;
this reader/rule milestone is a prerequisite to that work.
