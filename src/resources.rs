extern crate gl;

use pyramid::pon::*;
use gl_resources::*;
use pon_to_resource::*;
use mesh::*;
use gl::types::*;

use std::path::PathBuf;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use image::RgbaImage;


pub struct ResourceContainer<T> {
    construct: Box<Fn(&Pon) -> T>,
    resources: HashMap<Pon, Rc<T>>
}

impl<T> ResourceContainer<T> {
    pub fn new<F: Fn(&Pon) -> T + 'static>(construct: F) -> ResourceContainer<T> {
        ResourceContainer {
            construct: Box::new(construct),
            resources: HashMap::new()
        }
    }
    pub fn get(&mut self, key: &Pon) -> Rc<T> {
        let value = match self.resources.get(key) {
            Some(value) => return value.clone(),
            None => {}
        };
        let resource = Rc::new(self.construct.call((key,)));
        self.resources.insert(key.clone(), resource.clone());
        resource
    }
    pub fn set(&mut self, key: &Pon, value: Rc<T>) {
        self.resources.insert(key.clone(), value.clone());
    }
}



pub struct Resources {
    pub gl_shader_programs: Rc<RefCell<ResourceContainer<GLShaderProgram>>>,

    pub gl_meshes: Rc<RefCell<ResourceContainer<GLMesh>>>,
    pub gl_vertex_arrays: Rc<RefCell<ResourceContainer<GLVertexArray>>>,

    pub gl_textures: Rc<RefCell<ResourceContainer<GLTexture>>>
}

impl Resources {
    pub fn new(root_path: PathBuf) -> Resources {
        let root_path2 = root_path.clone();
        let meshes = Rc::new(RefCell::new(ResourceContainer::new(move |key| pon_to_mesh(&root_path2, key).unwrap())));
        let gl_meshes = Rc::new(RefCell::new(ResourceContainer::new(move |key| {
            let mesh = meshes.borrow_mut().get(key).clone();
            GLMesh::new(&*mesh)
        })));
        let root_path2 = root_path.clone();
        let gl_shader_programs = Rc::new(RefCell::new(ResourceContainer::new(move |key| {
            let shader = pon_to_shader(&root_path2, key).unwrap();
            GLShaderProgram::new(&GLShader::new(&shader.vertex_src, gl::VERTEX_SHADER), &GLShader::new(&shader.fragment_src, gl::FRAGMENT_SHADER))
        })));
        let gl_shader_programs2 = gl_shader_programs.clone();
        let gl_meshes2 = gl_meshes.clone();
        let gl_vertex_arrays = Rc::new(RefCell::new(ResourceContainer::new(move |key| {
            let arr = key.as_array().unwrap();
            let shader_key = arr[0].clone();
            let mesh_key = arr[1].clone();
            let gl_shader = gl_shader_programs2.borrow_mut().get(&shader_key);
            let gl_mesh = gl_meshes2.borrow_mut().get(&mesh_key);
            GLVertexArray::new(&gl_shader, &gl_mesh)
        })));
        let textures = Rc::new(RefCell::new(ResourceContainer::new(move |key| pon_to_texture(&root_path, &key).unwrap())));
        let gl_textures = Rc::new(RefCell::new(ResourceContainer::new(move |key| {
            let texture = textures.borrow_mut().get(key).clone();
            GLTexture::new((*texture).clone())
        })));

        Resources {
            gl_shader_programs: gl_shader_programs,
            gl_meshes: gl_meshes,
            gl_vertex_arrays: gl_vertex_arrays,
            gl_textures: gl_textures,
        }
    }
}
