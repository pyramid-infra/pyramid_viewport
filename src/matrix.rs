extern crate cgmath;

use std::collections::HashMap;

use pyramid::pon::*;
use cgmath::*;

pub fn from_prop_node(node: &Pon) -> Result<Matrix4<f32>, PonTranslateErr> {
    let &TypedPon { ref type_name, ref data } = try!(node.as_transform());
    match type_name.as_str() {
        "matrix" => {
            let data = try!(data.as_float_array());
            return Ok(Matrix4::new(
                data[0], data[1], data[2], data[3],
                data[4], data[5], data[6], data[7],
                data[8], data[9], data[10], data[11],
                data[12], data[13], data[14], data[15]));
        },
        "translate" => {
            let data = try!(data.as_object());
            return Ok(Matrix4::from_translation(&(try!(to_vec3(data)))));
        },
        "rotate_x" => {
            let data: Rad<f32> = Rad { s: *try!(data.as_float()) };
            return Ok(Quaternion::from_angle_x(data).into());
        },
        "rotate_y" => {
            let data: Rad<f32> = Rad { s: *try!(data.as_float()) };
            return Ok(Quaternion::from_angle_y(data).into());
        },
        "rotate_z" => {
            let data: Rad<f32> = Rad { s: *try!(data.as_float()) };
            return Ok(Quaternion::from_angle_z(data).into());
        },
        "scale" => {
            let data = try!(data.as_object());
            let v = try!(to_vec3(data));
            let mat = Matrix4::new(v.x,  zero(), zero(), zero(),
                     zero(), v.y,  zero(), zero(),
                     zero(), zero(), v.z,  zero(),
                     zero(), zero(), zero(),  one());
            return Ok(mat);
        },
        "lookat" => {
            let data = try!(data.as_object());
            let eye = match data.get("eye") {
                Some(v) => try!(to_vec3(try!(v.as_object()))),
                None => return Err(PonTranslateErr::NoSuchField { field: "eye".to_string() })
            };
            let center = match data.get("center") {
                Some(v) => try!(to_vec3(try!(v.as_object()))),
                None => return Err(PonTranslateErr::NoSuchField { field: "center".to_string() })
            };
            let up = match data.get("up") {
                Some(v) => try!(to_vec3(try!(v.as_object()))),
                None => Vector3::new(0.0, 0.0, 1.0)
            };
            return Ok(Matrix4::look_at(&Point3::from_vec(&eye), &Point3::from_vec(&center), &up));
        },
        "projection" => {
            let data = try!(data.as_object());
            let fovy: f32 = match data.get("fovy") {
                Some(v) => *try!(v.as_float()),
                None => 1.0
            };
            let aspect: f32 = match data.get("aspect") {
                Some(v) => *try!(v.as_float()),
                None => 1.0
            };
            let near: f32 = match data.get("near") {
                Some(v) => *try!(v.as_float()),
                None => 0.1
            };
            let far: f32 = match data.get("far") {
                Some(v) => *try!(v.as_float()),
                None => 10.0
            };
            return Ok(perspective(Rad { s: fovy }, aspect, near, far));
        },
        "mul" => {
            let data = try!(data.as_array());
            let mut a = Matrix4::identity();
            for v in data {
                let b = try!(from_prop_node(v));
                a = a * b;
            }
            return Ok(a);
        },
        _ => Err(PonTranslateErr::UnrecognizedTypedPon(type_name.to_string()))
    }
}

fn to_vec3(map: &HashMap<String, Pon>) -> Result<Vector3<f32>, PonTranslateErr> {
    let x: f32 = match map.get("x") { Some(&ref v) => *try!(v.as_float()), _ => 0.0 };
    let y: f32 = match map.get("y") { Some(&ref v) => *try!(v.as_float()), _ => 0.0 };
    let z: f32 = match map.get("z") { Some(&ref v) => *try!(v.as_float()), _ => 0.0 };
    Ok(Vector3::new(x, y, z))
}
