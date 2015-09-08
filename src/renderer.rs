extern crate cgmath;
extern crate gl;
extern crate image;

use pyramid::document::*;
use resources::*;
use gl_resources::*;
use shader_uniforms::*;

use gl::types::*;
use std::fs::File;
use std::io::prelude::*;
use cgmath::*;
use std::ptr;
use std::ffi::CString;
use std::mem;
use std::rc::Rc;



pub struct Renderer {
    opaque_nodes: Vec<RenderNode>,
    translucent_nodes: Vec<RenderNode>,
    pub camera: Matrix4<f32>
}


pub struct RenderNode {
    pub id: u64,
    pub shader: Rc<GLShaderProgram>,
    pub vertex_array: Rc<GLVertexArray>,
    pub transform: Matrix4<f32>,
    pub textures: Vec<(String, Rc<GLTexture>)>,
    pub uniforms: ShaderUniforms,
    pub alpha: bool
}


impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            opaque_nodes: vec![],
            translucent_nodes: vec![],
            camera: Matrix4::identity()
        }
    }
    fn draw_node(&self, node: &RenderNode) {
        unsafe {
            gl::UseProgram(node.shader.program);
            gl::BindFragDataLocation(node.shader.program, 0,
                                     CString::new("out_color").unwrap().as_ptr());

            gl::BindVertexArray(node.vertex_array.vao);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, node.vertex_array.mesh.ebo);

            let trans_loc = gl::GetUniformLocation(node.shader.program, CString::new("transform").unwrap().as_ptr());

            let transform = self.camera * node.transform;
            transform.gl_write_to_uniform(trans_loc);

            for &(ref name, ref uniform) in &node.uniforms.0 {
                let loc = gl::GetUniformLocation(node.shader.program, CString::new(name.to_string()).unwrap().as_ptr());
                uniform.gl_write_to_uniform(loc);
            }

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
    }
    pub fn render(&self) {
        unsafe {
            gl::DepthMask(gl::TRUE);
            gl::Enable(gl::DEPTH_TEST);
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Disable(gl::BLEND);
            for node in &self.opaque_nodes {
                self.draw_node(node);
            }
            gl::DepthMask(gl::FALSE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            for node in &self.translucent_nodes {
                self.draw_node(node);
            }
        };
    }

    pub fn add_node(&mut self, node: RenderNode) {
        if node.alpha {
            self.translucent_nodes.push(node);
        } else {
            self.opaque_nodes.push(node);
        }
    }
    pub fn remove_node(&mut self, key: &u64) {
        self.translucent_nodes.retain(|x| x.id != *key);
        self.opaque_nodes.retain(|x| x.id != *key);
    }
    pub fn set_transform(&mut self, key: &u64, transform: Matrix4<f32>) {
        match self.translucent_nodes.iter_mut().find(|x| x.id == *key) {
            Some(node) => node.transform = transform,
            None => {}
        }
        match self.opaque_nodes.iter_mut().find(|x| x.id == *key) {
            Some(node) => node.transform = transform,
            None => {}
        }
    }
}
