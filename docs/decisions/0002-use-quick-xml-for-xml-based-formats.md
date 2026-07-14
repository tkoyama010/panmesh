---
status: proposed
date: 2026-07-15
decision-makers: [tkoyama010]
consulted: []
informed: []
---

# Use quick-xml for XML-Based Mesh Formats

## Context and Problem Statement

panmesh will need to parse XML-based mesh formats — primarily XDMF (`.xdmf`) files whose light data (topology, geometry metadata) is XML, and VTM (multiblock) XML wrapper files. Which Rust crate should we use for XML parsing? This is tracked in [#8](https://github.com/tkoyama010/panmesh/issues/8) and [#17](https://github.com/tkoyama010/panmesh/issues/17).

## Decision Drivers

* **Performance**: XML parsing should be fast, especially for large XDMF files with many elements.
* **Memory footprint**: Large files should be parseable with low memory usage (streaming preferred).
* **Minimal new dependencies**: Prefer crates already in the dependency tree to avoid bloating the build.
* **Serde integration**: Declarative deserialization support reduces boilerplate for structured XML.
* **Writer support**: The crate should also support generating XML output for writing XDMF/VTM files.
* **License compatibility**: Must be compatible with panmesh's MIT license.

## Considered Options

* [`quick-xml`](https://crates.io/crates/quick-xml) 0.41.0 — Event-based XML reader/writer with optional serde support
* [`xml-rs`](https://crates.io/crates/xml-rs) — Another XML parser for Rust
* [`roxmltree`](https://crates.io/crates/roxmltree) — DOM-based XML parser

## Decision Outcome

Chosen option: "quick-xml 0.41.0", because it is the de facto standard XML parser for Rust, is already a transitive dependency of `vtkio`, and offers both event-based streaming and optional serde integration. Adding it as a direct dependency introduces no new build dependencies.

### Consequences

* Good, because no new transitive dependencies are added (already pulled in by `vtkio`).
* Good, because event-based streaming provides a low memory footprint for large files.
* Good, because optional serde integration enables declarative deserialization of XML structures.
* Good, because writer support is available for generating XML output.
* Bad, because the low-level event API requires manual state management for complex XML schemas like XDMF.
* Neutral, because the serde integration has some quirks with mixed content models, though this is unlikely to affect XDMF parsing.

### Confirmation

Compliance with this decision will be confirmed by:

1. `quick-xml` is added as a direct dependency in `Cargo.toml` with the `serialize` feature.
2. The pinned version is compatible with the `vtkio` transitive dependency (verified via `Cargo.lock` — only one version resolved).
3. XDMF XML parsing is implemented using `quick-xml` when the XDMF story is started.

## Pros and Cons of the Options

### quick-xml 0.41.0

See [https://crates.io/crates/quick-xml](https://crates.io/crates/quick-xml)

* Good, because it is approximately 50x faster than `xml-rs` in benchmarks.
* Good, because event-based streaming keeps memory footprint low for large files.
* Good, because optional serde integration supports declarative deserialization.
* Good, because it includes a writer for generating XML output.
* Good, because it is mature, actively maintained, and widely used (1,000+ GitHub stars).
* Good, because it is already a transitive dependency of `vtkio` — no new build dependencies.
* Neutral, because the serde integration has some quirks with mixed content models.
* Bad, because the low-level event API requires manual state management for complex schemas.

### xml-rs

See [https://crates.io/crates/xml-rs](https://crates.io/crates/xml-rs)

* Good, because it is a simple, easy-to-use XML parser.
* Bad, because it is significantly slower than `quick-xml` (~50x in benchmarks).
* Bad, because it is not already in the dependency tree — adds a new dependency.
* Bad, because it lacks serde integration.
* Bad, because it is less actively maintained than `quick-xml`.

### roxmltree

See [https://crates.io/crates/roxmltree](https://crates.io/crates/roxmltree)

* Good, because DOM-based parsing provides easy random access to the XML tree.
* Bad, because DOM-based parsing loads the entire document into memory — unsuitable for large XDMF files.
* Bad, because it is read-only (no writer support).
* Bad, because it is not already in the dependency tree.

## More Information

* quick-xml crate: [https://crates.io/crates/quick-xml](https://crates.io/crates/quick-xml)
* Proposed `Cargo.toml` entry: `quick-xml = { version = "0.36", features = ["serialize"] }`
* Pin to a version compatible with the `vtkio` transitive dependency to avoid pulling two versions. Check `Cargo.lock` for the exact version `vtkio` 0.6.3 resolves to.
* Related issues: [#8](https://github.com/tkoyama010/panmesh/issues/8) (spike), [#17](https://github.com/tkoyama010/panmesh/issues/17) (evaluate quick-xml)
