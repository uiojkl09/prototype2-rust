# Build and delivery

The verified toolchain is Rust/Cargo **1.97.1**, `x86_64-pc-windows-msvc`.
rust-toolchain.toml pins the compiler; Cargo.lock fixes dependency versions/checksums.
Bevy 0.18.1 and flate2 1.1.10 are also explicitly pinned. Build scripts use `--locked`.
The initial toolchain inspection found usable VS 2019 and VS 2026 C++ tools; no
replacement Visual Studio installation was required. A versioned Rust 1.97.1
toolchain was then installed matching the already-installed stable compiler.

## Windows

Install Rust through rustup if missing. Use Visual Studio's x64 MSVC C++ workload
and Windows SDK. The standard Rust build discovers the linker without opening an
IDE. From this source checkout:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\Build.ps1
.\target\x86_64-pc-windows-msvc\release\prototype2-rust.exe view --game 'F:\SteamLibrary\steamapps\common\Prototype 2'
```

The script checks formatting, core tests and Clippy before the x64 release build.
The user supplies a game path; no game files are copied or changed. Release packages
do not require a Rust installation. The current release uses native system/GPU
services; arbitrary Windows/GPU configurations have not been validated.

For investigation without rendering:

```powershell
cargo test --locked --no-default-features
cargo build --locked --release --no-default-features --target x86_64-pc-windows-msvc
```

An offline build requires the pinned toolchain and a previously fetched Cargo cache.
Run `cargo fetch --locked` while online, then use `--offline --locked` for Cargo.
The project is dependency reproducible; it does not yet claim byte-identical EXEs
across different MSVC/Windows SDK versions. BUILD-INFO.txt records a delivered
package's compiler, target and source commit; SHA256SUMS.txt identifies its executable.

## Retail checks

```powershell
$env:PROTOTYPE2_GAME = 'F:\SteamLibrary\steamapps\common\Prototype 2'
cargo test --locked --no-default-features --test retail -- --ignored --nocapture
.\target\x86_64-pc-windows-msvc\release\prototype2-rust.exe verify
.\target\x86_64-pc-windows-msvc\release\prototype2-rust.exe scan --archive cells.rcf
```

Retail tests are opt-in and absent from public CI. CI can verify source, parsers,
geometric query behavior and compilation with synthetic fixtures only.

## Package a reviewed commit

Python 3 and Git are packaging tools, not runtime dependencies. Ensure the executable
has passed the required checks and visual inspection, review/stage the source,
run `python scripts/audit-publication.py`, then commit. Use a fresh external path:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\Package.ps1 -OutputDirectory 'D:\Builds\prototype2-rust-v0.1.0'
```

The script requires a clean tracked source tree, audits the index, builds the exact
commit, packages the executable, full Git source archive, docs, Run.ps1, Animate.ps1,
project/third-party notices, dependency license files, build identity and hash. It accepts
only an output directory outside the source repository. Screenshots, logs, extracted
data, local references and dumps are not included. The source archive includes all
build/test scripts. Inspect the final package entries before upload.
