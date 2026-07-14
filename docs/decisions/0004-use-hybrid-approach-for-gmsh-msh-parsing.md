---
status: proposed
date: 2026-07-15
decision-makers: [tkoyama010]
consulted: []
informed: []
---

# Use Hybrid Approach for GMSH MSH Parsing

## Context and Problem Statement

panmesh needs to read and write GMSH mesh files in the `.msh` format. Two versions are in widespread use: MSH 2.2 (legacy, simple text-based, widely supported by meshio) and MSH 4.1 (current, block-based, supports ASCII and binary). How should we parse and write these formats? This is tracked in [#8](https://github.com/tkoyama010/panmesh/issues/8) and [#19](https://github.com/tkoyama010/panmesh/issues/19).

## Decision Drivers

* **Format coverage**: Must support both MSH 2.2 (legacy) and MSH 4.1 (modern) formats.
* **Read and write support**: Must support both reading and writing, since panmesh is a reader/writer library.
* **Minimal dependencies**: Prefer to avoid external dependencies when the format is simple enough to parse manually.
* **Binary support**: MSH 4.1 binary format should be supported for performance.
* **License compatibility**: Any crate used must be compatible with panmesh's MIT license.
* **Maintained dependencies**: If using a crate, it should be actively maintained or at least stable.

## Considered Options

* Hybrid approach — Manual parser for MSH 2.2; `mshio` for MSH 4.1 reading; manual writer for both versions
* [`mshio`](https://crates.io/crates/mshio) 0.8.0 — Use for both MSH 2.2 and 4.1
* Manual parsing — Implement parsers and writers for both MSH 2.2 and 4.1 from scratch

## Decision Outcome

Chosen option: "Hybrid approach", because MSH 2.2 is simple enough that a manual parser avoids an unnecessary dependency, while `mshio` provides solid MSH 4.1 read support (including binary) that would be costly to reimplement. No available crate provides write support, so writers must be manual regardless.

### Consequences

* Good, because no external dependency is needed for MSH 2.2 — the format is simple line-oriented text parseable in ~200 lines of Rust.
* Good, because `mshio` provides MSH 4.1 ASCII and binary read support, avoiding the cost of reimplementing the complex block-based format.
* Good, because manual writers for both versions give full control over output format.
* Bad, because two code paths (manual + crate) means more surface area to maintain.
* Bad, because `mshio` is not actively maintained (last commit ~2 years ago, 9 GitHub stars).
* Neutral, because if `mshio` proves insufficient, a manual parser for 4.1 can be implemented later — the format spec is well-documented.

### Confirmation

Compliance with this decision will be confirmed by:

1. `src/gmsh.rs` implements a manual reader and writer for MSH 2.2.
2. `mshio = "0.8"` is added to `Cargo.toml` for MSH 4.1 read support.
3. Manual writers for MSH 2.2 and 4.1 are implemented in `src/gmsh.rs`.
4. Round-trip tests pass for both MSH 2.2 and 4.1 formats.
5. If `mshio` proves insufficient, a follow-up ADR documents the switch to a manual 4.1 parser.

## Pros and Cons of the Options

### Hybrid approach — Manual for 2.2, mshio for 4.1 read, manual writers

* Good, because MSH 2.2 is simple enough to parse manually (~200 lines), avoiding an unnecessary dependency for the most common legacy format.
* Good, because `mshio` provides MSH 4.1 ASCII and binary read support, avoiding costly reimplementation.
* Good, because manual writers for both versions provide full control (no crate offers write support).
* Good, because the approach minimizes dependencies while covering both format versions.
* Bad, because two code paths (manual + crate) increase maintenance surface area.
* Bad, because `mshio` is not actively maintained — may need forking or replacement.

### mshio for both MSH 2.2 and 4.1

See [https://crates.io/crates/mshio](https://crates.io/crates/mshio)

* Good, because it is published on crates.io with CI.
* Good, because it supports both ASCII and binary MSH 4.1.
* Bad, because it only supports MSH 4.1, not legacy 2.2 — a separate solution is still needed for 2.2.
* Bad, because write support is not implemented (read-only).
* Bad, because it has a small community (9 GitHub stars) and is not actively maintained.
* Bad, because using it for only 4.1 read while needing manual code for 2.2 and all writing is essentially the hybrid approach.

### Manual parsing for both 2.2 and 4.1

* Good, because it avoids all external dependencies.
* Good, because full control over parsing and writing for both formats.
* Bad, because MSH 4.1 is a complex block-based format with binary support — implementing from scratch is significantly more effort than MSH 2.2.
* Bad, because binary MSH 4.1 parsing adds further complexity.
* Bad, because it duplicates what `mshio` already provides for 4.1 reading.

## More Information

* mshio crate: [https://crates.io/crates/mshio](https://crates.io/crates/mshio)
* MSH format specification: [https://gmsh.info/doc/texinfo/gmsh.html#File-formats](https://gmsh.info/doc/texinfo/gmsh.html#File-formats)
* Proposed `Cargo.toml` entry: `mshio = "0.8"` (for MSH 4.1 read support)
* Also considered: `gmsh-parser` (GitHub only, not on crates.io, ASCII only, not production-ready — ruled out)
* Related issues: [#8](https://github.com/tkoyama010/panmesh/issues/8) (spike), [#19](https://github.com/tkoyama010/panmesh/issues/19) (evaluate GMSH parsing)
