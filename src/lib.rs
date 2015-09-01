#![feature(box_patterns, rc_weak, convert, unboxed_closures, core)]

extern crate gl;
extern crate libc;
extern crate image;
extern crate cgmath;
extern crate time;
extern crate byteorder;
#[macro_use]
extern crate pyramid;
extern crate glutin;
extern crate mesh;

mod renderer;
mod resources;
mod matrix;
mod gl_resources;
mod fps_counter;
mod propnode_to_resource;

use pyramid::interface::*;
use pyramid::propnode::*;
use pyramid::document::*;
use pyramid::*;

use mesh::*;

use renderer::*;
use gl_resources::*;
use resources::*;
use fps_counter::*;
use propnode_to_resource::*;

use image::RgbaImage;
use std::collections::HashMap;
use std::collections::HashSet;
use cgmath::*;
use std::mem;
use gl::types::*;
use std::str;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

static SHADER_BASIC_VS: &'static [u8] = include_bytes!("../shaders/basic_vs.glsl");
static SHADER_BASIC_FS: &'static [u8] = include_bytes!("../shaders/basic_fs.glsl");


pub struct ViewportSubSystem {
    root_path: PathBuf,
    window: glutin::Window,
    renderer: Renderer,
    resources: Resources,
    default_textures: PropNode,
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
            root_path: root_path.clone(),
            window: window,
            renderer: Renderer::new(),
            resources: Resources::new(root_path.clone()),
            default_textures: propnode_parser::parse("{ diffuse: static_texture { pixels: [255, 0, 0, 255], width: 1, height: 1 } }").unwrap(),
            fps_counter: FpsCounter::new()
        };

        let shader_program = GLShaderProgram::new(
            &GLShader::new(str::from_utf8(SHADER_BASIC_VS).unwrap(), gl::VERTEX_SHADER),
            &GLShader::new(str::from_utf8(SHADER_BASIC_FS).unwrap(), gl::FRAGMENT_SHADER));

        viewport.resources.gl_shader_programs.borrow_mut().set(&PropNode::String("basic".to_string()), Rc::new(shader_program));

        viewport
    }
}

impl ViewportSubSystem {

    fn renderer_add(&mut self, system: &ISystem, entity_id: &EntityId) {
        let shader_key: PropNode = match system.get_property_value(entity_id, "shader") {
            Ok(shader) => shader,
            Err(err) => PropNode::String("basic".to_string())
        };
        let mesh_key: PropNode = match system.get_property_value(entity_id, "mesh") {
            Ok(mesh) => mesh,
            Err(err) => return ()
        };
        let texture_keys: PropNode = match system.get_property_value(entity_id, "textures") {
            Ok(textures) => textures,
            Err(err) => {
                match system.get_property_value(entity_id, "diffuse") {
                    Ok(diffuse) => PropNode::Object(hashmap![
                        "diffuse".to_string() => diffuse
                    ]),
                    Err(_) => return()
                }
            }
        };

        let gl_shader = self.resources.gl_shader_programs.borrow_mut().get(&shader_key);
        let gl_vertex_array = self.resources.gl_vertex_arrays.borrow_mut().get(&PropNode::Array(vec![shader_key, mesh_key]));
        let mut gl_textures = vec![];
        for (name, texture_key) in texture_keys.as_object().unwrap() {
            let gl_texture = self.resources.gl_textures.borrow_mut().get(texture_key);
            gl_textures.push((name.clone(), gl_texture));
        }

        let render_node = RenderNode {
            id: entity_id.clone(),
            shader: gl_shader,
            vertex_array: gl_vertex_array,
            textures: gl_textures,
            transform: match system.get_property_value(&entity_id, "transform") {
                Ok(trans) => matrix::from_prop_node(&trans).unwrap(),
                Err(err) => Matrix4::identity()
            }
        };
        self.renderer.add_node(render_node);
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
                if (pr.property_key == "mesh" || pr.property_key == "diffuse") {
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
        for pr in prop_refs.iter().filter(|pr| pr.property_key == "camera") {
            let camera = match system.get_property_value(&pr.entity_id, "camera") {
                Ok(trans) => matrix::from_prop_node(&trans).unwrap(),
                Err(err) => Matrix4::identity()
            };
            self.renderer.camera = camera;
        }
    }

    fn update(&mut self, system: &mut ISystem, delta_time: time::Duration) {
        self.fps_counter.add_frame(delta_time);
        self.window.set_title(format!("pyramid {:.0} fps", self.fps_counter.fps()).as_str());

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
