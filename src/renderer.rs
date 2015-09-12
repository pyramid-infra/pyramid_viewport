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

#[derive(Debug)]
pub struct RenderNodeResources {
    pub shader: Rc<GLShaderProgram>,
    pub vertex_array: Rc<GLVertexArray>,
    pub textures: Vec<Rc<GLTexture>>,
}

#[derive(Debug)]
pub struct RenderNodeConfig {
    pub texture_ids: Vec<String>,
    pub transform: Matrix4<f32>,
    pub uniforms: ShaderUniforms,
    pub alpha: bool
}

#[derive(Debug)]
pub struct RenderNode {
    pub id: u64,
    pub resources: RenderNodeResources,
    pub config: RenderNodeConfig
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
            gl::UseProgram(node.resources.shader.program);
            gl::BindFragDataLocation(node.resources.shader.program, 0,
                                     CString::new("out_color").unwrap().as_ptr());

            gl::BindVertexArray(node.resources.vertex_array.vao);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, node.resources.vertex_array.mesh.ebo);

            let trans_loc = gl::GetUniformLocation(node.resources.shader.program, CString::new("transform").unwrap().as_ptr());

            let transform = self.camera * node.config.transform;
            transform.gl_write_to_uniform(trans_loc);

            for &(ref name, ref uniform) in &node.config.uniforms.0 {
                let loc = gl::GetUniformLocation(node.resources.shader.program, CString::new(name.to_string()).unwrap().as_ptr());
                uniform.gl_write_to_uniform(loc);
            }

            for texi in 0..node.resources.textures.len() {
                let texture = &node.resources.textures[texi];
                let name = &node.config.texture_ids[texi];
                gl::ActiveTexture(gl::TEXTURE0 + texi as GLuint);
                gl::BindTexture(gl::TEXTURE_2D, texture.texture);
                let tex_loc = gl::GetUniformLocation(node.resources.shader.program, CString::new(name.to_string()).unwrap().as_ptr());
                gl::Uniform1i(tex_loc, texi as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            }

            gl::DrawElements(gl::TRIANGLES, node.resources.vertex_array.mesh.nindices, gl::UNSIGNED_INT, ptr::null());
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
        if node.config.alpha {
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
            Some(node) => node.config.transform = transform,
            None => {}
        }
        match self.opaque_nodes.iter_mut().find(|x| x.id == *key) {
            Some(node) => node.config.transform = transform,
            None => {}
        }
    }
}
