# RADJ Binary Adjacency Format (`.adj.bin`)

**Status:** Format v2 (current)
**Producer:** `redist fetch` (writes from TIGER), `redist-data::serialize_adjacency`
**Consumer:** `redist-data::deserialize_adjacency`, PyO3 binding `redist_py.deserialize_adjacency`

A compact binary format for adjacency graphs with vertex weights and per-edge weights. Designed for fast Rust-to-Rust I/O; the `redist` pipeline reads `.adj.bin` directly.

The format intentionally stays Rust-internal — external tools should consume `final_assignments.json` (which is the human-readable plan output) rather than the adjacency file.

## File Layout

All multi-byte integers are **little-endian**. Floats are IEEE 754 doubles, little-endian.

```
Offset  Size  Field          Type   Notes
─────────────────────────────────────────────────────────────────────
0       4     magic          [u8;4] b"RADJ"
4       4     version        u32    must equal 2
8       4     n_vertices     u32    number of tracts/units
12      4     n_edges        u32    number of edges with weight entries

16      ...   vertex_weights        repeated n_vertices times:
                              i64   per-vertex weight (population)

...     ...   adjacency             repeated n_vertices times:
                              u32   n_neighbors_i
                              u32 × n_neighbors_i  neighbor indices

...     4     n_weights      u32   must equal header n_edges
...     ...   edge_weights          repeated n_weights times:
                              u32   u  (vertex 1, must be < v)
                              u32   v  (vertex 2)
                              f64   weight
```

## Validation

A reader must fail with a typed error if any of the following hold:
1. Magic bytes are not `RADJ`
2. Version is not 2
3. Header is truncated before all sections can be read
4. Edge-weight section count does not equal the header `n_edges`
5. Any `(u, v)` pair has `u ≥ v` (canonical order required)

Reference implementation: `redist-data::serialize.rs::deserialize_adjacency`.

## Determinism

The serializer:
- Sorts edge weights by `(u, v)` before writing → byte-deterministic for the same graph
- Asserts canonical edge-key order (`u < v`) at write time
- Writes vertex weights and adjacency in adjacency-vertex-index order

Two serializations of equivalent graphs produce byte-identical files.

## Versioning

| Version | Date | Change |
|---|---|---|
| 1 | (pre-2026-04) | Initial format: header + adjacency + edge weights |
| 2 | 2026-04-25 | Added vertex_weights section between header and adjacency |

The format version is stored at offset 4. Readers must reject unknown versions with `UnsupportedVersion(v)`.

## File Layout (concrete example)

A 3-vertex graph with edges `(0,1)`, `(1,2)` and vertex weights `[100, 200, 150]`:

```
00000000  52 41 44 4a              # "RADJ"
00000004  02 00 00 00              # version = 2
00000008  03 00 00 00              # n_vertices = 3
0000000c  02 00 00 00              # n_edges = 2

# vertex_weights (3 × i64)
00000010  64 00 00 00 00 00 00 00  # 100
00000018  c8 00 00 00 00 00 00 00  # 200
00000020  96 00 00 00 00 00 00 00  # 150

# adjacency
00000028  01 00 00 00              # vertex 0: 1 neighbor
0000002c  01 00 00 00              #   neighbor 0 = 1
00000030  02 00 00 00              # vertex 1: 2 neighbors
00000034  00 00 00 00              #   neighbor 0 = 0
00000038  02 00 00 00              #   neighbor 1 = 2
0000003c  01 00 00 00              # vertex 2: 1 neighbor
00000040  01 00 00 00              #   neighbor 0 = 1

# edge_weights
00000044  02 00 00 00              # n_weights = 2
00000048  00 00 00 00              # u = 0
0000004c  01 00 00 00              # v = 1
00000050  ... 8 bytes f64 ...      # weight 0-1
00000058  01 00 00 00              # u = 1
0000005c  02 00 00 00              # v = 2
00000060  ... 8 bytes f64 ...      # weight 1-2
```

## Why this format

Considered alternatives:
- **JSON / TOML**: too slow at 50-state scale (compact graphs are millions of edges)
- **Pickle (.pkl)**: Python-only; incompatible with the Rust binary that's now the production tool
- **Protocol Buffers / MessagePack**: extra dep + framing overhead; the format is fixed enough to not need a schema language
- **GerryChain JSON**: tract-level metadata + geometry per node; way more than we need at this layer

The format is small, fast, and explicit. Not intended for cross-tool consumption.

## Migration from `.pkl`

Legacy Python pipelines wrote pickle (`.pkl`) adjacency files. These are no longer produced. All 50 states were one-shot converted to `.adj.bin` in 2026-04-25 (per `design/rust-port/migration-log.md` Phase 2 entry). The conversion script (`scripts/data/generate_adj_bin.py`) was deleted in Plan 02 Task 6 — it is no longer needed.

If you have an old `.pkl` you need to read, restore the script from `git log -- scripts/data/generate_adj_bin.py`.
