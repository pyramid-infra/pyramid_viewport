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
extern crate ppromise;

mod renderer;
mod resources;
mod gl_resources;
mod fps_counter;
mod pon_to_resource;
mod shader_uniforms;

use pyramid::interface::*;
use pyramid::pon::*;
use pyramid::document::*;
use pyramid::system::*;

use mesh::*;

use renderer::*;
use gl_resources::*;
use resources::*;
use fps_counter::*;
use pon_to_resource::*;
use shader_uniforms::*;

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
use time::*;
use ppromise::*;

static SHADER_BASIC_VS: &'static [u8] = include_bytes!("../shaders/basic_vs.glsl");
static SHADER_BASIC_FS: &'static [u8] = include_bytes!("../shaders/basic_fs.glsl");

struct PendingAdd {
    id: EntityId,
    resources: Promise<RenderNodeResources>,
    config: RenderNodeConfig
}

pub struct ViewportSubSystem {
    root_path: PathBuf,
    window: glutin::Window,
    renderer: Renderer,
    resources: Resources,
    pending_add: Vec<PendingAdd>,
    default_textures: Pon,
    fps_counter: FpsCounter,
    start_time: Timespec,
    prev_time: Timespec,
    first_load_timed: bool
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
            pending_add: vec![],
            default_textures: Pon::from_string("{ diffuse: static_texture { pixels: [255, 0, 0, 255], width: 1, height: 1 } }").unwrap(),
            fps_counter: FpsCounter::new(),
            start_time: time::get_time(),
            prev_time: time::get_time(),
            first_load_timed: false
        };

        let shader_program = GLShaderProgram::new(
            &GLShader::new(str::from_utf8(SHADER_BASIC_VS).unwrap(), gl::VERTEX_SHADER, "bundled"),
            &GLShader::new(str::from_utf8(SHADER_BASIC_FS).unwrap(), gl::FRAGMENT_SHADER, "bundled"));

        viewport.resources.gl_shader_programs.insert(Pon::String("basic".to_string()), Promise::resolved(Rc::new(shader_program)));

        viewport
    }
}

impl ViewportSubSystem {

    fn renderer_add(&mut self, document: &mut Document, entity_id: &EntityId) {
        let shader_key: Pon = match document.get_property_value(entity_id, "shader") {
            Ok(shader) => shader.clone(),
            Err(err) => Pon::String("basic".to_string())
        };
        let mesh_key: Pon = match document.get_property_value(entity_id, "mesh") {
            Ok(mesh) => mesh.clone(),
            Err(err) => return ()
        };
        let texture_keys: Pon = match document.get_property_value(entity_id, "textures") {
            Ok(textures) => textures.clone(),
            Err(err) => {
                match document.get_property_value(entity_id, "diffuse") {
                    Ok(diffuse) => Pon::Object(hashmap![
                        "diffuse".to_string() => diffuse.clone()
                    ]),
                    Err(_) => return()
                }
            }
        };

        let mut texture_keys_vec = vec![];
        let mut texture_ids = vec![];
        for (name, texture_key) in texture_keys.translate::<&HashMap<String, Pon>>(&mut TranslateContext::from_doc(document)).unwrap() {
            texture_ids.push(name.to_string());
            texture_keys_vec.push(texture_key.clone());
        }

        self.pending_add.push(PendingAdd {
            id: entity_id.clone(),
            resources: self.resources.get(document, &mesh_key, &shader_key, texture_keys_vec),
            config: RenderNodeConfig {
                texture_ids: texture_ids,
                transform: match document.get_property_value(&entity_id, "transformed") {
                    Ok(trans) => trans.translate(&mut TranslateContext::from_doc(document)).unwrap(),
                    Err(err) => Matrix4::identity()
                },
                uniforms: match document.get_property_value(&entity_id, "uniforms") {
                    Ok(uniforms) => uniforms.translate(&mut TranslateContext::from_doc(document)).unwrap(),
                    Err(err) => ShaderUniforms(vec![])
                },
                alpha: match document.get_property_value(&entity_id, "alpha") {
                    Ok(trans) => *trans.translate::<&bool>(&mut TranslateContext::from_doc(document)).unwrap(),
                    Err(err) => false
                }
            }
        });
    }
    fn renderer_remove(&mut self, entity_id: &EntityId) {
        self.renderer.remove_node(entity_id);
        self.pending_add.retain(|p| p.id != *entity_id);
    }
}

impl ISubSystem for ViewportSubSystem {

    fn on_property_value_change(&mut self, system: &mut System, prop_refs: &Vec<PropRef>) {
        let document = system.document_mut();
        //println!("CHANGED {:?}", prop_refs);
        let renderable_changed: HashSet<EntityId> = prop_refs.iter()
            .filter_map(|pr| {
                if pr.property_key == "mesh" || pr.property_key == "diffuse" || pr.property_key == "alpha" || pr.property_key == "uniforms" {
                    return Some(pr.entity_id);
                } else {
                    return None;
                }
            }).collect();
        for entity_id in renderable_changed {
            self.renderer_remove(&entity_id);
            self.renderer_add(document, &entity_id);
        }
        for pr in prop_refs.iter().filter(|pr| pr.property_key == "transformed") {
            let transform = match document.get_property_value(&pr.entity_id, "transformed") {
                Ok(trans) => trans.translate(&mut TranslateContext::from_doc(document)).unwrap(),
                Err(err) => Matrix4::identity()
            };
            self.renderer.set_transform(&pr.entity_id, transform);
        }
        for pr in prop_refs.iter().filter(|pr| pr.property_key == "camera") {
            let camera = match document.get_property_value(&pr.entity_id, "camera") {
                Ok(trans) => trans.translate(&mut TranslateContext::from_doc(document)).unwrap(),
                Err(err) => Matrix4::identity()
            };
            self.renderer.camera = camera;
        }
    }

    fn update(&mut self, system: &mut System) {
        let delta_time = time::get_time() - self.prev_time;
        self.prev_time = time::get_time();
        let total_time = time::get_time() - self.start_time;
        self.fps_counter.add_frame(delta_time);
        self.window.set_title(&format!("pyramid {}", self.fps_counter.to_string()));

        self.resources.update();

        let pending_adds = mem::replace(&mut self.pending_add, vec![]);
        let pending_adds_was_0 = pending_adds.len() == 0;
        self.pending_add = pending_adds.into_iter().filter_map(|pending_add| {
            let is_some = {
                pending_add.resources.value().is_some()
            };
            if is_some {
                self.renderer.add_node(RenderNode {
                    id: pending_add.id,
                    resources: pending_add.resources.into_value(),
                    config: pending_add.config
                });
                return None;
            } else {
                return Some(pending_add);
            }
        }).collect();
        if self.pending_add.len() == 0 && !pending_adds_was_0 && !self.first_load_timed {
            self.first_load_timed = true;
            println!("All entities added to renderer. {} ms", total_time.num_milliseconds());
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
