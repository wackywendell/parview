/*!
# ParView
*/

#![crate_id = "parview#0.1"]
#![crate_type = "lib"]
#![deny(non_camel_case_types)]
#![deny(unnecessary_parens)]
#![deny(non_uppercase_statics)]
#![deny(unnecessary_qualification)]
#![deny(missing_doc)]
#![deny(unused_result)]
#![deny(unnecessary_typecast)]
#![deny(visible_private_types)]

extern crate serialize;

extern crate nalgebra;
extern crate kiss3d;

use serialize::{json, Encodable};
use std::rand::random;
use std::io;

use nalgebra::na;

#[deriving(Decodable, Encodable)]
/// A single frame, containing spheres
pub struct Frame {
    spheres : Vec<(na::Vec3<f32>, f32)>
}

/// A random Vec3<f32>, with coordinates in (-0.5, 0.5)
pub fn rand_vec() -> na::Vec3<f32> {
    na::Vec3::new(random(), random(), random()) - na::Vec3::new(0.5f32, 0.5f32, 0.5f32)
}

fn main() {
    let f = Frame {
        spheres : vec!(
            (rand_vec(), random()),
            (rand_vec(), random()),
            (rand_vec(), random()),
            (rand_vec(), random())
        )
    };
    
    let path = Path::new("test_frame.json");
    let mut file = BufferedReader::new(File::open(&path));
    {
        let mut encoder = json::Encoder::new(&mut m as &mut std::io::Writer);
        match f.encode(&mut encoder) {
            Ok(()) => (println!("Made object {}", m)),
            Err(e) => fail!("json encoding error: {}", e)
        };
    }
}
