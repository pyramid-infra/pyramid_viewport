extern crate cgmath;

use std::collections::HashMap;

use pyramid::propnode::*;
use cgmath::*;

pub fn from_prop_node(node: &PropNode) -> Result<Matrix4<f32>, PropTranslateErr> {
    let &PropTransform { ref name, ref arg } = try!(node.as_transform());
    match name.as_str() {
        "matrix" => {
            let arg = try!(arg.as_float_array());
            return Ok(Matrix4::new(
                arg[0], arg[1], arg[2], arg[3],
                arg[4], arg[5], arg[6], arg[7],
                arg[8], arg[9], arg[10], arg[11],
                arg[12], arg[13], arg[14], arg[15]));
        },
        "translate" => {
            let arg = try!(arg.as_object());
            return Ok(Matrix4::from_translation(&(try!(to_vec3(arg)))));
        },
        "rotate_x" => {
            let arg: Rad<f32> = Rad { s: *try!(arg.as_float()) };
            return Ok(Quaternion::from_angle_x(arg).into());
        },
        "rotate_y" => {
            let arg: Rad<f32> = Rad { s: *try!(arg.as_float()) };
            return Ok(Quaternion::from_angle_y(arg).into());
        },
        "rotate_z" => {
            let arg: Rad<f32> = Rad { s: *try!(arg.as_float()) };
            return Ok(Quaternion::from_angle_z(arg).into());
        },
        "scale" => {
            let arg = try!(arg.as_object());
            let v = try!(to_vec3(arg));
            let mat = Matrix4::new(v.x,  zero(), zero(), zero(),
                     zero(), v.y,  zero(), zero(),
                     zero(), zero(), v.z,  zero(),
                     zero(), zero(), zero(),  one());
            return Ok(mat);
        },
        "lookat" => {
            let arg = try!(arg.as_object());
            let eye = match arg.get("eye") {
                Some(v) => try!(to_vec3(try!(v.as_object()))),
                None => return Err(PropTranslateErr::NoSuchField { field: "eye".to_string() })
            };
            let center = match arg.get("center") {
                Some(v) => try!(to_vec3(try!(v.as_object()))),
                None => return Err(PropTranslateErr::NoSuchField { field: "center".to_string() })
            };
            let up = match arg.get("up") {
                Some(v) => try!(to_vec3(try!(v.as_object()))),
                None => Vector3::new(0.0, 0.0, 1.0)
            };
            return Ok(Matrix4::look_at(&Point3::from_vec(&eye), &Point3::from_vec(&center), &up));
        },
        "projection" => {
            let arg = try!(arg.as_object());
            let fovy: f32 = match arg.get("fovy") {
                Some(v) => *try!(v.as_float()),
                None => 1.0
            };
            let aspect: f32 = match arg.get("aspect") {
                Some(v) => *try!(v.as_float()),
                None => 1.0
            };
            let near: f32 = match arg.get("near") {
                Some(v) => *try!(v.as_float()),
                None => 0.1
            };
            let far: f32 = match arg.get("far") {
                Some(v) => *try!(v.as_float()),
                None => 10.0
            };
            return Ok(perspective(Rad { s: fovy }, aspect, near, far));
        },
        "mul" => {
            let arg = try!(arg.as_array());
            let mut a = Matrix4::identity();
            for v in arg {
                let b = try!(from_prop_node(v));
                a = a * b;
            }
            return Ok(a);
        },
        _ => Err(PropTranslateErr::UnrecognizedPropTransform(name.to_string()))
    }
}

fn to_vec3(map: &HashMap<String, PropNode>) -> Result<Vector3<f32>, PropTranslateErr> {
    let x: f32 = match map.get("x") { Some(&ref v) => *try!(v.as_float()), _ => 0.0 };
    let y: f32 = match map.get("y") { Some(&ref v) => *try!(v.as_float()), _ => 0.0 };
    let z: f32 = match map.get("z") { Some(&ref v) => *try!(v.as_float()), _ => 0.0 };
    Ok(Vector3::new(x, y, z))
}
