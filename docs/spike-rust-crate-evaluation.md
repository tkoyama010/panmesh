# Spike: Evaluate Rust Crates for Mesh File Parsing

**Issue:** [#8](https://github.com/tkoyama010/panmesh/issues/8)
**Date:** 2026-07-15
**Status:** Complete

## Overview

This spike evaluates available Rust crates for parsing the mesh file formats
that panmesh aims to support: VTK, VTU, GMSH, and XDMF. The goal is to inform
the implementation approach for upcoming reader/writer stories.

---

## 1. VTK / VTU — `vtkio`

| Attribute | Value |
|---|---|
| Crate | [`vtkio`](https://crates.io/crates/vtkio) |
| Current version | 0.6.3 (stable); 0.7.0-rc1 (pre-release) |
| License | MIT / Apache-2.0 |
| Downloads | ~11,000/month |
| GitHub stars | 66 |
| Used by | 13 crates |
| **Already in panmesh** | **Yes** (Cargo.toml, `vtkio = "0.6"`) |

### Evaluation

`vtkio` is a feature-complete parser and writer for both Legacy VTK (`.vtk`)
and XML VTK formats (`.vtu`, `.vtp`, `.vti`, `.vts`, `.vtr`, `.vtm`). It is
already integrated and proven in panmesh — `src/vtk.rs` uses it for read and
write operations with full round-trip tests passing.

**Strengths:**
- Supports both Legacy and XML formats (serial and parallel)
- Handles all standard cell types (vertex, line, triangle, quad, tetra, hex, wedge, pyramid, quadratic variants)
- Built-in compression support (zlib, LZ4, LZMA) for XML formats
- Rich data model: `IOBuffer` supports f32/f64/i8/i16/i32/i64/u8/u16/u32/u64
- Active maintenance; 0.7.0 release candidate available
- Uses `nom` for legacy parsing, `quick-xml` + `serde` for XML — robust stack

**Weaknesses:**
- Some edge cases with files produced by `meshio`/`pygmsh` reported (upstream issue #21), though these appear to be largely resolved in recent versions
- The `IOBuffer` type requires manual conversion to `f64` (our `iobuffer_to_f64` helper)
- Voxel/Pixel node ordering differs from meshio conventions (requires reordering, already handled in `src/vtk.rs`)
- 0.7.0 is still a release candidate; 0.6.3 is from March 2021

**Recommendation:** **Keep using `vtkio`.** It is already integrated and
working. Plan to upgrade to 0.7.0 once it reaches stable release for improved
XML parsing and bug fixes. No alternative crate matches its feature set.

### Proposed `Cargo.toml` entry

```toml
vtkio = "0.6"
# Future: upgrade to vtkio = "0.7" when stable
```

---

## 2. XML-based Formats — `quick-xml`

| Attribute | Value |
|---|---|
| Crate | [`quick-xml`](https://crates.io/crates/quick-xml) |
| Current version | 0.41.0 |
| License | MIT |
| Downloads | Very high (widely used) |
| GitHub stars | 1,000+ |
| **Already a dependency** | **Yes** (transitive via `vtkio`) |

### Evaluation

`quick-xml` is the de facto standard XML reader/writer for Rust. It is a
high-performance, event-based (pull) parser with optional `serde`
serialization support. It is already in the dependency tree as a transitive
dependency of `vtkio`.

**Strengths:**
- ~50x faster than `xml-rs` in benchmarks
- Event-based streaming: low memory footprint for large files
- Optional `serde` integration for declarative deserialization
- Writer support for generating XML output
- Mature, actively maintained, frequent releases
- Already in dependency tree (no new transitive deps)

**Use cases in panmesh:**
- XDMF `.xdmf` files are XML with light data — `quick-xml` is ideal for parsing the XML structure
- VTM (multiblock) XML wrapper format
- Any future XML-based mesh format

**Weaknesses:**
- Low-level event API requires manual state management for complex XML schemas
- The `serde` integration has some quirks with mixed content models

**Recommendation:** **Adopt `quick-xml` directly** for XDMF XML parsing and
any other XML-based formats. Since it is already a transitive dependency of
`vtkio`, adding it as a direct dependency adds no new build dependencies.

### Proposed `Cargo.toml` entry

```toml
quick-xml = { version = "0.36", features = ["serialize"] }
```

> **Note:** Pin to a version compatible with the `vtkio` transitive dependency
> to avoid pulling two versions. Check `Cargo.lock` for the exact version
> `vtkio` 0.6.3 resolves to.

---

## 3. XDMF Heavy Data — `hdf5`

| Attribute | Value |
|---|---|
| Crate | [`hdf5`](https://crates.io/crates/hdf5) (aldanor/hdf5-rust) |
| Current version | 0.8.1 |
| License | MIT / Apache-2.0 |
| Downloads | ~50,000/month |
| GitHub stars | 345 |
| System dependency | Requires libhdf5 (1.8.4+) installed on the system |

### Evaluation

XDMF separates "light" data (topology, geometry metadata) in XML from "heavy"
data (actual coordinate arrays, field values) stored in HDF5 files. The `hdf5`
crate provides thread-safe Rust bindings to the HDF5 C library.

**Strengths:**
- Full-featured high-level API over the HDF5 C library
- `ndarray` integration for multi-dimensional array I/O
- Thread-safe even with non-threadsafe libhdf5 builds (via reentrant mutexes)
- Supports compression filters (gzip, blosc/zstd)
- Well-maintained, 345 stars, mature project
- Can be built with bundled HDF5 source via `hdf5-src` feature (avoids system dependency on CI)

**Weaknesses:**
- Requires a system HDF5 installation or the `hdf5-src` feature (which adds build time and a CMake dependency)
- Adds a native dependency — complicates wheel building for PyO3/maturin distribution
- HDF5 licensing is BSD-3-Clause (compatible with MIT)
- The `hdf5-sys` crate needs to locate libhdf5 at build time (via `HDF5_DIR` env var or pkg-config)

**Alternative: `hdf5-pure`** (pure Rust, no C dependency):
- Newer crate, pure Rust implementation of HDF5 reader/writer
- No system dependency required — simplifies build and distribution
- Supports contiguous, chunked, and compressed datasets
- Less mature than the C-binding crate; smaller community
- Could be preferred if minimizing build complexity is a priority

**Recommendation:** **Adopt the `hdf5` crate with the `hdf5-src` feature** for
development and CI to avoid requiring a system HDF5 installation. Evaluate
`hdf5-pure` as a fallback if the native dependency proves too heavy for the
maturin build pipeline. XDMF support is a lower priority than GMSH, so this
dependency can be added when the XDMF story is implemented.

### Proposed `Cargo.toml` entry (when XDMF story is implemented)

```toml
hdf5 = { version = "0.8", features = ["hdf5-src", "blosc"] }
# Or, if native dependency is problematic:
# hdf5-pure = "0.15"
```

### XDMF implementation approach

1. Use `quick-xml` to parse the `.xdmf` XML file (light data: topology, geometry, attribute metadata)
2. For `DataItem` elements with `Format="HDF"`, extract the HDF5 file path and dataset path
3. Use `hdf5` to read the heavy data arrays from the referenced `.h5` file
4. For `DataItem` elements with `Format="XML"`, parse inline data directly from the XML
5. For `DataItem` elements with `Format="Binary"`, read raw binary data from a referenced file
6. Map XDMF topology types to panmesh cell types (similar to VTK mapping in `src/vtk.rs`)

---

## 4. GMSH / MSH Format

GMSH uses the `.msh` file format. Two versions are in widespread use:
- **MSH 2.2** (legacy): Simple text-based format, widely supported by meshio
- **MSH 4.1** (current): Block-based format supporting both ASCII and binary

### Option A: `mshio` crate

| Attribute | Value |
|---|---|
| Crate | [`mshio`](https://crates.io/crates/mshio) |
| Current version | 0.8.0 |
| License | MIT |
| GitHub stars | 9 |
| Format support | MSH 4.1 only (ASCII + binary) |

**Strengths:**
- Published on crates.io with CI
- Supports both ASCII and binary MSH 4.1
- Well-structured output mirroring the MSH file format
- MIT licensed

**Weaknesses:**
- Only supports MSH 4.1, not legacy 2.2
- Write support not implemented (read-only)
- Small community (9 stars)
- Not actively maintained (last commit ~2 years ago)

### Option B: `gmsh-parser` crate

| Attribute | Value |
|---|---|
| Crate | `gmsh-parser` (GitHub only, not on crates.io) |
| License | Not specified |
| Format support | MSH 4.1 ASCII only |

**Strengths:**
- Comprehensive section support (all MSH 4.1 sections)
- Detailed error reporting via `miette`

**Weaknesses:**
- Not on crates.io (git dependency only)
- ASCII only, no binary support
- Personal project, explicitly "not production-ready"
- No write support

### Option C: Manual parsing

The MSH 2.2 format is a simple, line-oriented text format:
- `$MeshFormat` / `$EndMeshFormat`
- `$Nodes` / `$EndNodes`
- `$Elements` / `$EndElements`
- `$PhysicalNames` / `$EndPhysicalNames`

A manual parser for MSH 2.2 can be written in ~200 lines of Rust using
standard library string parsing. This avoids any external dependency for the
most common legacy format.

### Recommendation

**Use a hybrid approach:**

1. **MSH 2.2 (legacy):** Implement a manual parser. The format is simple enough
   that a dependency would be overkill. This covers the most common GMSH files
   in the wild (meshio's default GMSH output for years was 2.2).
2. **MSH 4.1 (modern):** Use `mshio` for reading. If `mshio` proves
   insufficient or unmaintained, fork or implement a manual parser for 4.1 as
   well (the format spec is well-documented).
3. **Writing:** Implement manual writers for both 2.2 and 4.1, since no
   available crate provides write support.

This approach minimizes dependencies while covering both MSH format versions.

---

## Summary of Recommendations

| Format | Crate | Action | Priority |
|---|---|---|---|
| VTK / VTU | `vtkio` 0.6 | Keep (already integrated) | Done |
| XML (XDMF, VTM) | `quick-xml` | Add as direct dependency | When XDMF story starts |
| XDMF heavy data | `hdf5` (with `hdf5-src`) | Add when XDMF story starts | Medium |
| GMSH MSH 2.2 | Manual parser | Implement in `src/gmsh.rs` | High |
| GMSH MSH 4.1 | `mshio` for read, manual writer | Add `mshio` dependency | Medium |

## Proposed Dependency Additions

```toml
[dependencies]
pyo3 = "0.22"
vtkio = "0.6"
quick-xml = { version = "0.36", features = ["serialize"] }  # for XDMF XML
mshio = "0.8"                                               # for GMSH MSH 4.1 read
# hdf5 = { version = "0.8", features = ["hdf5-src"] }       # for XDMF HDF5 (later)
```

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `vtkio` 0.7 breaking changes on upgrade | Medium | Low | Thorough round-trip tests already in place |
| `hdf5` native dependency complicates maturin build | Medium | High | Use `hdf5-src` feature or `hdf5-pure` alternative |
| `mshio` unmaintained / missing features | Medium | Medium | Keep manual parser as fallback; format spec is documented |
| `quick-xml` version conflict with `vtkio` transitive dep | Low | Low | Pin compatible version; check Cargo.lock |

## Next Steps

1. Implement GMSH MSH 2.2 reader/writer (manual parser) — highest value, no new deps
2. Add `mshio` for MSH 4.1 read support
3. Add `quick-xml` as direct dependency and implement XDMF XML parsing
4. Add `hdf5` (with `hdf5-src`) and implement XDMF HDF5 heavy data reading
5. Plan `vtkio` 0.7 upgrade after stable release
