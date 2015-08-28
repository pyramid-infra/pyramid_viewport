// DirectX .x is an old legacy model format. Not recommended for any use, only reason it's
// here is because a lot of the old DML files were in .x format

use resources::*;

#[derive(PartialEq, Debug, Clone)]
pub enum DXNode {
    Obj {
        name: String,
        arg: Option<String>,
        children: Vec<DXNode>
    },
    Qualifier(String),
    Value(f32),
    Values(Vec<Vec<Vec<f32>>>)
}

impl DXNode {
    fn get_mesh_node(&self, id: &String) -> Option<&DXNode> {
        match self {
            &DXNode::Obj { ref name, ref arg, ref children } => {
                if let &Some(ref arg) = arg {
                    if name == "Mesh" && *arg == *id {
                        return Some(self);
                    }
                }
                for k in children {
                    if let Some(n) = k.get_mesh_node(id) {
                        return Some(n);
                    }
                }
                None
            },
            _ => None
        }
    }
    pub fn to_mesh(&self, id: String) -> Result<Mesh, String> {
        let node_children = match self.get_mesh_node(&id) {
            Some(&DXNode::Obj { ref children, .. }) => children,
            _ => return Err(format!("Can't find mesh node: {}", id))
        };
        let verts_node = match &node_children[1] {
            &DXNode::Values(ref vals) => vals,
            _ => return Err(format!("Can't find vertices for mesh {}", id))
        };
        let indices_node = match &node_children[3] {
            &DXNode::Values(ref vals) => vals,
            _ => return Err(format!("Can't find indices for mesh {}", id))
        };
        let texcords_node = match node_children.iter().find(|x| {
            if let &&DXNode::Obj { ref name, .. } = x {
                if name == "MeshTextureCoords" {
                    return true;
                }
            }
            false
        }) {
            Some(&DXNode::Obj { ref children, .. }) => match &children[1] {
                &DXNode::Values(ref values) => values,
                _ => return Err(format!("Can't find texcords for mesh {}", id))
            },
            _ => return Err(format!("Can't find texcords for mesh {}", id))
        };
        let mut verts = vec![];
        for i in 0..verts_node.len() {
            verts.push(verts_node[i][0][0]);
            verts.push(verts_node[i][1][0]);
            verts.push(verts_node[i][2][0]);
            verts.push(texcords_node[i][0][0]);
            verts.push(texcords_node[i][1][0]);
        }
        let mut indices = vec![];
        for inds in indices_node {
            if inds[1].len() == 4 {
                indices.push(inds[1][0] as u32);
                indices.push(inds[1][1] as u32);
                indices.push(inds[1][2] as u32);
                indices.push(inds[1][0] as u32);
                indices.push(inds[1][2] as u32);
                indices.push(inds[1][3] as u32);
            } else if inds[1].len() == 3 {
                indices.push(inds[1][0] as u32);
                indices.push(inds[1][1] as u32);
                indices.push(inds[1][2] as u32);
            }
        }
        return Ok(Mesh {
            vertices: verts,
            indices: indices
        });
    }
}
