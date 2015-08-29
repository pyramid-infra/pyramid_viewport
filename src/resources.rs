extern crate image;

use image::RgbaImage;
use pyramid::propnode::*;

use std::path::Path;
use std::fmt;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::*;


#[derive(Debug, PartialEq)]
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
