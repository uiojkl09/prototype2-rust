# Validation record — 2026-10-08

This record describes completed local checks and their limits. It does not claim
the first gameplay milestone is complete.

## Game identity

Steam app **115320**, installed build **19788008**. Depot manifests read locally:
115321 = 463566073465694220; 115322 = 9048180919412686294;
115323 = 7756741017743835184. Full manifest/account information is not published.

| File | Bytes | PE machine | COFF timestamp | SHA-256 |
| --- | --- | --- | --- | --- |
| prototype2.exe | 3,241,472 | 0x014c (x86) | 1756404256 | 1d1f1e15a7630552387c3459bf03b349954996363b575bee260340aa9358dfe2 |
| prototype2engine.dll | 15,517,184 | 0x014c (x86) | 1756404282 | c3d51d6355387963a63cbac058381b8fd1ce39f51088fa50276a49285ee94174 |

No FileVersion/ProductVersion resource strings were returned. PE timestamps identify
the August 2025 binary revision; hashes and the manifest are the authoritative
record for these tested files. The new executable targets x64 MSVC independently.

## Completed checks

| Check | Observed result | What it establishes |
| --- | --- | --- |
| Seven synthetic tests | Passed | Malformed/truncated/nested P3D, RZ size/checksum/trailer/cap, RCF name/range/alignment rejection and geometric nearest-ray behavior |
| v0.1.1 controller/metadata tests | Five additional tests passed (12 synthetic total) | Analog stick/dead-zone behavior, button edges, focus/disconnect/reconnect/multiple pads/inversion; metadata reference offsets and malformed envelope lengths |
| Capsule/collision prerequisite tests | Seven additional tests passed (19 synthetic total) | Checked body identity/truncation/dimensions, fast wall sweeps, both windings, tangent floor, edge/side contacts, overlap, bad indices and degenerate/endpoint contacts |
| Actual character factory | Recovered centre (0,0.175,0), axis (0,1,0), extent 0.175, radius 0.5 | Inspected asset schema corroborated by native field registration/geometry; active locomotion use still unknown |
| Actual swept-capsule query | Third opt-in retail test passed; floor contact fraction 0.2300088813 on a 50-unit downward translation | Capsule query against real patch agrees with ray-minus-radius within 0.002 units; no retail collision-response comparison |
| All RCF indexes | 12 archives; 26,385 entries passed | Metadata, hash joins, bounds, alignment, uniqueness and non-overlap |
| Full cells.rcf scan | 2,969 P3D files; 551,832,928 stored bytes; 1,219,150,189 decoded bytes; 811,757 chunks | Every selected archive payload decoded; complete structural traversal passed |
| Scan timing | 29.403 seconds in an unoptimized core build | A single local scan measurement, not runtime performance |
| Selected yellow-zone Cell_29 | 2,210 chunks; 38 meshes; 177,596 vertices; 185,586 triangles | Native position/index decoding and reference validation |
| Selected ground geometry | 60 chunks; 20,075 faces; 232,810 unknown trailing bytes | Candidate ground geometry decoded, tags/tail still unknown |
| Opt-in retail test | Passed; 66 of 81 grid rays hit | Actual section triangles are queryable; no claim of retail collision filtering |
| Additional section summaries | green_zone/Cell_18: 24 meshes, 74,244 triangles; yellow_zone/Cell_49: 33 meshes, 74,949 triangles | Limited merged-buffer decoder works on additional real cells; these are not fully assembled scenes |
| Native viewer | Ran, captured frames and exited successfully | Real meshes reached the GPU/window and produced nonempty frames |
| Formatting / Clippy | Passed with warnings denied | Source hygiene checks for all compiled targets; not semantic correctness |

Controller update: the native Windows API detected an Xbox/XInput device in slot 0.
Its neutral raw readings (-232,159) / (909,129) yielded zero processed travel/look.
The viewer logged the connection, and the owner reported that the listed controller
controls work with no drift. Physical disconnection/focus and other device/transport
combinations remain separate playtest checks; automated state-transition tests pass.
The metadata inspector reproduced the two character/startup leads in MOVEMENT.md.

The v0.1.0 hosted CI build/check step passed, but its packaging step failed while
loading Cargo JSON passed through PowerShell. v0.1.1 reads that metadata directly
from Cargo in Python, avoiding the native text pipe. Local packaging is checked;
v0.1.1 main hosted build, package and artifact-upload steps subsequently passed.
Consult GitHub Actions for later hosted results.

The viewer was checked with local screenshots, which remain outside the public
repository/package. Screenshots reveal plain debug geometry and omitted dressing;
they do not establish fidelity to retail appearance. Initial camera placement was
adjusted after a roof obscured the overview. Camera placement remains heuristic.

Host inspection reports **Windows 11 Home 10.0.26200**, RTX 4070 Ti, driver
32.0.16.1692. The brief named Windows 10, so that remains untested. The 1600x900
logical window captured at 3200x1800 physical pixels under host DPI scaling. A 4K
performance target, repeated interactive playtest and other CPU/GPU setups remain
unverified. No retail executable/DLL was modified or executed by the new runtime.

## Compatibility investigation ledger

| Question | Evidence so far | Acceptance / missing work |
| --- | --- | --- |
| High-core-count startup issues | Historical Prototype 1 launcher workaround only; not a Prototype 2 reproduction | Measure current retail build on actual CPUs. New runtime must start/exit repeatedly at default affinity. No root cause claimed. |
| Camera stutter / HID interaction | Prototype 1 notes, no current Prototype 2 measurement | Record camera/frame pacing and controlled input on original and rewrite. No fix claimed. |
| Simulation speed changes with frame rate | No recovered Prototype 2 tick or movement trace | Run identical replay input at multiple presentation rates; compare states and trajectories with original. |
| Binary/data build mismatch | Current files are individually fingerprinted; other versions not tested | Require identity in bug reports and validate readers against each supported corpus. |
| Large world / 4K stability | One section rendered, no streaming or long-run trace | Establish CPU/GPU memory budgets and frame-time distribution on the full target workloads. |

Reader limits and independent x64 execution address controllable architecture
properties. They do not demonstrate that all original engine problems are fixed.
