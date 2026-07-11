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

## License

[MIT](LICENSE)
