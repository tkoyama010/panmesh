use std::collections::HashMap;
use std::path::Path;

use vtkio::model::{
    Attribute, CellType, DataSet, ElementType, IOBuffer, Piece, PolyDataPiece,
    UnstructuredGridPiece, VertexNumbers, Vtk,
};

use crate::{CellBlock, Mesh};

fn cell_type_to_string(ct: &CellType) -> &str {
    match ct {
        CellType::Vertex => "vertex",
        CellType::PolyVertex => "poly_vertex",
        CellType::Line => "line",
        CellType::PolyLine => "poly_line",
        CellType::Triangle => "triangle",
        CellType::TriangleStrip => "triangle_strip",
        CellType::Polygon => "polygon",
        CellType::Pixel => "quad",
        CellType::Quad => "quad",
        CellType::Tetra => "tetra",
        CellType::Voxel => "hexahedron",
        CellType::Hexahedron => "hexahedron",
        CellType::Wedge => "wedge",
        CellType::Pyramid => "pyramid",
        CellType::QuadraticEdge => "quadratic_edge",
        CellType::QuadraticTriangle => "quadratic_triangle",
        CellType::QuadraticQuad => "quadratic_quad",
        CellType::QuadraticTetra => "quadratic_tetra",
        CellType::QuadraticHexahedron => "quadratic_hexahedron",
        CellType::QuadraticWedge => "quadratic_wedge",
        CellType::QuadraticPyramid => "quadratic_pyramid",
        _ => "unknown",
    }
}

fn iobuffer_to_f64(buf: &IOBuffer) -> Vec<f64> {
    match buf {
        IOBuffer::F64(v) => v.clone(),
        IOBuffer::F32(v) => v.iter().map(|&x| x as f64).collect(),
        IOBuffer::I64(v) => v.iter().map(|&x| x as f64).collect(),
        IOBuffer::I32(v) => v.iter().map(|&x| x as f64).collect(),
        IOBuffer::U64(v) => v.iter().map(|&x| x as f64).collect(),
        IOBuffer::U32(v) => v.iter().map(|&x| x as f64).collect(),
        IOBuffer::U8(v) => v.iter().map(|&x| x as f64).collect(),
        IOBuffer::I8(v) => v.iter().map(|&x| x as f64).collect(),
        IOBuffer::U16(v) => v.iter().map(|&x| x as f64).collect(),
        IOBuffer::I16(v) => v.iter().map(|&x| x as f64).collect(),
        IOBuffer::Bit(_) => Vec::new(),
    }
}

fn extract_points(buf: &IOBuffer) -> Vec<[f64; 3]> {
    let flat = iobuffer_to_f64(buf);
    flat.chunks(3).map(|c| [c[0], c[1], c[2]]).collect()
}

fn vertex_numbers_to_cells(vn: &VertexNumbers) -> Vec<Vec<usize>> {
    match vn {
        VertexNumbers::Legacy { vertices, .. } => {
            let mut cells = Vec::new();
            let mut i = 0;
            while i < vertices.len() {
                let n = vertices[i] as usize;
                let cell: Vec<usize> = vertices[i + 1..i + 1 + n]
                    .iter()
                    .map(|&v| v as usize)
                    .collect();
                cells.push(cell);
                i += 1 + n;
            }
            cells
        }
        VertexNumbers::XML {
            connectivity,
            offsets,
        } => {
            let mut cells = Vec::new();
            let mut prev = 0usize;
            for &offset in offsets {
                let end = offset as usize;
                let cell: Vec<usize> = connectivity[prev..end]
                    .iter()
                    .map(|&v| v as usize)
                    .collect();
                cells.push(cell);
                prev = end;
            }
            cells
        }
    }
}

fn expand_triangle_strip(verts: &[usize]) -> Vec<Vec<usize>> {
    if verts.len() < 3 {
        return Vec::new();
    }
    let mut tris = Vec::with_capacity(verts.len() - 2);
    for i in 0..verts.len() - 2 {
        if i % 2 == 0 {
            tris.push(vec![verts[i], verts[i + 1], verts[i + 2]]);
        } else {
            tris.push(vec![verts[i + 1], verts[i], verts[i + 2]]);
        }
    }
    tris
}

fn num_comp_from_element_type(et: &ElementType) -> usize {
    et.num_comp() as usize
}

fn extract_point_attributes(attrs: &[Attribute]) -> HashMap<String, Vec<f64>> {
    let mut map = HashMap::new();
    for attr in attrs {
        match attr {
            Attribute::DataArray(da) => {
                let name = da.name.clone();
                let data = iobuffer_to_f64(&da.data);
                map.insert(name, data);
            }
            Attribute::Field { name, data_array } => {
                for fa in data_array {
                    let fa_name = format!("{}.{}", name, fa.name);
                    let data = iobuffer_to_f64(&fa.data);
                    map.insert(fa_name, data);
                }
            }
        }
    }
    map
}

fn extract_cell_attributes(
    attrs: &[Attribute],
    cell_indices_per_block: &[Vec<usize>],
) -> HashMap<String, Vec<Vec<f64>>> {
    let mut map = HashMap::new();
    for attr in attrs {
        if let Attribute::DataArray(da) = attr {
            let name = da.name.clone();
            let data = iobuffer_to_f64(&da.data);
            let num_comp = num_comp_from_element_type(&da.elem);

            let per_block: Vec<Vec<f64>> = cell_indices_per_block
                .iter()
                .map(|indices| {
                    let mut block_data = Vec::with_capacity(indices.len() * num_comp);
                    for &idx in indices {
                        let start = idx * num_comp;
                        let end = start + num_comp;
                        if end <= data.len() {
                            block_data.extend_from_slice(&data[start..end]);
                        }
                    }
                    block_data
                })
                .collect();

            map.insert(name, per_block);
        }
    }
    map
}

fn voxel_to_hex_reorder(cell: &[usize]) -> Vec<usize> {
    if cell.len() == 8 {
        vec![
            cell[0], cell[1], cell[3], cell[2], cell[4], cell[5], cell[7], cell[6],
        ]
    } else {
        cell.to_vec()
    }
}

fn pixel_to_quad_reorder(cell: &[usize]) -> Vec<usize> {
    if cell.len() == 4 {
        vec![cell[0], cell[1], cell[3], cell[2]]
    } else {
        cell.to_vec()
    }
}

fn process_unstructured_grid(piece: UnstructuredGridPiece) -> Result<Mesh, String> {
    let points = extract_points(&piece.points);

    let raw_cells = vertex_numbers_to_cells(&piece.cells.cell_verts);
    let cell_types = &piece.cells.types;

    if raw_cells.len() != cell_types.len() {
        return Err(format!(
            "Cell count mismatch: {} cells vs {} cell types",
            raw_cells.len(),
            cell_types.len()
        ));
    }

    let mut groups: Vec<(String, Vec<usize>, Vec<Vec<usize>>)> = Vec::new();
    for (i, (cell, ct)) in raw_cells.iter().zip(cell_types.iter()).enumerate() {
        let type_str = cell_type_to_string(ct).to_string();
        let reordered = match ct {
            CellType::Voxel => voxel_to_hex_reorder(cell),
            CellType::Pixel => pixel_to_quad_reorder(cell),
            _ => cell.clone(),
        };

        if let Some(pos) = groups
            .iter()
            .position(|g: &(String, Vec<usize>, Vec<Vec<usize>>)| g.0 == type_str)
        {
            groups[pos].1.push(i);
            groups[pos].2.push(reordered);
        } else {
            groups.push((type_str, vec![i], vec![reordered]));
        }
    }

    let cells: Vec<CellBlock> = groups
        .iter()
        .map(|(t, _, data)| CellBlock {
            cell_type: t.clone(),
            data: data.clone(),
        })
        .collect();

    let cell_indices_per_block: Vec<Vec<usize>> =
        groups.iter().map(|(_, idxs, _)| idxs.clone()).collect();

    let point_data = extract_point_attributes(&piece.data.point);
    let cell_data = extract_cell_attributes(&piece.data.cell, &cell_indices_per_block);

    let field_data = extract_field_attributes(&piece.data.point, &piece.data.cell);

    Ok(Mesh {
        points,
        cells,
        point_data,
        cell_data,
        field_data,
    })
}

fn extract_field_attributes(
    point_attrs: &[Attribute],
    cell_attrs: &[Attribute],
) -> HashMap<String, Vec<f64>> {
    let mut map = HashMap::new();
    for attr in point_attrs.iter().chain(cell_attrs.iter()) {
        if let Attribute::Field { data_array, .. } = attr {
            for fa in data_array {
                let data = iobuffer_to_f64(&fa.data);
                map.insert(fa.name.clone(), data);
            }
        }
    }
    map
}

fn polydata_topology_to_cells(vn: &Option<VertexNumbers>) -> Vec<Vec<usize>> {
    vn.as_ref().map(vertex_numbers_to_cells).unwrap_or_default()
}

fn process_polydata(piece: PolyDataPiece) -> Result<Mesh, String> {
    let points = extract_points(&piece.points);

    let mut groups: Vec<(String, Vec<usize>, Vec<Vec<usize>>)> = Vec::new();
    let mut cell_index = 0usize;

    let verts = polydata_topology_to_cells(&piece.verts);
    for cell in &verts {
        let type_str = "vertex";
        if let Some(pos) = groups.iter().position(|g| g.0 == type_str) {
            groups[pos].1.push(cell_index);
            groups[pos].2.push(cell.clone());
        } else {
            groups.push((type_str.to_string(), vec![cell_index], vec![cell.clone()]));
        }
        cell_index += 1;
    }

    let lines = polydata_topology_to_cells(&piece.lines);
    for cell in &lines {
        let type_str = "line";
        if let Some(pos) = groups.iter().position(|g| g.0 == type_str) {
            groups[pos].1.push(cell_index);
            groups[pos].2.push(cell.clone());
        } else {
            groups.push((type_str.to_string(), vec![cell_index], vec![cell.clone()]));
        }
        cell_index += 1;
    }

    let polys = polydata_topology_to_cells(&piece.polys);
    for cell in &polys {
        let type_str = match cell.len() {
            3 => "triangle",
            4 => "quad",
            _ => "polygon",
        };
        if let Some(pos) = groups.iter().position(|g| g.0 == type_str) {
            groups[pos].1.push(cell_index);
            groups[pos].2.push(cell.clone());
        } else {
            groups.push((type_str.to_string(), vec![cell_index], vec![cell.clone()]));
        }
        cell_index += 1;
    }

    let strips_raw = polydata_topology_to_cells(&piece.strips);
    for strip in &strips_raw {
        let tris = expand_triangle_strip(strip);
        for tri in &tris {
            let type_str = "triangle";
            if let Some(pos) = groups.iter().position(|g| g.0 == type_str) {
                groups[pos].1.push(cell_index);
                groups[pos].2.push(tri.clone());
            } else {
                groups.push((type_str.to_string(), vec![cell_index], vec![tri.clone()]));
            }
            cell_index += 1;
        }
    }

    let cells: Vec<CellBlock> = groups
        .iter()
        .map(|(t, _, data)| CellBlock {
            cell_type: t.clone(),
            data: data.clone(),
        })
        .collect();

    let cell_indices_per_block: Vec<Vec<usize>> =
        groups.iter().map(|(_, idxs, _)| idxs.clone()).collect();

    let point_data = extract_point_attributes(&piece.data.point);
    let cell_data = extract_cell_attributes(&piece.data.cell, &cell_indices_per_block);

    let field_data = extract_field_attributes(&piece.data.point, &piece.data.cell);

    Ok(Mesh {
        points,
        cells,
        point_data,
        cell_data,
        field_data,
    })
}

pub fn read_vtk(path: &Path) -> Result<Mesh, String> {
    let vtk = Vtk::import(path).map_err(|e| format!("Failed to import VTK file: {:?}", e))?;

    match vtk.data {
        DataSet::UnstructuredGrid { pieces, .. } => {
            let piece = pieces
                .into_iter()
                .next()
                .ok_or("No pieces in UnstructuredGrid")?;
            let piece_data = match piece {
                Piece::Inline(data) => *data,
                Piece::Loaded(data) => UnstructuredGridPiece::try_from(*data)
                    .map_err(|e| format!("Failed to convert loaded piece: {:?}", e))?,
                Piece::Source(..) => {
                    return Err(
                        "Referenced piece not loaded. Use load_all_pieces() first.".to_string()
                    )
                }
            };
            process_unstructured_grid(piece_data)
        }
        DataSet::PolyData { pieces, .. } => {
            let piece = pieces.into_iter().next().ok_or("No pieces in PolyData")?;
            let piece_data = match piece {
                Piece::Inline(data) => *data,
                Piece::Loaded(data) => PolyDataPiece::try_from(*data)
                    .map_err(|e| format!("Failed to convert loaded piece: {:?}", e))?,
                Piece::Source(..) => {
                    return Err(
                        "Referenced piece not loaded. Use load_all_pieces() first.".to_string()
                    )
                }
            };
            process_polydata(piece_data)
        }
        _ => Err(format!("Unsupported dataset type: {:?}", vtk.data)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const TET_VTK: &str = "\
# vtk DataFile Version 3.0
Tetrahedron mesh
ASCII
DATASET UNSTRUCTURED_GRID
POINTS 4 float
0.0 0.0 0.0
1.0 0.0 0.0
0.0 1.0 0.0
0.0 0.0 1.0

CELLS 1 5
4 0 1 2 3

CELL_TYPES 1
10

POINT_DATA 4
SCALARS pressure float 1
LOOKUP_TABLE default
1.0 2.0 3.0 4.0

CELL_DATA 1
SCALARS material_id int 1
LOOKUP_TABLE default
42
";

    const MIXED_VTK: &str = "\
# vtk DataFile Version 3.0
Mixed mesh
ASCII
DATASET UNSTRUCTURED_GRID
POINTS 6 float
0.0 0.0 0.0
1.0 0.0 0.0
1.0 1.0 0.0
0.0 1.0 0.0
0.0 0.0 1.0
1.0 0.0 1.0

CELLS 3 13
3 0 1 2
3 0 2 3
4 0 1 4 5

CELL_TYPES 3
5
5
10

POINT_DATA 6
SCALARS temperature float 1
LOOKUP_TABLE default
10.0 20.0 30.0 40.0 50.0 60.0

CELL_DATA 3
SCALARS region int 1
LOOKUP_TABLE default
1 2 3
";

    const TRI_POLYDATA_VTK: &str = "\
# vtk DataFile Version 2.0
Triangle example
ASCII
DATASET POLYDATA
POINTS 3 float
0.0 0.0 0.0
1.0 0.0 0.0
0.0 1.0 0.0

POLYGONS 1 4
3 0 1 2

POINT_DATA 3
SCALARS scalar_data float 1
LOOKUP_TABLE default
1.0 2.0 3.0
";

    fn write_temp_vtk(content: &str, name: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(name);
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_read_tet_unstructured_grid() {
        let path = write_temp_vtk(TET_VTK, "panmesh_test_tet.vtk");
        let mesh = read_vtk(&path).expect("Failed to read tet VTK");

        assert_eq!(mesh.points.len(), 4);
        assert_eq!(mesh.points[0], [0.0, 0.0, 0.0]);
        assert_eq!(mesh.points[1], [1.0, 0.0, 0.0]);
        assert_eq!(mesh.points[2], [0.0, 1.0, 0.0]);
        assert_eq!(mesh.points[3], [0.0, 0.0, 1.0]);

        assert_eq!(mesh.cells.len(), 1);
        assert_eq!(mesh.cells[0].cell_type, "tetra");
        assert_eq!(mesh.cells[0].data.len(), 1);
        assert_eq!(mesh.cells[0].data[0], vec![0, 1, 2, 3]);

        let pressure = mesh.point_data.get("pressure").expect("Missing pressure");
        assert_eq!(pressure.len(), 4);
        assert!((pressure[0] - 1.0).abs() < 1e-10);
        assert!((pressure[1] - 2.0).abs() < 1e-10);
        assert!((pressure[2] - 3.0).abs() < 1e-10);
        assert!((pressure[3] - 4.0).abs() < 1e-10);

        let material = mesh
            .cell_data
            .get("material_id")
            .expect("Missing material_id");
        assert_eq!(material.len(), 1);
        assert_eq!(material[0].len(), 1);
        assert!((material[0][0] - 42.0).abs() < 1e-10);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_read_mixed_unstructured_grid() {
        let path = write_temp_vtk(MIXED_VTK, "panmesh_test_mixed.vtk");
        let mesh = read_vtk(&path).expect("Failed to read mixed VTK");

        assert_eq!(mesh.points.len(), 6);

        assert_eq!(mesh.cells.len(), 2);
        assert_eq!(mesh.cells[0].cell_type, "triangle");
        assert_eq!(mesh.cells[0].data.len(), 2);
        assert_eq!(mesh.cells[0].data[0], vec![0, 1, 2]);
        assert_eq!(mesh.cells[0].data[1], vec![0, 2, 3]);

        assert_eq!(mesh.cells[1].cell_type, "tetra");
        assert_eq!(mesh.cells[1].data.len(), 1);
        assert_eq!(mesh.cells[1].data[0], vec![0, 1, 4, 5]);

        let temp = mesh
            .point_data
            .get("temperature")
            .expect("Missing temperature");
        assert_eq!(temp.len(), 6);

        let region = mesh.cell_data.get("region").expect("Missing region");
        assert_eq!(region.len(), 2);
        assert_eq!(region[0].len(), 2);
        assert!((region[0][0] - 1.0).abs() < 1e-10);
        assert!((region[0][1] - 2.0).abs() < 1e-10);
        assert_eq!(region[1].len(), 1);
        assert!((region[1][0] - 3.0).abs() < 1e-10);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_read_polydata_triangle() {
        let path = write_temp_vtk(TRI_POLYDATA_VTK, "panmesh_test_tri.vtk");
        let mesh = read_vtk(&path).expect("Failed to read polydata VTK");

        assert_eq!(mesh.points.len(), 3);
        assert_eq!(mesh.points[0], [0.0, 0.0, 0.0]);
        assert_eq!(mesh.points[1], [1.0, 0.0, 0.0]);
        assert_eq!(mesh.points[2], [0.0, 1.0, 0.0]);

        assert_eq!(mesh.cells.len(), 1);
        assert_eq!(mesh.cells[0].cell_type, "triangle");
        assert_eq!(mesh.cells[0].data.len(), 1);
        assert_eq!(mesh.cells[0].data[0], vec![0, 1, 2]);

        let scalar = mesh
            .point_data
            .get("scalar_data")
            .expect("Missing scalar_data");
        assert_eq!(scalar.len(), 3);
        assert!((scalar[0] - 1.0).abs() < 1e-10);
        assert!((scalar[1] - 2.0).abs() < 1e-10);
        assert!((scalar[2] - 3.0).abs() < 1e-10);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = read_vtk(std::path::Path::new("/nonexistent/file.vtk"));
        assert!(result.is_err());
    }
}
