extern crate cgmath;
extern crate gl;
extern crate image;

use pyramid::document::*;
use resources::*;
use gl_resources::*;

use gl::types::*;
use std::fs::File;
use std::io::prelude::*;
use cgmath::*;
use std::ptr;
use std::ffi::CString;
use std::mem;
use std::rc::Rc;



pub struct Renderer {
    nodes: Vec<RenderNode>,
    pub camera: Matrix4<f32>
}

#[derive(Clone)]
pub struct RenderNode {
    pub id: u64,
    pub shader: Rc<GLShaderProgram>,
    pub vertex_array: Rc<GLVertexArray>,
    pub transform: Matrix4<f32>,
    pub textures: Vec<(String, Rc<GLTexture>)>
}


impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            nodes: vec![],
            camera: Matrix4::identity()
        }
    }
    pub fn render(&self) {
        unsafe {
            //gl::Disable(gl::CULL_FACE);
            gl::Enable(gl::DEPTH_TEST);
            for node in &self.nodes {
                gl::UseProgram(node.shader.program);
                gl::BindFragDataLocation(node.shader.program, 0,
                                         CString::new("out_color").unwrap().as_ptr());

                gl::BindVertexArray(node.vertex_array.vao);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, node.vertex_array.mesh.ebo);

                let trans_loc = gl::GetUniformLocation(node.shader.program, CString::new("trans").unwrap().as_ptr());

                let transform = self.camera * node.transform;
                let t: [f32; 16] = mem::transmute(transform);
                gl::UniformMatrix4fv(trans_loc, 1, gl::FALSE, t.as_ptr());

                let mut texi = 0;
                for &(ref name, ref texture) in &node.textures {
                    gl::ActiveTexture(gl::TEXTURE0 + texi);
                    gl::BindTexture(gl::TEXTURE_2D, texture.texture);
                    let tex_loc = gl::GetUniformLocation(node.shader.program, CString::new(name.to_string()).unwrap().as_ptr());
                    gl::Uniform1i(tex_loc, texi as GLint);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
                    texi += 1;
                }

                gl::DrawElements(gl::TRIANGLES, node.vertex_array.mesh.nindices, gl::UNSIGNED_INT, ptr::null());
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
