extern crate gl;
extern crate image;

use mesh::*;
use image::GenericImage;
use libc::types::common::c95::c_void;
use image::RgbaImage;
use gl::types::*;
use std::mem;
use std::ptr;
use std::ffi::CString;
use std::str;
use std::rc::Rc;

use propnode_to_resource::*;


#[derive(Clone)]
pub struct GLMesh {
    pub layout: Layout,
    pub vbo: GLuint,
    pub ebo: GLuint,
    pub nindices: GLint
}

impl GLMesh {
    pub fn new(mesh: &Mesh) -> GLMesh {
        let mut vbo = 0;
        let mut ebo = 0;

        unsafe {
            // Create a Vertex Buffer Object and copy the vertex data to it
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER,
                (mesh.vertex_data.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                mem::transmute(&mesh.vertex_data[0]),
                gl::STATIC_DRAW);

            // Element buffer
            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                (mesh.element_data.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                mem::transmute(&mesh.element_data[0]),
                gl::STATIC_DRAW);
        }
        return GLMesh {
            layout: mesh.layout.clone(),
            vbo: vbo,
            ebo: ebo,
            nindices: mesh.element_data.len() as GLint
        };
    }

}

pub struct GLVertexArray {
    pub mesh: Rc<GLMesh>,
    pub vao: GLuint
}

impl GLVertexArray {
    pub fn new(shader_program: &Rc<GLShader>, mesh: &Rc<GLMesh>) -> GLVertexArray {
        let mut vao = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, mesh.vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, mesh.ebo);

            // Specify the layout of the vertex data
            for attr in &mesh.layout.attributes {
                let gl_attr = gl::GetAttribLocation(shader_program.shader, CString::new(attr.name.clone()).unwrap().as_ptr()) as GLuint;
                gl::EnableVertexAttribArray(gl_attr);
                let stride = (mesh.layout.stride * mem::size_of::<GLfloat>()) as GLint;
                let offset = (attr.offset * mem::size_of::<GLfloat>()) as *const GLvoid;
                gl::VertexAttribPointer(gl_attr, attr.size as GLint, gl::FLOAT, gl::FALSE as GLboolean, stride, offset);
            }
        }
        GLVertexArray {
            mesh: mesh.clone(),
            vao: vao
        }
    }
}
impl Drop for GLVertexArray {
    fn drop(&mut self) {
        println!("TODO: clean up GL stuff");
    }
}

pub struct GLTexture {
    pub texture: GLuint
}

impl GLTexture {
    pub fn new(image: RgbaImage) -> GLTexture {
        println!("create_texture START");
        let mut tex = 0;
        unsafe {
            gl::GenTextures(1, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, image.width() as GLint, image.height() as GLint, 0,
                gl::RGBA, gl::UNSIGNED_BYTE, mem::transmute(&image.into_raw()[0]));
        }
        println!("create_texture END");
        return GLTexture {
            texture: tex
        };
    }
}

pub struct GLShader {
    pub shader: GLuint
}

impl GLShader {
    pub fn new(shader: &ShaderSource) -> GLShader {
        println!("SHADERS {:?}", shader);
        let vs = compile_shader(&shader.vertex_src, gl::VERTEX_SHADER);
        let fs = compile_shader(&shader.fragment_src, gl::FRAGMENT_SHADER);
        GLShader {
            shader: link_program(vs, fs)
        }
    }
}

pub fn compile_shader(src: &str, ty: GLenum) -> GLuint {
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

pub fn link_program(vs: GLuint, fs: GLuint) -> GLuint { unsafe {
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
