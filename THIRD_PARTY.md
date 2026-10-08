# Source provenance and third-party notices

The Rust code and synthetic tests were authored for this project. No proprietary
source, decompiled source or game assets are redistributed. Format reading was
informed by the projects below and checked against installed data. This is a new
Rust implementation, not the original Gibbed or RcfTools implementation.

## Gibbed.Prototype

[Rick / Gibbed.Prototype](https://github.com/gibbed/Gibbed.Prototype/tree/026da397342c4f8a43af575232dccf4b104df2e5),
revision 026da397342c4f8a43af575232dccf4b104df2e5. RCF fields, filename hashing,
metadata, RZ envelope and Pure3D chunk hierarchy informed the native readers.
Its permissive source notice is preserved here conservatively for this lineage:

> Copyright (c) 2012 Rick (rick 'at' gibbed 'dot' us)
>
> This software is provided 'as-is', without any express or implied
> warranty. In no event will the authors be held liable for any damages
> arising from the use of this software.
>
> Permission is granted to anyone to use this software for any purpose,
> including commercial applications, and to alter it and redistribute it
> freely, subject to the following restrictions:
>
> 1. The origin of this software must not be misrepresented; you must not
>    claim that you wrote the original software. If you use this software
>    in a product, an acknowledgment in the product documentation would
>    be appreciated but is not required.
> 2. Altered source versions must be plainly marked as such, and must not
>    be misrepresented as being the original software.
> 3. This notice may not be removed or altered from any source
>    distribution.

## RcfTools

[NixsonLai/RcfTools](https://github.com/NixsonLai/RcfTools/tree/636904bfcd9d5c136c377a624d89fe92634d336d),
revision 636904bfcd9d5c136c377a624d89fe92634d336d. Independent comparison of RCF
table offsets, metadata and hash behavior. Preserve its notice for the reader lineage:

> MIT License
>
> Copyright (c) 2021 Nicolás
>
> Permission is hereby granted, free of charge, to any person obtaining a copy
> of this software and associated documentation files (the "Software"), to deal
> in the Software without restriction, including without limitation the rights
> to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
> copies of the Software, and to permit persons to whom the Software is
> furnished to do so, subject to the following conditions:
>
> The above copyright notice and this permission notice shall be included in all
> copies or substantial portions of the Software.
>
> THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
> IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
> FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
> AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
> LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
> OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
> SOFTWARE.

## Other research

IW4L, AI Game Modding Guides, gamedb, RustPorts, universal-modder and the decompilation
catalog are research/workflow references; their code is not included. Their inspected
revisions and evaluations are in docs/RESEARCH.md. The Noesis importer was examined
as a format clue; its code was not copied/ported because no clear reuse license was
established. Scene/collision structures were derived and checked from actual bytes.
Private community messages and their attachments are not redistributed.

## Rust dependencies and binary notices

Bevy uses MIT OR Apache-2.0. flate2 uses MIT OR Apache-2.0. anyhow, serde, serde_json
and sha2 use MIT OR Apache-2.0. Transitive dependencies retain their own licenses;
Cargo.lock and Cargo metadata identify the exact resolved graph. Package.ps1 collects
distributed LICENSE/COPYING/NOTICE files, including nested font/asset notices, into
`dependency-licenses/` beside the binary and records SPDX metadata in inventory.json.
That directory includes the resolved Windows graph and build-only dependencies
conservatively; other-platform-only crates are not distributed in this binary.
Generated notices are outside this source repository and contain no game assets.
When a published crate omits notice files, the packager retrieves its upstream
notices at the exact Git revision recorded in .cargo_vcs_info.json. Source URLs
and hashes are retained in the inventory; missing notices fail packaging.
Two inspected releases contain no upstream license files: constgebra 0.1.4 declares
MIT OR Apache-2.0, and hexf-parse 0.2.1 declares CC0-1.0. For those exact revisions,
the package preserves Cargo.toml.orig's license declaration and supplies the standard
Apache-2.0 or CC0-1.0 text from SPDX license-list-data v3.28.0. The explicit exceptions
fail if version, recorded revision or declared license changes; they are not a
generic fallback for unknown licenses. Source URLs and hashes identify these texts.

Original game data, proprietary binaries and trademarks are not covered by the
project's MIT license. Players supply their own installation.
