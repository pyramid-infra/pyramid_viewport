extern crate cgmath;
extern crate gl;
extern crate image;

use pyramid::document::*;
use resources::*;
use gl_resources::*;
use promise::*;

use gl::types::*;
use std::fs::File;
use std::str;
use std::io::prelude::*;
use cgmath::*;
use std::ptr;
use std::ffi::CString;
use std::mem;



pub struct Renderer {
    nodes: Vec<RenderNode>,
    pub shader_program: GLuint
}

pub struct RenderNode {
    pub id: u64,
    pub mesh: Promise<GLMesh>,
    pub transform: Matrix4<f32>,
    pub texture: Promise<GLTexture>
}


impl Renderer {
    pub fn new() -> Renderer {
        // Create GLSL shaders
        let vs = compile_shader(&string_from_file("shaders/basic_vs.glsl").unwrap().to_string(), gl::VERTEX_SHADER);
        let fs = compile_shader(&string_from_file("shaders/basic_fs.glsl").unwrap().to_string(), gl::FRAGMENT_SHADER);
        let shader_program = link_program(vs, fs);

        unsafe {
            // Use shader program
            gl::UseProgram(shader_program);
            gl::BindFragDataLocation(shader_program, 0,
                                     CString::new("out_color").unwrap().as_ptr());
            gl::Disable(gl::CULL_FACE);
        }

        Renderer {
            nodes: vec![],
            shader_program: shader_program
        }
    }
    pub fn render(&self) {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            for node in &self.nodes {
                if let &Some(ref mesh) = &*node.mesh.value() {
                    gl::BindVertexArray(mesh.vao);
                }

                let uniTrans = gl::GetUniformLocation(self.shader_program, CString::new("trans").unwrap().as_ptr());

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

fn string_from_file(filename: &str) -> Option<String> {
    let mut file = File::open(filename).unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string);
    return Some(string);
}


fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;

    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("{}", str::from_utf8(&buf).ok().expect("ShaderInfoLog not valid utf8"));
        }
    }
    shader
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint { unsafe {
    let program = gl::CreateProgram();
    gl::AttachShader(program, vs);
    gl::AttachShader(program, fs);
    gl::LinkProgram(program);
    // Get the link status
    let mut status = gl::FALSE as GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

    // Fail on error
    if status != (gl::TRUE as GLint) {
        let mut len: GLint = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = Vec::with_capacity(len as usize);
        buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
        gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
        panic!("{}", str::from_utf8(&buf).ok().expect("ProgramInfoLog not valid utf8"));
    }
    program
} }
