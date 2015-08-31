extern crate image;

use image::RgbaImage;
use pyramid::propnode::*;
use mesh::*;

use std::path::Path;
use std::fmt;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::*;

#[derive(Debug)]
pub struct ShaderSource {
    pub vertex_src: String,
    pub fragment_src: String,
}

fn propnode_to_layout(layout_node_array: &Vec<PropNode>) -> Result<Layout, PropTranslateErr> {
    let mut layout = vec![];
    for p in layout_node_array {
        let p = try!(p.as_array());
        layout.push((try!(p[0].as_string()).clone(), *try!(p[1].as_integer()) as usize));
    }
    Ok(Layout::new(layout))
}

pub fn propnode_to_mesh(root_path: &Path, node: &PropNode) -> Result<Mesh, PropTranslateErr> {
    let &PropTransform { name: ref transform_name, ref arg } = try!(node.as_transform());

    match transform_name.as_str() {
        "static_mesh" => {
            let layout_node_array = try!(try!(arg.get_object_field("layout")).as_array());
            let layout = try!(propnode_to_layout(layout_node_array));
            let vertices = try!(try!(arg.get_object_field("vertices")).as_float_array());
            let indices = try!(try!(arg.get_object_field("indices")).as_integer_array());

            return Ok(Mesh {
                layout: layout,
                vertex_data: vertices,
                element_data: indices.iter().map(|x| *x as u32).collect()
            });
        },
        "grid_mesh" => {
            let obj_arg = try!(arg.as_object());
            let mut grid = Grid::new();
            grid.layout = match obj_arg.get("layout") {
                Some(layout_node) => try!(propnode_to_layout(try!(layout_node.as_array()))),
                None => Layout::position_texcoord_normal()
            };
            grid.n_vertices_width = *try!(try!(arg.get_object_field("n_vertices_width")).as_integer()) as u32;
            grid.n_vertices_height = *try!(try!(arg.get_object_field("n_vertices_height")).as_integer()) as u32;

            return Ok(grid.into());
        },
        // "mesh_from_file" => {
        //     let config = match arg.as_string() {
        //         Ok(filename) => (filename.clone(), "polySurface1".to_string()),
        //         Err(err) => {
        //             match arg.as_object() {
        //                 Ok(arg) => {
        //                     (match arg.get("filename") {
        //                         Some(filename) => try!(filename.as_string()).clone(),
        //                         None => return Err(PropTranslateErr::NoSuchField { field: "filename".to_string() })
        //                     }, match arg.get("mesh_id") {
        //                         Some(mesh_id) => try!(mesh_id.as_string()).clone(),
        //                         None => "polySurface1".to_string()
        //                     })
        //                 },
        //                 Err(err) => return Err(err)
        //             }
        //         }
        //     };
        //     let path_buff = root_path.join(Path::new(&config.0));
        //     let path = path_buff.as_path();
        //     println!("Loading mesh {:?}", path);
        //     let mut file = match File::open(&path) {
        //         Err(why) => panic!("couldn't open {}: {}", config.0, Error::description(&why)),
        //         Ok(file) => file,
        //     };
        //     let mut content = String::new();
        //     return match file.read_to_string(&mut content) {
        //         Ok(_) => {
        //             let dx = match legacy_directx_x_parse::file(&content.as_str()) {
        //                 Ok(mesh) => mesh,
        //                 Err(err) => panic!("Failed to load mesh {:?} with error: {:?}", path, err)
        //             };
        //             let mesh = match dx.to_mesh(config.1) {
        //                 Ok(mesh) => mesh,
        //                 Err(err) => panic!("Failed to load mesh {:?} with error: {:?}", path, err)
        //             };
        //             println!("Loaded mesh {}", config.0);
        //             return Ok(mesh);
        //         },
        //         Err(err) => Err(PropTranslateErr::Generic(format!("Failed to load mesh: {}: {:?}", config.0, err)))
        //     }
        // },
        _ => Err(PropTranslateErr::UnrecognizedPropTransform(transform_name.clone()))
    }
}

pub fn propnode_to_texture(root_path: &Path, node: &PropNode) -> Result<RgbaImage, PropTranslateErr> {
    let &PropTransform { name: ref transform_name, ref arg } = try!(node.as_transform());

    match transform_name.as_str() {
        "static_texture" => {
            let arg = try!(arg.as_object());
            let pixel_data = match arg.get("pixels") {
                Some(verts) => try!(verts.as_integer_array()),
                None => return Err(PropTranslateErr::NoSuchField { field: "pixels".to_string() })
            };
            let pixel_data: Vec<u8> = pixel_data.iter().map(|x| *x as u8).collect();
            let width = match arg.get("width") {
                Some(verts) => *try!(verts.as_integer()) as u32,
                None => return Err(PropTranslateErr::NoSuchField { field: "width".to_string() })
            };
            let height = match arg.get("height") {
                Some(verts) => *try!(verts.as_integer()) as u32,
                None => return Err(PropTranslateErr::NoSuchField { field: "height".to_string() })
            };
            if width * height * 4 != pixel_data.len() as u32 {
                return Err(PropTranslateErr::Generic(format!("Expected {} pixels, found {}", width * height * 4, pixel_data.len())));
            }
            return match RgbaImage::from_raw(width, height, pixel_data) {
                Some(image) => Ok(image),
                None => Err(PropTranslateErr::Generic("Failed to create image in static_texture".to_string()))
            }
        },
        "texture_from_file" => {
            let filename = try!(arg.as_string());
            let path_buff = root_path.join(Path::new(filename));
            let path = path_buff.as_path();
            println!("Loading image {:?}", path);
            let img = image::open(&path);
            println!("Image loaded!");
            return match img {
                Ok(img) => Ok(img.to_rgba()),
                Err(err) => Err(PropTranslateErr::Generic(format!("Failed to load image: {}: {:?}", filename, err)))
            };
        },
        _ => Err(PropTranslateErr::UnrecognizedPropTransform(transform_name.clone()))
    }
}

pub fn propnode_to_shader(root_path: &Path, node: &PropNode) -> Result<ShaderSource, PropTranslateErr> {
    let &PropTransform { name: ref transform_name, ref arg } = try!(node.as_transform());

    match transform_name.as_str() {
        "shader_program" => {
            let vertex = try!(try!(arg.get_object_field("vertex")).as_transform());
            let fragment = try!(try!(arg.get_object_field("fragment")).as_transform());

            let vertex_string_arg = try!(vertex.arg.as_string()).clone();
            let vertex_src = match vertex.name.as_str() {
                "shader_from_file" => string_from_file(&root_path.join(Path::new(&vertex_string_arg))),
                "static_shader" => vertex_string_arg,
                _ => return Err(PropTranslateErr::UnrecognizedPropTransform(vertex.name.to_string()))
            };
            let fragment_string_arg = try!(fragment.arg.as_string()).clone();
            let fragment_src = match fragment.name.as_str() {
                "shader_from_file" => string_from_file(&root_path.join(Path::new(&fragment_string_arg))),
                "static_shader" => fragment_string_arg,
                _ => return Err(PropTranslateErr::UnrecognizedPropTransform(fragment.name.to_string()))
            };

            return Ok(ShaderSource {
                vertex_src: vertex_src,
                fragment_src: fragment_src
            })
        },
        _ => Err(PropTranslateErr::UnrecognizedPropTransform(transform_name.clone()))
    }
}

fn string_from_file(path: &Path) -> String {
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {:?}: {}", path, Error::description(&why)),
        Ok(file) => file,
    };
    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Ok(_) => content,
        Err(err) => panic!("Failed to read file {}", Error::description(&err))
    }
}
