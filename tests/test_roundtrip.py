"""End-to-end tests for the panmesh read/write API.

These tests exercise the public Python API built from the Rust extension
(via maturin + PyO3). They perform a VTK round-trip: build a mesh, write it
to a file, read it back, and confirm the data survives the trip.
"""

import pytest

import panmesh


def _build_tetra_mesh():
    """A single-tetrahedron mesh with point and cell data."""
    return panmesh.Mesh(
        points=[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ],
        cells=[panmesh.CellBlock("tetra", [[0, 1, 2, 3]])],
        point_data={"pressure": [1.0, 2.0, 3.0, 4.0]},
        cell_data={"material_id": [[42.0]]},
    )


def test_roundtrip_points_and_cells(tmp_path):
    """Write a mesh to VTK, read it back, and compare points and cells."""
    mesh = _build_tetra_mesh()

    out = tmp_path / "roundtrip.vtk"
    panmesh.write(mesh, str(out))
    assert out.exists()

    result = panmesh.read(str(out))

    # Points match (float tolerance).
    assert len(result.points) == len(mesh.points)
    for original, roundtripped in zip(mesh.points, result.points):
        assert original == pytest.approx(roundtripped, abs=1e-10)

    # Cells match (exact connectivity).
    assert len(result.cells) == len(mesh.cells)
    for original, roundtripped in zip(mesh.cells, result.cells):
        assert roundtripped.cell_type == original.cell_type
        assert roundtripped.data == original.data


def test_roundtrip_point_and_cell_data(tmp_path):
    """Point data and cell data survive the round-trip."""
    mesh = _build_tetra_mesh()

    out = tmp_path / "roundtrip_data.vtk"
    panmesh.write(mesh, str(out))
    result = panmesh.read(str(out))

    assert result.point_data["pressure"] == pytest.approx([1.0, 2.0, 3.0, 4.0])

    material_id = result.cell_data["material_id"]
    assert len(material_id) == 1
    assert material_id[0] == pytest.approx([42.0])


def test_read_missing_file_raises(tmp_path):
    """Reading a nonexistent file raises rather than returning garbage."""
    missing = tmp_path / "does_not_exist.vtk"
    with pytest.raises(Exception):
        panmesh.read(str(missing))
