# prototype2-rust

An independent 64-bit Rust reimplementation project for **Prototype 2**, starting
with native retail-data readers and an experimental world-section viewer.

**This release is an investigation milestone, not a playable game.** It loads real
merged geometry from `yellow_zone/Cell_29`, shows it without textures, and can display
recovered ground-collision triangles. Free-camera controls do not implement Heller's
movement. Missions, powers, combat, AI, animation, audio and progression are absent.

Players provide their own installed copy. The program reads its RCF archives in
place; it does not load the retail executable or engine DLL. No game assets ship here.
The long-term target is faithful gameplay with reliable modern hardware support.
[STATUS.md](STATUS.md) records what has and has not been achieved.

## Try the build

Download the Windows package from this repository's Releases, extract it into a
writable directory, and open PowerShell there:

```powershell
.\prototype2-rust.exe view --game 'F:\SteamLibrary\steamapps\common\Prototype 2'
```

**Xbox controller:** left stick to move, right stick to look, A/B to rise/descend,
LB for faster flight, X for the orange collision overlay, Y to reset, and Menu/Start
to exit. Native Windows XInput works without a keyboard mapper. Analog speed,
stick dead zones, connection changes and window focus are handled. See
[controller preferences and checks](docs/CONTROLLER.md).

Keyboard remains available: WASD, Q/E, arrows, Shift; C collision, R reset, Escape exit.
This camera passes through walls. Debug colors replace retail materials.

To retain a diagnostic log, use the packaged script:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\Run.ps1 -GameDirectory 'F:\SteamLibrary\steamapps\common\Prototype 2'
```

The package contains `source.zip`, complete build instructions and license notices.
See [testing instructions](docs/PLAYTEST.md) for exact checks and reports.

## Build from source

Rust 1.97.1, the x64 MSVC C++ tools and Windows SDK are required. The toolchain and
Cargo dependency graph are pinned. No Python, Blender or decompiler is needed to run.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\Build.ps1
.\target\x86_64-pc-windows-msvc\release\prototype2-rust.exe view --game 'F:\SteamLibrary\steamapps\common\Prototype 2'
```

[BUILD.md](docs/BUILD.md) covers reproducibility, core-only builds and packaging.
Windows 11 with an RTX 4070 Ti has been exercised; Windows 10 remains a validation
target, not a completed compatibility claim.

## Investigation commands

All commands accept `--game`, or the `PROTOTYPE2_GAME` environment variable.

| Command | What it verifies or displays |
| --- | --- |
| `inspect` | Executable/DLL hashes, PE machine/timestamps and Steam build ID |
| `verify` | Every RCF index, name/hash association, alignment and payload bounds |
| `scan --archive cells.rcf` | Decode every P3D entry and validate its full chunk tree |
| `list --filter Cell_29 --limit 20` | Matching entries, offsets and stored sizes |
| `chunks --limit 30` | Bounded structural listing and chunk-ID histogram |
| `meta --filter CollisionCapsuleFactory` | Checked metadata envelopes and reference offsets; body schemas remain unknown |
| `scene` | Selected cell's geometry/collision counts, bounds and limitations |
| `ray --origin 1266,80,-1932 --direction 0,-1,0` | Experimental geometric collision query |
| `view` | Standalone Bevy viewer of the selected cell |
| `controller --seconds 15` | Xbox/XInput raw and processed input diagnostics; no game path required |

Use `--archive` and `--entry` to select a different file. Empty cells and unsupported
scene layouts produce errors. Reader support is broader than scene interpretation.

## Evidence and contributions

Read [format evidence](docs/FORMATS.md), [research assessment](docs/RESEARCH.md),
[architecture](docs/ARCHITECTURE.md), [validation](docs/VALIDATION.md) and
[movement comparison protocol](docs/MOVEMENT.md). Small changes should identify the
data/build tested and preserve reader validation. Do not attach game files or dumps.

Original project code is MIT licensed. [THIRD_PARTY.md](THIRD_PARTY.md) records source
lineage and notices. Prototype 2 assets and trademarks belong to their owners; this
project is unaffiliated with the game's developer and publisher.
