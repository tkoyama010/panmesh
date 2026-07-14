# panmesh

A Rust-based [meshio](https://github.com/nschloe/meshio) Python package.

`panmesh` aims to provide a high-performance drop-in replacement for
[`meshio`](https://github.com/nschloe/meshio) by implementing the core I/O
and mesh-manipulation routines in Rust and exposing them to Python via
[PyO3](https://github.com/PyO3/pyo3) and
[maturin](https://github.com/PyO3/maturin).

## Features (planned)

- Read and write a variety of mesh file formats (VTK, VTU, VTM, GMSH, XDMF, etc.)
- Mesh data structure compatible with `meshio.Mesh`
- High-performance cell and point data operations powered by Rust

## Installation

```bash
pip install panmesh
```

For development:

```bash
pip install maturin
maturin develop
```

## Usage

```python
import panmesh

mesh = panmesh.read("mesh.vtu")
print(mesh)
panmesh.write(mesh, "out.vtk")
```

## Testing

The Python end-to-end tests live under `tests/` and run against the built
extension. Build the extension into the environment first, then run pytest:

```bash
uv run maturin develop
uv run --group test pytest
```

## Architecture Decision Records

Architecturally significant decisions — such as crate choices for file-format
parsing — are recorded as [Markdown Architectural Decision Records
(MADR)](https://adr.github.io/madr/) in [`docs/decisions/`](docs/decisions/).

When opening an issue for an architecturally significant proposal, accompany it
with a new ADR record:

1. Copy `docs/decisions/adr-template.md` to `docs/decisions/NNNN-short-title-with-dashes.md`
   (use the next free number).
2. Fill in the YAML front matter (`status`, `date`, `decision-makers`,
   `consulted`, `informed`) and the MADR sections.
3. Link the ADR from the issue and reference the issue from the ADR.
4. Include the ADR in the PR that implements the decision.

A CI step lints ADR files with markdownlint using the MADR `.markdownlint.yml`
configuration on PRs that touch `docs/decisions/`.

## License

[MIT](LICENSE)
