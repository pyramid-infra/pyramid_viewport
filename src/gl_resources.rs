extern crate gl;
extern crate image;

use resources::*;
use image::GenericImage;
use libc::types::common::c95::c_void;
use image::RgbaImage;
use gl::types::*;
use std::mem;
use std::ptr;
use std::ffi::CString;

pub struct GLMesh {
    pub vao: GLuint,
    pub nindices: GLint
}

pub struct GLTexture {
    pub texture: GLuint
}


pub fn create_mesh(shader_program: GLuint, mesh: &Mesh) -> GLMesh {
    let mut vao = 0;
    let mut vbo = 0;
    let mut ebo = 0;

    unsafe {
        // Create Vertex Array Object
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        // Create a Vertex Buffer Object and copy the vertex data to it
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
            (mesh.vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            mem::transmute(&mesh.vertices[0]),
            gl::STATIC_DRAW);

        // Element buffer
        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
            (mesh.indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
            mem::transmute(&mesh.indices[0]),
            gl::STATIC_DRAW);

        // Specify the layout of the vertex data
        let pos_attr = gl::GetAttribLocation(shader_program, CString::new("position").unwrap().as_ptr()) as GLuint;
        let texcord_attr = gl::GetAttribLocation(shader_program, CString::new("texcoord").unwrap().as_ptr()) as GLuint;
        gl::EnableVertexAttribArray(pos_attr);
        gl::EnableVertexAttribArray(texcord_attr);
        gl::VertexAttribPointer(pos_attr, 3, gl::FLOAT, gl::FALSE as GLboolean, 5 * mem::size_of::<GLfloat>() as GLint, ptr::null());
        gl::VertexAttribPointer(texcord_attr, 2, gl::FLOAT, gl::FALSE as GLboolean, 5 * mem::size_of::<GLfloat>() as GLint, (3 * mem::size_of::<GLfloat>()) as *const GLvoid);
    }
    return GLMesh {
        vao: vao,
        nindices: mesh.indices.len() as GLint
    };
}

pub fn create_texture(image: RgbaImage) -> GLTexture {
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
