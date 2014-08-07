/*!
# ParView
*/

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

//use serialize::{json, Encodable};
use std::rand::random;
//use std::io;

use nalgebra::na;

#[deriving(Decodable, Encodable, Clone)]
/// A single frame, containing spheres
/// format is (location, radius, Option(color triple))
pub struct Sphere {
	/// location of the sphere
    pub loc : na::Vec3<f32>,
    /// Radius
    pub radius : f32,
    /// Color. if none, one will be assigned
    pub color : Option<(u8,u8,u8)>,
}

#[deriving(Decodable, Encodable, Clone)]
/// A single frame, which is a series of spheres
pub struct Frame {
	/// the spheres
    pub spheres : Vec<Sphere>
}

/// A random Vec3<f32>, with coordinates in (-0.5, 0.5)
pub fn rand_vec() -> na::Vec3<f32> {
    na::Vec3::new(random(), random(), random()) - na::Vec3::new(0.5f32, 0.5f32, 0.5f32)
}
