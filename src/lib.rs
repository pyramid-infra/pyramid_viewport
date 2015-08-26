#![feature(plugin, box_patterns, rc_weak, convert, unboxed_closures, core)]
#![plugin(peg_syntax_ext)]

extern crate gl;
extern crate libc;
extern crate image;
extern crate cgmath;
extern crate time;
extern crate pyramid;
extern crate glutin;

mod renderer;
mod resources;
mod matrix;
mod legacy_directx_x;
mod legacy_directx_x_test;
mod promise;
mod gl_resources;

use pyramid::interface::*;
use pyramid::propnode::*;
use pyramid::document::*;
use pyramid::*;

use renderer::*;
use promise::*;
use gl_resources::*;
use resources::*;
use image::RgbaImage;
use std::collections::HashMap;
use std::collections::HashSet;
use cgmath::*;
use std::mem;

pub struct ViewportSubSystem {
    window: glutin::Window,
    renderer: Renderer,
    meshes_to_resolve: Vec<AsyncPromise<Mesh>>,
    textures_to_resolve: Vec<AsyncPromise<RgbaImage>>,
    meshes: HashMap<PropNode, Promise<GLMesh>>,
    textures: HashMap<PropNode, Promise<GLTexture>>,
    default_texture: PropNode,
}

impl ViewportSubSystem {
    pub fn new() -> ViewportSubSystem {
        let window = glutin::Window::new().unwrap();

        unsafe { window.make_current() };

        unsafe {
            gl::load_with(|symbol| window.get_proc_address(symbol));
            gl::ClearColor(1.0, 1.0, 0.0, 1.0);
        }

        ViewportSubSystem {
            window: window,
            renderer: Renderer::new(),
            meshes_to_resolve: vec![],
            textures_to_resolve: vec![],
            meshes: HashMap::new(),
            textures: HashMap::new(),
            default_texture: propnode_parser::parse("static_texture { pixels: [255, 0, 0, 255], width: 1, height: 1 }").unwrap(),
        }
    }
}

#[no_mangle]
pub fn new() -> Box<SubSystem> {
    Box::new(ViewportSubSystem::new())
}

impl ViewportSubSystem {

    fn renderer_add(&mut self, system: &System, entity_id: &EntityId) {
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
                let shader_program = self.renderer.shader_program.clone();
                let mesh_async_promise = AsyncPromise::new(move || load_mesh(&mesh_pn2).unwrap());
                let gl_mesh_promise = mesh_async_promise.promise.then(move |mesh| gl_resources::create_mesh(shader_program, mesh));
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
                let texture_async_promise = AsyncPromise::new(move || load_texture(&texture_pn2).unwrap());
                let gl_texture_promise = texture_async_promise.promise.then(move |texture| gl_resources::create_texture(texture.clone()));
                self.textures_to_resolve.push(texture_async_promise);
                self.textures.insert(texture_pn, gl_texture_promise.clone());
                gl_texture_promise
            }
        };
        let node = {
            RenderNode {
                id: *entity_id,
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

impl SubSystem for ViewportSubSystem {

    fn on_entity_added(&mut self, system: &mut System, entity_id: &EntityId) {
        let prop_refs: Vec<PropRef> = { system.get_properties(&entity_id).unwrap() };
        self.on_property_value_change(system, &prop_refs);
    }
    fn on_property_value_change(&mut self, system: &mut System, prop_refs: &Vec<PropRef>) {
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

    fn update(&mut self, system: &mut System, delta_time: time::Duration) {

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
