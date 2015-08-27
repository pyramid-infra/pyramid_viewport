peg_file! legacy_directx_x_parse("legacy_directx_x.rustpeg");

extern crate image;

use image::RgbaImage;
use pyramid::propnode::*;

use std::path::Path;
use std::fmt;
use legacy_directx_x;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::*;


#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>
}

pub fn load_mesh(root_path: &Path, node: &PropNode) -> Result<Mesh, PropTranslateErr> {
    let &PropTransform { name: ref transform_name, ref arg } = try!(node.as_transform());

    match transform_name.as_str() {
        "static_mesh" => {
            let arg = try!(arg.as_object());
            let vertices = match arg.get("vertices") {
                Some(verts) => try!(verts.as_float_array()),
                None => return Err(PropTranslateErr::NoSuchField { field: "vertices".to_string() })
            };
            let indices = match arg.get("indices") {
                Some(verts) => try!(verts.as_integer_array()),
                None => return Err(PropTranslateErr::NoSuchField { field: "indices".to_string() })
            };
            return Ok(Mesh {
                vertices: vertices,
                indices: indices.iter().map(|x| *x as u32).collect()
            });
        },
        "mesh_from_file" => {
            let filename = try!(arg.as_string());
            let path_buff = root_path.join(Path::new(filename));
            let path = path_buff.as_path();
            println!("Loading mesh {:?}", path);
            let mut file = match File::open(&path) {
                Err(why) => panic!("couldn't open {}: {}", filename, Error::description(&why)),
                Ok(file) => file,
            };
            let mut content = String::new();
            return match file.read_to_string(&mut content) {
                Ok(_) => {
                    let dx = legacy_directx_x_parse::file(&content.as_str()).unwrap();
                    let mesh = dx.into_iter().find(|x| x.mesh.is_some()).unwrap().mesh.unwrap().to_mesh();
                    println!("Loaded mesh {}", filename);
                    return Ok(mesh);
                },
                Err(err) => Err(PropTranslateErr::Generic(format!("Failed to load mesh: {}: {:?}", filename, err)))
            }
        },
        _ => Err(PropTranslateErr::UnrecognizedPropTransform(transform_name.clone()))
    }
}

pub fn load_texture(root_path: &Path, node: &PropNode) -> Result<RgbaImage, PropTranslateErr> {
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
