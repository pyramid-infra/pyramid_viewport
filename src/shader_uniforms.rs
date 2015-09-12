
use gl;
use gl::types::*;
use std::mem;
use cgmath::*;
use std::collections::HashMap;
use pyramid::pon::*;
use std::fmt::Debug;

#[derive(Debug)]
pub struct ShaderUniforms(pub Vec<(String, Box<ShaderUniform>)>);

pub trait ShaderUniform : Debug {
    fn gl_write_to_uniform(&self, uniform_location: GLint);
}
impl ShaderUniform for f32 {
    fn gl_write_to_uniform(&self, uniform_location: GLint) {
        unsafe {
            gl::Uniform1f(uniform_location, *self);
        }
    }
}
impl ShaderUniform for Vector3<f32> {
    fn gl_write_to_uniform(&self, uniform_location: GLint) {
        unsafe {
            gl::Uniform3f(uniform_location, self.x, self.y, self.z);
        }
    }
}
impl ShaderUniform for Matrix4<f32> {
    fn gl_write_to_uniform(&self, uniform_location: GLint) {
        unsafe {
            let t: [f32; 16] = mem::transmute(*self);
            gl::UniformMatrix4fv(uniform_location, 1, gl::FALSE, t.as_ptr());
        }
    }
}

impl<'a> Translatable<'a, ShaderUniforms> for Pon {
    fn inner_translate(&'a self) -> Result<ShaderUniforms, PonTranslateErr> {
        let obj: &HashMap<String, Pon> = try!(self.translate());
        let mut res: Vec<(String, Box<ShaderUniform>)> = vec![];
        for (name, value) in obj {
            if let Ok(v) = value.translate::<f32>() {
                res.push((name.to_string(), Box::new(v)));
            } else if let Ok(v) = value.translate::<Vector3<f32>>() {
                res.push((name.to_string(), Box::new(v)));
            } else if let Ok(v) = value.translate::<Matrix4<f32>>() {
                res.push((name.to_string(), Box::new(v)));
            } else {
                return Err(PonTranslateErr::InvalidValue { value: value.to_string() } )
            }
        }
        Ok(ShaderUniforms(res))
    }
}
