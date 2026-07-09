#![allow(clippy::useless_conversion)]

mod vtk;

use pyo3::prelude::*;

/// A Mesh object holding points, cells, and field data.
#[pyclass]
#[derive(Clone, Debug)]
pub struct Mesh {
    #[pyo3(get, set)]
    pub(crate) points: Vec<[f64; 3]>,
    #[pyo3(get, set)]
    pub(crate) cells: Vec<CellBlock>,
    #[pyo3(get, set)]
    pub(crate) point_data: std::collections::HashMap<String, Vec<f64>>,
    #[pyo3(get, set)]
    pub(crate) cell_data: std::collections::HashMap<String, Vec<Vec<f64>>>,
    #[pyo3(get, set)]
    pub(crate) field_data: std::collections::HashMap<String, Vec<f64>>,
}

/// A block of cells of the same type.
#[pyclass]
#[derive(Clone, Debug)]
pub struct CellBlock {
    #[pyo3(get, set)]
    pub(crate) cell_type: String,
    #[pyo3(get, set)]
    pub(crate) data: Vec<Vec<usize>>,
}

#[pymethods]
impl Mesh {
    #[new]
    #[pyo3(signature = (points, cells, point_data=None, cell_data=None, field_data=None))]
    fn new(
        points: Vec<[f64; 3]>,
        cells: Vec<CellBlock>,
        point_data: Option<std::collections::HashMap<String, Vec<f64>>>,
        cell_data: Option<std::collections::HashMap<String, Vec<Vec<f64>>>>,
        field_data: Option<std::collections::HashMap<String, Vec<f64>>>,
    ) -> Self {
        Mesh {
            points,
            cells,
            point_data: point_data.unwrap_or_default(),
            cell_data: cell_data.unwrap_or_default(),
            field_data: field_data.unwrap_or_default(),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Mesh(points={}, cells={})",
            self.points.len(),
            self.cells.len()
        )
    }
}

#[pymethods]
impl CellBlock {
    #[new]
    fn new(cell_type: String, data: Vec<Vec<usize>>) -> Self {
        CellBlock { cell_type, data }
    }

    fn __repr__(&self) -> String {
        format!(
            "CellBlock(type={}, n_cells={})",
            self.cell_type,
            self.data.len()
        )
    }
}

/// Read a mesh from a file.
#[pyfunction]
fn read(filename: &str) -> PyResult<Mesh> {
    let path = std::path::Path::new(filename);
    vtk::read_vtk(path).map_err(pyo3::exceptions::PyRuntimeError::new_err)
}

/// Write a mesh to a file.
#[pyfunction]
fn write(mesh: &Mesh, filename: &str) -> PyResult<()> {
    let path = std::path::Path::new(filename);
    vtk::write_vtk(mesh, path).map_err(pyo3::exceptions::PyRuntimeError::new_err)
}

#[pymodule]
fn panmesh(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Mesh>()?;
    m.add_class::<CellBlock>()?;
    m.add_function(wrap_pyfunction!(read, m)?)?;
    m.add_function(wrap_pyfunction!(write, m)?)?;
    Ok(())
}
