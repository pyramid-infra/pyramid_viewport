extern crate cgmath;
extern crate gl;
extern crate image;

use pyramid::document::*;
use resources::*;
use gl_resources::*;
use ppromise::*;

use gl::types::*;
use std::fs::File;
use std::io::prelude::*;
use cgmath::*;
use std::ptr;
use std::ffi::CString;
use std::mem;



pub struct Renderer {
    nodes: Vec<RenderNode>
}

pub struct RenderNode {
    pub id: u64,
    pub shader: GLuint,
    pub mesh: Promise<GLMesh>,
    pub transform: Matrix4<f32>,
    pub texture: Promise<GLTexture>
}


impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            nodes: vec![]
        }
    }
    pub fn render(&self) {
        unsafe {
            gl::Disable(gl::CULL_FACE);
            gl::Enable(gl::DEPTH_TEST);
            for node in &self.nodes {
                gl::UseProgram(node.shader);
                gl::BindFragDataLocation(node.shader, 0,
                                         CString::new("out_color").unwrap().as_ptr());

                if let &Some(ref mesh) = &*node.mesh.value() {
                    gl::BindVertexArray(mesh.vao);
                }

                let uniTrans = gl::GetUniformLocation(node.shader, CString::new("trans").unwrap().as_ptr());

                let t: [f32; 16] = mem::transmute(node.transform);
                gl::UniformMatrix4fv(uniTrans, 1, gl::FALSE, t.as_ptr());

                if let &Some(ref texture) = &*node.texture.value() {
                    gl::BindTexture(gl::TEXTURE_2D, texture.texture);
                }
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);

                if let &Some(ref mesh) = &*node.mesh.value() {
                    gl::DrawElements(gl::TRIANGLES, mesh.nindices, gl::UNSIGNED_INT, ptr::null());
                }
            }
        };
    }

    pub fn add_node(&mut self, node: RenderNode) {
        self.nodes.push(node);
    }
    pub fn remove_node(&mut self, key: &u64) {
        self.nodes.retain(|x| x.id != *key);
    }
    pub fn set_transform(&mut self, key: &u64, transform: Matrix4<f32>) {
        match self.nodes.iter_mut().find(|x| x.id == *key) {
            Some(node) => node.transform = transform,
            None => {}
        }
    }
}
