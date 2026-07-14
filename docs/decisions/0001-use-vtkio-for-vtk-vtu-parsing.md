---
status: accepted
date: 2026-07-15
decision-makers: [tkoyama010]
consulted: []
informed: []
---

# Use vtkio for VTK/VTU Mesh File Parsing

## Context and Problem Statement

panmesh needs to read and write VTK mesh files in both legacy (`.vtk`) and XML (`.vtu`, `.vtp`, `.vti`, `.vts`, `.vtr`, `.vtm`) formats. Which Rust crate should we use for VTK/VTU parsing and writing? This is tracked in [#8](https://github.com/tkoyama010/panmesh/issues/8).

## Decision Drivers

* **Format coverage**: Must support both legacy VTK and XML VTK formats (serial and parallel).
* **Cell type support**: Must handle all standard cell types (vertex, line, triangle, quad, tetra, hex, wedge, pyramid, quadratic variants).
* **Proven integration**: Prefer a crate already integrated and tested in panmesh over introducing a new dependency.
* **Active maintenance**: The crate should be actively maintained or at a stable, reliable version.
* **License compatibility**: Must be compatible with panmesh's MIT license.

## Considered Options

* [`vtkio`](https://crates.io/crates/vtkio) 0.6.3 — Rust VTK/XML parser and writer
* Manual parsing — Implement VTK legacy and XML parsers from scratch
* `vtkio` 0.7.0-rc1 — Pre-release version with improvements

## Decision Outcome

Chosen option: "vtkio 0.6.3", because it is already integrated and proven in panmesh (`src/vtk.rs`) with full round-trip tests passing. No alternative crate matches its feature set for VTK/VTU parsing.

### Consequences

* Good, because both legacy and XML formats are supported through a single crate.
* Good, because built-in compression support (zlib, LZ4, LZMA) is available for XML formats.
* Good, because the rich `IOBuffer` data model covers all standard numeric types.
* Bad, because the `IOBuffer` type requires manual conversion to `f64` (handled by our `iobuffer_to_f64` helper).
* Bad, because voxel/pixel node ordering differs from meshio conventions (requires reordering, already handled in `src/vtk.rs`).
* Neutral, because 0.6.3 is from March 2021; a 0.7.0 release candidate is available but not yet stable.

### Confirmation

Compliance with this decision is confirmed by:

1. `vtkio = "0.6"` is present in `Cargo.toml`.
2. `src/vtk.rs` uses `vtkio` for both read and write operations.
3. Python end-to-end round-trip tests (`tests/`) pass with VTK files.

## Pros and Cons of the Options

### vtkio 0.6.3

See [https://crates.io/crates/vtkio](https://crates.io/crates/vtkio)

* Good, because it supports both legacy and XML VTK formats (serial and parallel).
* Good, because it handles all standard cell types including quadratic variants.
* Good, because it uses `nom` for legacy parsing and `quick-xml` + `serde` for XML — a robust parsing stack.
* Good, because it is MIT/Apache-2.0 licensed.
* Good, because it is already integrated and working in panmesh.
* Neutral, because it has ~11,000 downloads/month and 66 GitHub stars — modest but sufficient.
* Bad, because some edge cases with files produced by `meshio`/`pygmsh` have been reported (upstream issue #21), though largely resolved in recent versions.
* Bad, because 0.6.3 has not been updated since March 2021.

### Manual parsing

* Good, because it avoids any external dependency.
* Good, because full control over parsing edge cases.
* Bad, because the VTK XML format is complex with many cell types and data attributes — implementing from scratch would be a large effort.
* Bad, because compression support (zlib, LZ4, LZMA) would need to be implemented separately.
* Bad, because it would duplicate what `vtkio` already provides.

### vtkio 0.7.0-rc1

* Good, because it includes improved XML parsing and bug fixes.
* Bad, because it is a release candidate, not yet stable — risk of breaking changes before the final release.
* Bad, because adopting a pre-release could introduce instability.

## More Information

* vtkio crate: [https://crates.io/crates/vtkio](https://crates.io/crates/vtkio)
* Plan to upgrade to `vtkio = "0.7"` once it reaches a stable release.
* Related issues: [#8](https://github.com/tkoyama010/panmesh/issues/8) (spike), [#9](https://github.com/tkoyama010/panmesh/issues/9) (read VTK), [#10](https://github.com/tkoyama010/panmesh/issues/10) (write VTK)
