---
status: proposed
date: 2026-07-15
decision-makers: [tkoyama010]
consulted: []
informed: []
---

# Use hdf5 Crate for XDMF Heavy Data

## Context and Problem Statement

XDMF separates "light" data (topology, geometry metadata) in XML from "heavy" data (actual coordinate arrays, field values) stored in HDF5 files. panmesh needs to read and write HDF5 files to support XDMF. Which Rust crate should we use for HDF5 I/O? This is tracked in [#8](https://github.com/tkoyama010/panmesh/issues/8) and [#18](https://github.com/tkoyama010/panmesh/issues/18).

## Decision Drivers

* **Feature completeness**: Must support reading and writing multi-dimensional arrays with compression.
* **ndarray integration**: Prefer a crate with `ndarray` integration for ergonomic multi-dimensional array I/O.
* **Thread safety**: Must be safe to use in a multi-threaded context.
* **Build complexity**: Prefer to minimize build complexity, especially for PyO3/maturin wheel distribution.
* **Maturity**: Prefer a well-maintained crate with an active community.
* **License compatibility**: Must be compatible with panmesh's MIT license.

## Considered Options

* [`hdf5`](https://crates.io/crates/hdf5) 0.8.1 — Rust bindings to the HDF5 C library (aldanor/hdf5-rust)
* [`hdf5-pure`](https://crates.io/crates/hdf5-pure) — Pure Rust HDF5 reader/writer
* Manual HDF5 parsing — Implement HDF5 reading from scratch

## Decision Outcome

Chosen option: "hdf5 0.8.1 with `hdf5-src` feature", because it is the most feature-complete and mature HDF5 crate for Rust, with full `ndarray` integration and thread-safe bindings. The `hdf5-src` feature bundles the HDF5 C source, avoiding a system HDF5 installation requirement on CI. Evaluate `hdf5-pure` as a fallback if the native dependency proves too heavy for the maturin build pipeline.

### Consequences

* Good, because full-featured high-level API over the HDF5 C library provides robust read/write support.
* Good, because `ndarray` integration enables ergonomic multi-dimensional array I/O.
* Good, because thread-safe even with non-threadsafe libhdf5 builds (via reentrant mutexes).
* Good, because compression filters (gzip, blosc/zstd) are supported.
* Good, because the `hdf5-src` feature bundles HDF5 source, avoiding a system dependency on CI.
* Bad, because the `hdf5-sys` crate needs to locate libhdf5 at build time (via `HDF5_DIR` env var or pkg-config) when not using `hdf5-src`.
* Bad, because a native dependency complicates wheel building for PyO3/maturin distribution.
* Neutral, because HDF5 licensing is BSD-3-Clause, which is compatible with MIT.

### Confirmation

Compliance with this decision will be confirmed by:

1. `hdf5 = { version = "0.8", features = ["hdf5-src", "blosc"] }` is added to `Cargo.toml` when the XDMF story is implemented.
2. The build succeeds on CI without requiring a system HDF5 installation (via `hdf5-src`).
3. XDMF heavy data reading is implemented using the `hdf5` crate.
4. If the native dependency proves problematic for maturin, a follow-up ADR evaluates `hdf5-pure`.

## Pros and Cons of the Options

### hdf5 0.8.1 (with hdf5-src)

See [https://crates.io/crates/hdf5](https://crates.io/crates/hdf5)

* Good, because it is a full-featured high-level API over the HDF5 C library.
* Good, because it has `ndarray` integration for multi-dimensional array I/O.
* Good, because it is thread-safe via reentrant mutexes.
* Good, because it supports compression filters (gzip, blosc/zstd).
* Good, because it is well-maintained (345 GitHub stars, ~50,000 downloads/month).
* Good, because the `hdf5-src` feature bundles HDF5 source, avoiding a system dependency.
* Good, because it is MIT/Apache-2.0 licensed.
* Bad, because it requires a system HDF5 installation or the `hdf5-src` feature (which adds build time and a CMake dependency).
* Bad, because the native dependency complicates maturin wheel building.

### hdf5-pure

See [https://crates.io/crates/hdf5-pure](https://crates.io/crates/hdf5-pure)

* Good, because it is a pure Rust implementation — no C dependency, simplifying build and distribution.
* Good, because it supports contiguous, chunked, and compressed datasets.
* Good, because no system dependency is required.
* Bad, because it is less mature than the C-binding crate with a smaller community.
* Bad, because it may lack some advanced HDF5 features.
* Neutral, because it could be preferred if minimizing build complexity is the top priority.

### Manual HDF5 parsing

* Good, because it avoids any external dependency.
* Bad, because HDF5 is a complex binary format — implementing a parser from scratch would be a massive effort.
* Bad, because compression support would need to be implemented separately.
* Bad, because it would be extremely difficult to match the robustness of existing crates.

## More Information

* hdf5 crate: [https://crates.io/crates/hdf5](https://crates.io/crates/hdf5)
* Proposed `Cargo.toml` entry: `hdf5 = { version = "0.8", features = ["hdf5-src", "blosc"] }`
* Alternative if native dependency is problematic: `hdf5-pure = "0.15"`
* XDMF implementation approach: Use `quick-xml` (see [ADR-0002](0002-use-quick-xml-for-xml-based-formats.md)) for XML light data, then `hdf5` for heavy data arrays referenced by `DataItem` elements with `Format="HDF"`.
* Related issues: [#8](https://github.com/tkoyama010/panmesh/issues/8) (spike), [#18](https://github.com/tkoyama010/panmesh/issues/18) (evaluate hdf5)
