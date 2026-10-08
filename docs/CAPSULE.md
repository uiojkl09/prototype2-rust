# Recovered asset capsule and independent geometric queries

This milestone recovers an asset structure and implements a reproducible collision
investigation tool. It does not implement character movement or the retail contact
solver. The target remains faithful Prototype 2 behavior, including Xbox controls.

## Evidence: installed Steam build 19788008

Binary identity is in VALIDATION.md. Preferred image base of the inspected x86
engine DLL is 0x10000000; the addresses below are preferred virtual addresses, not
live process addresses. The DLL was inspected read-only and was not loaded/called.
Only authored descriptions and Rust source are published; disassembly stays private.

`ravenphysics::CollisionCapsuleFactory` reflection registration at 0x108eb630 defines
four ordered fields: `centre`, `axis`, `extent`, `radius`. Its base registration at
0x108eb4e0 defines `material` and `intersectionProperties`; **Character is a reference
to intersection properties**, not a decoded collision group/mask. Shape construction
at 0x108d76e0 copies its values into the capsule product. The segment endpoint routine
at 0x107ee890 computes centre minus/plus axis times extent; the bounds routine at
0x107eec10 expands the segment bounds by radius. This corroborates extent as half
the segment length and radius as spherical expansion, rather than guessed labels.

Observed `boot.rcf` / `art\alex\alex_tod.p3d`, AlexPhysicsFactory at 0x3afa, body
0x3b9d, 128 bytes:

| Body-relative offset | Observed structure/value |
| --- | --- |
| 0x00 | META signature |
| 0x04 | u32 type-name length 37, then CollisionCapsuleFactory qualified name |
| 0x2d / 0x2f | u16 version 1 / observed u32 token 0x9ca40a36 |
| 0x33 | u8-length material reference AlexFrictionlessFlesh |
| 0x49 | u8-length intersection-properties reference Character |
| 0x53 | centre float3 (0, 0.175, 0) |
| 0x5f | axis float3 (0, 1, 0) |
| 0x6b | extent float 0.175 |
| 0x6f | radius float 0.5 |
| 0x73 | 13 remaining SimplePhysicsObjectFactory bytes, uninterpreted |

These dimensions are in uncalibrated game units. Their occurrence in this factory
does not establish which locomotion states use them or their runtime transform.
Fight-data capsule tracks and state-dependent changes still require decoding.
The reader accepts only the inspected envelope/type/version/token and tail length,
checks finite dimensions/unit axis and rejects truncation or unsupported layouts.
The token's hash algorithm/meaning is not claimed recovered.

## Reproduce from the player's installation

```powershell
.\prototype2-rust.exe capsule --game 'F:\SteamLibrary\steamapps\common\Prototype 2'
.\prototype2-rust.exe sweep --game 'F:\SteamLibrary\steamapps\common\Prototype 2' --origin 1218.130,55.001,-1898.738 --delta 0,-50,0
```

`--origin` is the factory's translation; `--delta` is a finite displacement, not
velocity or elapsed-time input. The command reads the factory from boot.rcf and
ground triangles from the selected cell. It reports the earliest geometric contact
fraction, point, normal, initial overlap, chunk, face and retained unknown face tag.
No extracted payload is needed or written. Other cells use existing archive/entry
options. A null hit means no geometric contact on that segment, not a clear retail
path: props, other primitives and filtering are still unresolved.

## Query implementation and limits

The original Rust query computes the closest points between the capsule segment
and each candidate triangle, then uses conservative advancement under translation.
The swept bounds reject distant faces. Double precision reduces cancellation in
queries at large world coordinates. It handles face, edge and vertex contacts,
both windings, tangent/separating contacts and initial overlap. Iteration is capped;
nonconvergence is an error rather than a false clear path. Synthetic tests cover
thin-wall high-speed traversal, capsule-side/edge hits, two-sided geometry, tangent
floor travel, downward contacts, bad dimensions and malformed indices.

This is an independent geometric algorithm, **not a recovered retail solver**.
It ignores tags and material/property masks, handles no rotation during the sweep,
and performs no depenetration, grounding, slope/step logic, wall slide or integration.
It scans candidate faces rather than using the unknown serialized spatial tree.
The CLI uses zero skin. The library's optional skin and convergence tolerances are
numerical query settings, not original Prototype 2 gameplay constants.

On the inspected real patch, the downward sweep contacted near y=43.000556 at
fraction 0.2300088813, with upward normal approximately (0,1,-0.00002727), no initial
overlap, triangle chunk 0x192ab3c, face 120, tag 2. Its travelled distance agrees with
the existing ray result minus radius within 0.002 game units. This validates one
geometric query against the real section; it is not an original-engine comparison.

Next recover fight-node serialization and active capsule/locomotion transitions,
intersection-property filtering, original input/timing and movement trajectories.
Walking/jumping can then be implemented and compared without substituting guessed
speeds, gravity, camera behavior or a stock controller for the original rules.
