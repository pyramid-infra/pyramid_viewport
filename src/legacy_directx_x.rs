// DirectX .x is an old legacy model format. Not recommended for any use, only reason it's
// here is because a lot of the old DML files were in .x format

use resources::*;

#[derive(PartialEq, Debug, Clone)]
pub struct DXFrame {
    pub name: String,
    pub transform: Vec<f32>,
    pub mesh: Option<DXMesh>
}

#[derive(PartialEq, Debug, Clone)]
pub struct DXMesh {
    pub name: String,
    pub vertices: Vec<Vec<f32>>,
    pub indices: Vec<Vec<i64>>,
    pub normals: DXMeshNormals,
    pub texcoords: Vec<Vec<f32>>
}

#[derive(PartialEq, Debug, Clone)]
pub struct DXMeshNormals {
    pub vertices: Vec<Vec<f32>>,
    pub indices: Vec<Vec<i64>>,
}

impl DXMesh {
    pub fn to_mesh(&self) -> Mesh {
        let mut verts = vec![];
        for i in 0..self.vertices.len() {
            verts.push(self.vertices[i][0]);
            verts.push(self.vertices[i][1]);
            verts.push(self.vertices[i][2]);
            verts.push(self.texcoords[i][0]);
            verts.push(self.texcoords[i][1]);
        }
        let mut indices = vec![];
        for inds in &self.indices {
            if inds.len() == 4 {
                indices.push(inds[0] as u32);
                indices.push(inds[1] as u32);
                indices.push(inds[2] as u32);
                indices.push(inds[0] as u32);
                indices.push(inds[2] as u32);
                indices.push(inds[3] as u32);
            } else if inds.len() == 3 {
                indices.push(inds[0] as u32);
                indices.push(inds[1] as u32);
                indices.push(inds[2] as u32);
            }
        }
        return Mesh {
            vertices: verts,
            indices: indices
        };
    }
}
