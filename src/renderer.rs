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
use std::collections::HashMap;
use std::cell::RefCell;



pub struct Renderer {
    opaque_nodes: Vec<Rc<RefCell<RenderNode>>>,
    translucent_nodes: Vec<Rc<RefCell<RenderNode>>>,
    nodes_by_id: HashMap<u64, Rc<RefCell<RenderNode>>>,
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
            nodes_by_id: HashMap::new(),
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
                self.draw_node(&*node.borrow());
            }
            gl::DepthMask(gl::FALSE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            for node in &self.translucent_nodes {
                self.draw_node(&*node.borrow());
            }
        };
    }

    pub fn add_node(&mut self, node: RenderNode) {
        let has_alpha = node.config.alpha;
        let id = node.id.clone();
        let node = Rc::new(RefCell::new(node));
        if has_alpha {
            self.translucent_nodes.push(node.clone());
        } else {
            self.opaque_nodes.push(node.clone());
        }
        self.nodes_by_id.insert(id, node);
    }
    pub fn remove_node(&mut self, key: &u64) {
        self.translucent_nodes.retain(|x| x.borrow().id != *key);
        self.opaque_nodes.retain(|x| x.borrow().id != *key);
        self.nodes_by_id.remove(key);
    }
    pub fn set_transform(&mut self, key: &u64, transform: Matrix4<f32>) {
        match self.nodes_by_id.get_mut(key) {
            Some(node) => node.borrow_mut().config.transform = transform,
            None => {}
        }
    }
}
