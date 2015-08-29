#![feature(box_patterns, rc_weak, convert, unboxed_closures, core)]

extern crate gl;
extern crate libc;
extern crate image;
extern crate cgmath;
extern crate time;
extern crate pyramid;
extern crate glutin;
extern crate ppromise;

mod renderer;
mod resources;
mod matrix;
mod gl_resources;
mod fps_counter;

use pyramid::interface::*;
use pyramid::propnode::*;
use pyramid::document::*;
use pyramid::*;

use renderer::*;
use ppromise::*;
use gl_resources::*;
use resources::*;
use fps_counter::*;

use image::RgbaImage;
use std::collections::HashMap;
use std::collections::HashSet;
use cgmath::*;
use std::mem;
use gl::types::*;
use std::str;
use std::path::Path;
use std::path::PathBuf;

static SHADER_BASIC_VS: &'static [u8] = include_bytes!("../shaders/basic_vs.glsl");
static SHADER_BASIC_FS: &'static [u8] = include_bytes!("../shaders/basic_fs.glsl");

pub struct ViewportSubSystem {
    root_path: PathBuf,
    window: glutin::Window,
    renderer: Renderer,
    meshes_to_resolve: Vec<AsyncPromise<Mesh>>,
    textures_to_resolve: Vec<AsyncPromise<RgbaImage>>,
    meshes: HashMap<PropNode, Promise<GLMesh>>,
    textures: HashMap<PropNode, Promise<GLTexture>>,
    shaders: HashMap<String, GLuint>,
    default_texture: PropNode,
    fps_counter: FpsCounter
}

impl ViewportSubSystem {
    pub fn new(root_path: PathBuf) -> ViewportSubSystem {
        let window = glutin::Window::new().unwrap();

        unsafe { window.make_current() };

        unsafe {
            gl::load_with(|symbol| window.get_proc_address(symbol));
            gl::ClearColor(1.0, 1.0, 0.0, 1.0);
        }

        let mut viewport = ViewportSubSystem {
            root_path: root_path,
            window: window,
            renderer: Renderer::new(),
            meshes_to_resolve: vec![],
            textures_to_resolve: vec![],
            meshes: HashMap::new(),
            textures: HashMap::new(),
            shaders: HashMap::new(),
            default_texture: propnode_parser::parse("static_texture { pixels: [255, 0, 0, 255], width: 1, height: 1 }").unwrap(),
            fps_counter: FpsCounter::new()
        };

        let vs = compile_shader(str::from_utf8(SHADER_BASIC_VS).unwrap(), gl::VERTEX_SHADER);
        let fs = compile_shader(str::from_utf8(SHADER_BASIC_FS).unwrap(), gl::FRAGMENT_SHADER);
        let shader_program = link_program(vs, fs);

        viewport.shaders.insert("basic".to_string(), shader_program);

        viewport
    }
}

impl ViewportSubSystem {

    fn renderer_add(&mut self, system: &ISystem, entity_id: &EntityId) {
        let shader = *self.shaders.get("basic").unwrap();
        let mesh_pn: PropNode = match system.get_property_value(entity_id, "mesh") {
            Ok(mesh) => mesh,
            Err(err) => return ()
        };
        let mesh = match self.meshes.get(&mesh_pn) {
            Some(mesh) => Some(mesh.clone()),
            None => None
        };
        let mesh = match mesh {
            Some(mesh) => mesh,
            None => {
                let mesh_pn2 = mesh_pn.clone();
                let shader = shader.clone();
                let root_path = self.root_path.clone();
                let mesh_async_promise = AsyncPromise::new(move || load_mesh(&root_path, &mesh_pn2).unwrap());
                let gl_mesh_promise = mesh_async_promise.promise.then(move |mesh| gl_resources::create_mesh(shader, mesh));
                self.meshes_to_resolve.push(mesh_async_promise);
                self.meshes.insert(mesh_pn, gl_mesh_promise.clone());
                gl_mesh_promise
            }
        };
        let texture_pn: PropNode = match system.get_property_value(entity_id, "texture") {
            Ok(pn) => pn,
            Err(err) => self.default_texture.clone()
        };
        let texture = match self.textures.get(&texture_pn) {
            Some(texture) => Some(texture.clone()),
            None => None
        };
        let texture = match texture {
            Some(texture) => texture,
            None => {
                let texture_pn2 = texture_pn.clone();
                let root_path = self.root_path.clone();
                let texture_async_promise = AsyncPromise::new(move || load_texture(&root_path, &texture_pn2).unwrap());
                let gl_texture_promise = texture_async_promise.promise.then(move |texture| gl_resources::create_texture(texture.clone()));
                self.textures_to_resolve.push(texture_async_promise);
                self.textures.insert(texture_pn, gl_texture_promise.clone());
                gl_texture_promise
            }
        };
        let node = {
            RenderNode {
                id: *entity_id,
                shader: shader,
                mesh: mesh,
                texture: texture,
                transform: match system.get_property_value(entity_id, "transform") {
                    Ok(trans) => matrix::from_prop_node(&trans).unwrap(),
                    Err(err) => Matrix4::identity()
                }
            }
        };
        self.renderer.add_node(node);
    }
    fn renderer_remove(&mut self, entity_id: &EntityId) {
        self.renderer.remove_node(entity_id);
    }
}

impl ISubSystem for ViewportSubSystem {

    fn on_property_value_change(&mut self, system: &mut ISystem, prop_refs: &Vec<PropRef>) {
        //println!("CHANGED {:?}", prop_refs);
        let renderable_changed: HashSet<EntityId> = prop_refs.iter()
            .filter_map(|pr| {
                if (pr.property_key == "mesh" || pr.property_key == "texture") {
                    return Some(pr.entity_id);
                } else {
                    return None;
                }
            }).collect();
        for entity_id in renderable_changed {
            self.renderer_remove(&entity_id);
            self.renderer_add(system, &entity_id);
        }
        for pr in prop_refs.iter().filter(|pr| pr.property_key == "transform") {
            let transform = match system.get_property_value(&pr.entity_id, "transform") {
                Ok(trans) => matrix::from_prop_node(&trans).unwrap(),
                Err(err) => Matrix4::identity()
            };
            self.renderer.set_transform(&pr.entity_id, transform);
        }
    }

    fn update(&mut self, system: &mut ISystem, delta_time: time::Duration) {
        self.fps_counter.add_frame(delta_time);
        self.window.set_title(format!("pyramid {:.0} fps", self.fps_counter.fps()).as_str());

        let meshes_to_resolve = mem::replace(&mut self.meshes_to_resolve, Vec::new());
        self.meshes_to_resolve = meshes_to_resolve.into_iter().filter(|m| !m.try_resolve()).collect();
        let textures_to_resolve = mem::replace(&mut self.textures_to_resolve, Vec::new());
        self.textures_to_resolve = textures_to_resolve.into_iter().filter(|m| !m.try_resolve()).collect();


        unsafe {
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        self.renderer.render();
        self.window.swap_buffers();

        for event in self.window.poll_events() {
            match event {
                glutin::Event::Closed => {
                    system.exit();
                    return;
                },
                _ => ()
            }
        }
    }
}
