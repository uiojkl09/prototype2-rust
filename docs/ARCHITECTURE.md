# Architecture decision 0001: native readers with a replaceable viewer

The first executable has a renderer-independent Rust library (`rcf`, `p3d`, `scene`,
`install`) and a thin command interface. The optional Bevy adapter consumes CPU mesh
products. A core-only build verifies data without a window or graphics dependencies.

```text
player installation (read-only)
  -> bounded selected RCF payload
  -> checked RZ decode / Pure3D tree
  -> inspected geometry and collision products
  -> commands / geometric query tests / Bevy viewer
```

Bevy **0.18.1** is selected for the experimental viewer, not irrevocably for the
complete game. This is the locally available version that was pinned and exercised;
newer Bevy releases were not evaluated for this milestone. Its native windows,
input, GPU-backed mesh renderer and optional module structure are sufficient here.
[Official feature documentation](https://docs.rs/bevy/0.18.1/bevy/) supports a reduced
feature set. MIT/Apache-2.0 licensing permits this adapter in an MIT project.

| Requirement | Assessment / remaining proof |
| --- | --- |
| Large streamed world | Native mesh rendering works for one section. Cell lifecycle, streaming queues, draw budgets, LOD, instancing, visibility and origin handling still need evidence and profiling. |
| Faithful rendering | Current diagnostic colors are substitutes. Retail materials, texture maps, shader semantics, shadows and time-of-day are unimplemented. Bevy PBR is not an automatic match. |
| Animation | Core skeleton/packed clip sampling and skin deformation work; Bevy renders their CPU products. Retail transitions, root-motion integration, constraints, morphs and blending still need evidence. |
| Simulation | Keep original rules in a renderer-independent explicit step, fixed input timeline and replayable state. A stock physics/controller must not become the specification. No gameplay simulation exists yet. |
| Performance | One section has run on the owner's RTX 4070 Ti. No 4K frame-time or multi-CPU guarantee is established; collect traces before changing architecture. |
| Mods | Clear format products and authored Rust systems are inspectable. Stable mod API, overrides and sandbox policy are future work. |

The viewer never calls retail DLL functions. It reads and presents data itself.
Its camera advances using presentation delta time; that is a viewer convenience and
must not be reused as the character simulation clock. No retail tick rate is assumed.

The renderer-independent `controller` module reads documented Windows XInput samples
and converts them into inspection actions. The viewer combines those with optional
keyboard input, preserves analog magnitude, gates input on focus and probes inactive
controller slots periodically. This backend needs no Bevy gamepad plugin/dependency.
Gameplay input mapping/timing must be recovered separately. The `meta` reader exposes
checked object envelopes/reference offsets, leaving unknown body schemas opaque.

`capsule` now decodes the single inspected character factory using recovered native
schema/geometry evidence. `collision` provides an independent swept geometric query
for investigation, with no retail solver, integration or renderer dependency.
Commands retain unknown tags and explicitly report overlap/nonconvergence. Neither
module establishes the active locomotion hull or supplies gameplay constants.

`fight`, `locomotion` and `capsule_state` expose checked graph structures and isolated
recovered scalar/shape rules. They do not supply a scheduler or game simulation.
`animation` and `skin` use glam math without Bevy to read/sample/deform original
character assets. `animation_viewer` is a separate adapter with an inspection clock
and root-neutral preview policy; it must not become the retail movement contract.
See FIGHT.md and ANIMATION.md for evidence and remaining behaviors.

The design was informed by [IW4L's asset-loading separation](https://github.com/vladtrc/iw4L/blob/a2f4e0b8573eeb828001f8ee757b5b724eee196f/docs/MAP-LOAD.md)
and explicit simulation input/time. None of its game-specific engine code is reused.
