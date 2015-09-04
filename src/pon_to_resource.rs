extern crate image;

use image::RgbaImage;
use pyramid::pon::*;
use mesh::*;

use std::path::Path;
use std::fmt;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::*;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug)]
pub struct ShaderSource {
    pub vertex_src: String,
    pub fragment_src: String,
}

#[derive(Clone)]
pub enum Texture {
    Image(RgbaImage),
    Floats {
        width: u32,
        height: u32,
        data: Vec<f32>
    }
}

fn pon_to_layout(layout_node_array: &Vec<Pon>) -> Result<Layout, PonTranslateErr> {
    let mut layout = vec![];
    for p in layout_node_array {
        let p = try!(p.as_array());
        layout.push((try!(p[0].as_string()).clone(), *try!(p[1].as_integer()) as usize));
    }
    Ok(Layout::new(layout))
}

pub fn pon_to_mesh(root_path: &Path, node: &Pon) -> Result<Mesh, PonTranslateErr> {
    let &TypedPon { type_name: ref type_name, ref data } = try!(node.as_transform());

    match type_name.as_str() {
        "static_mesh" => {
            let layout_node_array = try!(try!(data.get_object_field("layout")).as_array());
            let layout = try!(pon_to_layout(layout_node_array));
            let vertices = try!(try!(data.get_object_field("vertices")).as_float_array()).into_owned();
            let indices = try!(try!(data.get_object_field("indices")).as_integer_array()).into_owned();

            return Ok(Mesh {
                layout: layout,
                vertex_data: vertices,
                element_data: indices.iter().map(|x| *x as u32).collect()
            });
        },
        "grid_mesh" => {
            let obj_arg = try!(data.as_object());
            let mut grid = Grid::new();
            grid.layout = match obj_arg.get("layout") {
                Some(layout_node) => try!(pon_to_layout(try!(layout_node.as_array()))),
                None => Layout::position_texcoord_normal()
            };
            grid.n_vertices_width = *try!(try!(data.get_object_field("n_vertices_width")).as_integer()) as u32;
            grid.n_vertices_height = *try!(try!(data.get_object_field("n_vertices_height")).as_integer()) as u32;

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
        //                         None => return Err(PonTranslateErr::NoSuchField { field: "filename".to_string() })
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
        //         Err(err) => Err(PonTranslateErr::Generic(format!("Failed to load mesh: {}: {:?}", config.0, err)))
        //     }
        // },
        _ => Err(PonTranslateErr::UnrecognizedTypedPon(type_name.clone()))
    }
}

pub fn pon_to_texture(root_path: &Path, node: &Pon) -> Result<Texture, PonTranslateErr> {
    let &TypedPon { ref type_name, ref data } = try!(node.as_transform());

    match type_name.as_str() {
        "static_texture" => {
            let data = try!(data.as_object());
            let pixel_data = match data.get("pixels") {
                Some(verts) => try!(verts.as_integer_array()),
                None => return Err(PonTranslateErr::NoSuchField { field: "pixels".to_string() })
            };
            let pixel_data: Vec<u8> = pixel_data.iter().map(|x| *x as u8).collect();
            let width = match data.get("width") {
                Some(verts) => *try!(verts.as_integer()) as u32,
                None => return Err(PonTranslateErr::NoSuchField { field: "width".to_string() })
            };
            let height = match data.get("height") {
                Some(verts) => *try!(verts.as_integer()) as u32,
                None => return Err(PonTranslateErr::NoSuchField { field: "height".to_string() })
            };
            if width * height * 4 != pixel_data.len() as u32 {
                return Err(PonTranslateErr::Generic(format!("Expected {} pixels, found {}", width * height * 4, pixel_data.len())));
            }
            return match RgbaImage::from_raw(width, height, pixel_data) {
                Some(image) => Ok(Texture::Image(image)),
                None => Err(PonTranslateErr::Generic("Failed to create image in static_texture".to_string()))
            }
        },
        "texture_from_file" => {
            let filename = try!(data.as_string());
            let path_buff = root_path.join(Path::new(filename));
            let path = path_buff.as_path();
            println!("Loading image {:?}", path);
            if path.extension().unwrap().to_str().unwrap() == "dhm" {
                let mut f = File::open(path).unwrap();
                let mut data = vec![];
                f.read_to_end(&mut data);
                let mut rdr = Cursor::new(data);
                let width = rdr.read_i32::<LittleEndian>().unwrap() as u32;
                let height = rdr.read_i32::<LittleEndian>().unwrap() as u32;
                println!("SIZE {}, {}", width, height);
                let mut data = vec![];
                for y in 0..height {
                    for x in 0..width {
                        data.push(rdr.read_f32::<LittleEndian>().unwrap());
                    }
                }
                return Ok(Texture::Floats { width: width, height: height, data: data })
            } else {
                let img = image::open(&path);
                println!("Image loaded!");
                return match img {
                    Ok(img) => Ok(Texture::Image(img.to_rgba())),
                    Err(err) => Err(PonTranslateErr::Generic(format!("Failed to load image: {}: {:?}", filename, err)))
                };
            }
        },
        _ => Err(PonTranslateErr::UnrecognizedTypedPon(type_name.clone()))
    }
}

pub fn pon_to_shader(root_path: &Path, node: &Pon) -> Result<ShaderSource, PonTranslateErr> {
    let &TypedPon { ref type_name, ref data } = try!(node.as_transform());

    match type_name.as_str() {
        "shader_program" => {
            let vertex = try!(try!(data.get_object_field("vertex")).as_transform());
            let fragment = try!(try!(data.get_object_field("fragment")).as_transform());

            let vertex_string_arg = try!(vertex.data.as_string()).clone();
            let vertex_src = match vertex.type_name.as_str() {
                "shader_from_file" => string_from_file(&root_path.join(Path::new(&vertex_string_arg))),
                "static_shader" => vertex_string_arg,
                _ => return Err(PonTranslateErr::UnrecognizedTypedPon(vertex.type_name.to_string()))
            };
            let fragment_string_arg = try!(fragment.data.as_string()).clone();
            let fragment_src = match fragment.type_name.as_str() {
                "shader_from_file" => string_from_file(&root_path.join(Path::new(&fragment_string_arg))),
                "static_shader" => fragment_string_arg,
                _ => return Err(PonTranslateErr::UnrecognizedTypedPon(fragment.type_name.to_string()))
            };

            return Ok(ShaderSource {
                vertex_src: vertex_src,
                fragment_src: fragment_src
            })
        },
        _ => Err(PonTranslateErr::UnrecognizedTypedPon(type_name.clone()))
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
