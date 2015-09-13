extern crate gl;

use pyramid::pon::*;
use gl_resources::*;
use pon_to_resource::*;
use renderer::*;
use mesh::*;
use gl::types::*;
use pyramid::document::*;

use std::path::PathBuf;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::rc::Rc;
use std::cell::RefCell;
use image::RgbaImage;
use ppromise::*;


pub struct Resources {
    pub meshes: HashMap<Pon, Promise<Rc<Mesh>>>,
    pub gl_meshes: HashMap<Pon, Promise<Rc<GLMesh>>>,
    pub gl_shader_programs: HashMap<Pon, Promise<Rc<GLShaderProgram>>>,
    pub gl_vertex_arrays: HashMap<Pon, Promise<Rc<GLVertexArray>>>,
    pub textures: HashMap<Pon, Promise<Rc<Texture>>>,
    pub gl_textures: HashMap<Pon, Promise<Rc<GLTexture>>>,

    root_path: PathBuf,
    async_runner: AsyncRunner
}

impl Resources {
    pub fn new(root_path: PathBuf) -> Resources {
        Resources {
            root_path: root_path,
            meshes: HashMap::new(),
            gl_meshes: HashMap::new(),
            gl_shader_programs: HashMap::new(),
            gl_vertex_arrays: HashMap::new(),
            textures: HashMap::new(),
            gl_textures: HashMap::new(),
            async_runner: AsyncRunner::new_pooled(4)
        }
    }
    pub fn get(&mut self, document: &Document, mesh_key: &Pon, shader_program_key: &Pon, texture_keys: Vec<Pon>)
        -> Promise<RenderNodeResources> {
        let mut gl_shader_program = match self.gl_shader_programs.entry(shader_program_key.clone())  {
            Entry::Occupied(o) => {
                o.into_mut()
            },
            Entry::Vacant(v) => {
                let shader = pon_to_shader(&self.root_path, shader_program_key).unwrap();
                let vs = &GLShader::new(&shader.vertex_src, gl::VERTEX_SHADER, &shader.vertex_debug_source_name);
                let fs = &GLShader::new(&shader.fragment_src, gl::FRAGMENT_SHADER, &shader.fragment_debug_source_name);
                v.insert(Promise::resolved(Rc::new(GLShaderProgram::new(vs, fs))))
            }
        }.then(|x| x.clone());
        let gl_vertex_array_key = Pon::Array(vec![mesh_key.clone(), shader_program_key.clone()]);
        let mut gl_vertex_array = match self.gl_vertex_arrays.entry(gl_vertex_array_key.clone())  {
            Entry::Occupied(o) => {
                o.into_mut()
            },
            Entry::Vacant(v) => {
                let mut gl_mesh = match self.gl_meshes.entry(mesh_key.clone())  {
                    Entry::Occupied(o) => {
                        o.into_mut()
                    },
                    Entry::Vacant(v) => {
                        let mut mesh = match self.meshes.entry(mesh_key.clone()) {
                            Entry::Occupied(o) => {
                                o.into_mut()
                            },
                            Entry::Vacant(v) => {
                                let mesh_key = mesh_key.clone();
                                let root_path = self.root_path.clone();
                                let p = Promise::resolved(pon_to_mesh(document, &root_path, &mesh_key).unwrap());
                                v.insert(p)
                            }
                        }.then(|x| x.clone());
                        v.insert(mesh.then(|mesh| { println!("rc mesh to gl mesh"); Rc::new(GLMesh::new(mesh)) }))
                    }
                }.then(|x| x.clone());
                v.insert((&mut gl_shader_program.then(|x| x.clone()), &mut gl_mesh).join().then(|&(ref gl_shader_program, ref gl_mesh)| {
                    Rc::new(GLVertexArray::new(gl_shader_program, gl_mesh))
                }))
            }
        }.then(|x| x.clone());
        let mut gl_textures = vec![];
        for texture_key in texture_keys {
            let gl_texture = match self.gl_textures.entry(texture_key.clone())  {
                Entry::Occupied(o) => {
                    o.into_mut()
                },
                Entry::Vacant(v) => {
                    let texture = match self.textures.entry(texture_key.clone()) {
                        Entry::Occupied(o) => {
                            o.into_mut()
                        },
                        Entry::Vacant(v) => {
                            let texture_key = texture_key.clone();
                            let root_path = self.root_path.clone();
                            let p = self.async_runner
                                .exec_async(move || pon_to_texture(&root_path, &texture_key).unwrap())
                                .then_move(|texture| Rc::new(texture));
                            v.insert(p)
                        }
                    };
                    v.insert(texture.then(|texture| { println!("rc texture to gl texture"); Rc::new(GLTexture::new(texture)) }))
                }
            };
            gl_textures.push(gl_texture.then(|x| x.clone()));
        }
        (&mut gl_shader_program, &mut gl_vertex_array, &mut gl_textures.join()).join().then_move(|(sp, va, txs)| {
            RenderNodeResources {
                shader: sp,
                vertex_array: va,
                textures: txs
            }
        })
    }
    pub fn update(&mut self) {
        self.async_runner.try_resolve_all();
    }
}
