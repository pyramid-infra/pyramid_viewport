extern crate image;

use image::RgbaImage;
use pyramid::pon::*;
use pyramid::document::*;
use mesh::*;
use cgmath::*;

use std::path::Path;
use std::path::PathBuf;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::borrow::Cow;
use std::rc::Rc;
use ppromise::*;
use std::mem;

#[derive(Debug)]
pub struct ShaderSource {
    pub vertex_src: String,
    pub vertex_debug_source_name: String,
    pub fragment_src: String,
    pub fragment_debug_source_name: String
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

pub struct LocalAttributeSpec(AttributeSpec);

impl Translatable<LocalAttributeSpec> for Pon {
    fn inner_translate(&self, context: &mut TranslateContext) -> Result<LocalAttributeSpec, PonTranslateErr> {
        self.as_array(|p| {
                Ok(LocalAttributeSpec(AttributeSpec(try!(p[0].translate::<String>(context)), try!(p[1].translate::<i64>(context)) as usize)))
        })
    }
}

pub fn pon_to_mesh(root_path: &Path, node: &Pon, context: &mut TranslateContext) -> Result<Rc<Mesh>, PonTranslateErr> {
    println!("Pon to mesh");
    node.as_typed(|&TypedPon { type_name: ref type_name, ref data }| {
        match type_name.as_str() {
            "static_mesh" => {
                let attribs = try!(data.field_as::<PonAutoVec<LocalAttributeSpec>>("layout", context)).0.into_iter().map(|x| x.0).collect();
                let layout = Layout::new(attribs);
                let vertices = try!(data.field_as::<Vec<f32>>("vertices", context));
                let indices = try!(data.field_as::<Vec<i64>>("indices", context));

                return Ok(Rc::new(Mesh {
                    layout: layout,
                    vertex_data: vertices,
                    element_data: indices.iter().map(|x| *x as u32).collect()
                }));
            },
            "mesh_from_resource" => {
                let resource_id = try!(data.translate::<String>(context));
                return Ok(context.document.unwrap().resources.get(&resource_id).unwrap().downcast_ref::<Rc<Mesh>>().unwrap().clone());
            },
            "grid_mesh" => {
                let mut grid = Grid::new();
                let layout = match data.field_as::<PonAutoVec<LocalAttributeSpec>>("layout", context) {
                    Ok(attrs) => {
                        let attribs = attrs.0.into_iter().map(|x| x.0).collect();
                        Layout::new(attribs)
                    },
                    Err(_) => Layout::position_texcoord_normal()
                };
                grid.layout = layout;
                grid.n_vertices_width = try!(data.field_as::<i64>("n_vertices_width", context)) as u32;
                grid.n_vertices_height = try!(data.field_as::<i64>("n_vertices_height", context)) as u32;

                return Ok(Rc::new(grid.into()));
            },
            "box_mesh" => {
                let mut box_mesh = Box3::new();
                let layout = match data.field_as::<PonAutoVec<LocalAttributeSpec>>("layout", context) {
                    Ok(attrs) => {
                        let attribs = attrs.0.into_iter().map(|x| x.0).collect();
                        Layout::new(attribs)
                    },
                    Err(_) => Layout::position_texcoord_normal()
                };
                box_mesh.layout = layout;
                box_mesh.position = try!(data.field_as_or::<Vector3<f32>>("position", Vector3::zero(), context));
                box_mesh.size = try!(data.field_as_or::<Vector3<f32>>("size", Vector3::one(), context));

                return Ok(Rc::new(box_mesh.into()));
            },
            _ => Err(PonTranslateErr::UnrecognizedType(type_name.clone()))
        }
    })
}

pub trait LoadableTexture {
    fn load(&mut self, async_runner: &mut AsyncRunner) -> Promise<Texture>;
}
struct StaticTexture {
    texture: Option<Texture>
}
impl LoadableTexture for StaticTexture {
    fn load(&mut self, async_runner: &mut AsyncRunner) -> Promise<Texture> {
        let texture = mem::replace(&mut self.texture, None);
        Promise::resolved(texture.unwrap())
    }
}
struct TextureFromFile {
    path: PathBuf
}
impl LoadableTexture for TextureFromFile {
    fn load(&mut self, async_runner: &mut AsyncRunner) -> Promise<Texture> {
        let path = self.path.clone();
        async_runner.exec_async(move || {
            println!("Loading image {:?}", path);
            if path.extension().unwrap().to_str().unwrap() == "dhm" {
                let mut f = File::open(&path).unwrap();
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
                return Texture::Floats { width: width, height: height, data: data }
            } else {
                let img = image::open(&path);
                println!("Image loaded!");
                return match img {
                    Ok(img) => Texture::Image(img.to_rgba()),
                    Err(err) => panic!("Failed to load image: {}: {:?}", path.to_str().unwrap(), err)
                };
            }
        })
    }
}

pub fn pon_to_texture(root_path: &Path, node: &Pon, context: &mut TranslateContext) -> Result<Box<LoadableTexture>, PonTranslateErr> {
    println!("Pon to texture");
    node.as_typed(|&TypedPon { ref type_name, ref data }| -> Result<Box<LoadableTexture>, PonTranslateErr> {
        match type_name.as_str() {
            "static_texture" => {
                let pixel_data = try!(data.field_as::<Vec<i64>>("pixels", context));
                let pixel_data: Vec<u8> = pixel_data.iter().map(|x| *x as u8).collect();
                let width = try!(data.field_as::<i64>("width", context)) as u32;
                let height = try!(data.field_as::<i64>("height", context)) as u32;
                if width * height * 4 != pixel_data.len() as u32 {
                    return Err(PonTranslateErr::Generic(format!("Expected {} pixels, found {}", width * height * 4, pixel_data.len())));
                }
                return match RgbaImage::from_raw(width, height, pixel_data) {
                    Some(image) => Ok(Box::new(StaticTexture { texture: Some(Texture::Image(image)) })),
                    None => Err(PonTranslateErr::Generic("Failed to create image in static_texture".to_string()))
                }
            },
            "texture_from_file" => {
                let filename = try!(data.translate::<String>(context));
                let path_buff = root_path.join(Path::new(&filename));
                Ok(Box::new(TextureFromFile { path: path_buff }))
            },
            _ => Err(PonTranslateErr::UnrecognizedType(type_name.clone()))
        }
    })
}

pub fn pon_to_shader(root_path: &Path, node: &Pon, context: &mut TranslateContext) -> Result<ShaderSource, PonTranslateErr> {
    println!("Pon to shader");
    node.as_typed(|&TypedPon { ref type_name, ref data }| {
        match type_name.as_str() {
            "shader_program" => {
                let vertex = (try!(data.field("vertex"))).as_typed(|&TypedPon { ref type_name, ref data }| {
                    let string_arg = data.translate::<String>(context).unwrap();
                    match type_name.as_str() {
                        "shader_from_file" => Ok(string_from_file(&root_path.join(Path::new(&string_arg)))),
                        "static_shader" => Ok(string_arg),
                        _ => return Err(PonTranslateErr::UnrecognizedType(type_name.to_string()))
                    }
                });
                let fragment = (try!(data.field("fragment"))).as_typed(|&TypedPon { ref type_name, ref data }| {
                    let string_arg = data.translate::<String>(context).unwrap();
                    match type_name.as_str() {
                        "shader_from_file" => Ok(string_from_file(&root_path.join(Path::new(&string_arg)))),
                        "static_shader" => Ok(string_arg),
                        _ => return Err(PonTranslateErr::UnrecognizedType(type_name.to_string()))
                    }
                });

                return Ok(ShaderSource {
                    vertex_src: try!(vertex),
                    vertex_debug_source_name: data.to_string(),
                    fragment_src: try!(fragment),
                    fragment_debug_source_name: data.to_string()
                })
            },
            _ => Err(PonTranslateErr::UnrecognizedType(type_name.clone()))
        }
    })
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
