# Format evidence — 2026-10-08

These findings apply to Steam build **19788008**, identified by docs/VALIDATION.md.
Byte structures were checked on the installed files, not accepted from a status flag.
Offsets below are file/record offsets, not addresses in a retail executable.

## RCF 2.1, little endian — confirmed

| Header offset | Encoding | Observed meaning |
| --- | --- | --- |
| 0x00 | 23 printable bytes plus nine zeros | `ATG CORE CEMENT LIBRARY` |
| 0x20 | four bytes | Major 2, minor 1, little endian 0, valid flag 1 |
| 0x24 / 0x28 | u32 / u32 | Entry table offset / byte length |
| 0x2c / 0x30 | u32 / u32 | Metadata offset / byte length |
| 0x34 / 0x38 | u32 / u32 | Reserved zero / number of entries |

Each entry is three u32 values: filename hash, payload offset, payload size.
Entries are not associated with metadata by row order. The metadata preamble is
`2048, 0`; each metadata record has type hash, alignment, reserved zero, name length,
a terminated name (length includes the null), then three zero bytes.

Hash behavior follows [Gibbed's StringHelpers.cs](https://github.com/gibbed/Gibbed.Prototype/blob/026da397342c4f8a43af575232dccf4b104df2e5/Gibbed.Prototype.FileFormats/StringHelpers.cs):
discard one leading backslash, then use wrapping u32 accumulation `h = h*31 + c`,
where each ASCII byte below 0x61 is increased by 0x20. This transforms digits and
punctuation too. Case-insensitive lookup normalizes slashes before matching stored
names; hashing itself preserves the retail backslash representation.

The implementation verifies every metadata/hash association, uniqueness, table and
payload boundaries, alignment and non-overlap. All 12 installed archives passed.
Index verification does not decode every payload. Individual reads are capped at
256 MiB; tables at 64 MiB. Extraction to filesystem paths is not implemented.

## RZ and Pure3D — confirmed within tested corpus

An RZ wrapper is 16 bytes: `RZ`, six zero bytes, little-endian decoded size in its
low u32, then zero high u32. A zlib stream follows. Current files fit in the low
32 bits. Nonzero high sizes and other wrapper variants are unsupported. The reader
checks declared size, decompression cap, checksum and complete stream consumption.
[Gibbed's compressor](https://github.com/gibbed/Gibbed.Prototype/blob/026da397342c4f8a43af575232dccf4b104df2e5/Gibbed.Prototype.Compress/Program.cs)
provided a comparison for the wrapper layout.

The decoded root is u32 ID `0xff443350`, data size 12, and total file size.
All chunks have `id, data_size, total_size` u32 headers. Children occupy the region
between data_size and total_size. Unknown chunks remain structurally traversable.
The parser validates parent bounds and caps depth at 128 and nodes at one million.
Big-endian and other compressed Pure3D variants are unsupported.

All 2,969 P3D entries in cells.rcf decoded and passed structural validation. This
does not confirm the semantic meaning of all 811,757 chunks.

## Selected cell's merged geometry — confirmed layout, limited scene support

Entry: `art\locations\yellow_zone\Cell_29.p3d.rz`, archive payload at 191907840,
14,427,822 stored bytes, 26,928,173 decoded bytes, 2,210 chunks.

| ID | Observed use |
| --- | --- |
| 0x00010040 | Named vertex-buffer owner |
| 0x00010041 | Named index-buffer owner |
| 0x00010043 | Attribute descriptors |
| 0x00010042 | Raw byte buffer with u32 length |
| 0x00025000 / 0x00025003 | Drawable/group references; full instance assembly pending |

Names use a one-byte serialized length; the span includes any null/alignment bytes.
After a descriptor's attribute name come seven u32 values: observed encoding,
component count, byte offset, stride, element count, and two additional fields.
Inspected float3 position attributes use encoding 0, component count 3. Index
attributes use encoding 3, component count 1 and stride 2; they decode as u16.
Other encodings and flags are not claimed understood. Raw lengths, strides, counts,
finite positions, triangle divisibility and index bounds are checked.

Buffer pairs are joined by exact names with `_vertices` / `_indices` suffixes,
not by encounter order. Only the 38 inspected `mergedDrawableRoot*` pairs are shown.
Their coordinates already occupy the same world region as the ground collision.
41 local buffers, their transforms, skinning and instance visibility remain omitted.
Material identities, UV decoding and texture binding remain unimplemented.

## Ground collision — confirmed geometry, unknown gameplay semantics

The reader restricts triangle nodes to ancestor physics chunk 0x07020000 named
`ground`. Triangle chunk 0x07021007 has version 0, float3 lower/upper bounds, u32
vertex count, float3 vertices, u32 face count, then faces of four u16 values.
The first three values reference vertices; the fourth is retained as an unknown tag.
Positions are finite and within bounds; every referenced index is checked.

The fourth field's role and the remaining payload are unknown. Spatial-tree data
is a hypothesis, not a recovered specification. The selected ground contains 60
triangle chunks, 20,075 faces, and 232,810 uninterpreted trailing bytes. Other physics
primitives and prop collision are not implemented. The 63 triangle chunks inspected
including local objects all had valid candidate indices, but local objects are not
installed as world collision without recovered transforms.

`raycast` performs a two-sided geometric triangle query and ignores tags. It does
not implement the original collision filter, swept player hull, step-up, wall slide,
grounding or parkour contacts. Rendering triangles and obtaining a ray hit are
evidence of geometry recovery, not proof of correct character collision.

## Metadata envelopes — checked; body schemas unknown

Chunk 0x07f00000 has three aligned u8-length strings (long, short and type name),
two u16 fields and one u32 field. The numeric field meanings remain unknown; the
u32 may identify the schema/type but is not treated as a proven hash contract.
Its direct child 0x07f00001 has a u32 length followed by exactly that many bytes.
Inspected bodies begin with `META`; the reader reports this signature without
assuming every body has it. These envelope layouts were informed by Gibbed's
MetaObjectDefinition/MetaObjectData and checked against actual character/startup
entries. Header and body lengths must consume their complete payloads.

The `meta` command searches names and bounded ASCII references in opaque bodies,
returning absolute decoded-P3D offsets and sizes. It does not export bodies or infer
the meaning of nearby numeric values. It also works in files without merged geometry.
Character collision-related locations and remaining schema work are in MOVEMENT.md.
