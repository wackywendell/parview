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

extern crate serialize;

extern crate "nalgebra" as na;
extern crate kiss3d;

use na::Indexable;

//use serialize::{json, Encodable};
use std::rand::random;
//use std::io;

#[deriving(Decodable, Encodable, Clone)]
/// A single frame, containing spheres
/// format is (location, radius, Option(color triple))
pub struct Sphere {
	/// location of the sphere
    pub loc : (f32,f32,f32),
    /// Radius
    pub radius : f32,
    /// Color. if none, one will be assigned
    pub color : Option<(u8,u8,u8)>,
}

impl Sphere {
	/// Make a new sphere
	pub fn new(loc : na::Vec3<f32>, radius : f32, color : Option<(u8,u8,u8)>) -> Sphere {
		Sphere {
			loc : (loc.at(0), loc.at(1), loc.at(2)),
			radius : radius,
			color : color
		}
	}
	
	/// get the location as a Vec3
	pub fn x(&self) -> na::Vec3<f32> {
		let (x,y,z) = self.loc;
		na::Vec3::new(x,y,z)
	}
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
