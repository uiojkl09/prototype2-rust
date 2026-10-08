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
| Animation | Bevy provides animation infrastructure. Pure3D skeletons, constraints, compression, root motion and transition behavior need readers/contracts first. |
| Simulation | Keep original rules in a renderer-independent explicit step, fixed input timeline and replayable state. A stock physics/controller must not become the specification. No gameplay simulation exists yet. |
| Performance | One section has run on the owner's RTX 4070 Ti. No 4K frame-time or multi-CPU guarantee is established; collect traces before changing architecture. |
| Mods | Clear format products and authored Rust systems are inspectable. Stable mod API, overrides and sandbox policy are future work. |

The viewer never calls retail DLL functions. It reads and presents data itself.
Its camera advances using presentation delta time; that is a viewer convenience and
must not be reused as the character simulation clock. No retail tick rate is assumed.

The design was informed by [IW4L's asset-loading separation](https://github.com/vladtrc/iw4L/blob/a2f4e0b8573eeb828001f8ee757b5b724eee196f/docs/MAP-LOAD.md)
and explicit simulation input/time. None of its game-specific engine code is reused.
