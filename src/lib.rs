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
    pub spheres : Vec<Sphere>,
    /// Text to display
    pub text : Option<String>
}

/// A random Vec3<f32>, with coordinates in (-0.5, 0.5)
pub fn rand_vec() -> na::Vec3<f32> {
    na::Vec3::new(random(), random(), random()) - na::Vec3::new(0.5f32, 0.5f32, 0.5f32)
}

/// Tracks the drawn sphere objects
pub struct SphereNodes {
	spheres : Vec<kiss3d::scene::SceneNode>
}

/// Default colors for the spheres
static DEFAULT_COLORS : [(f32, f32, f32), ..11] = [
			(1.,1.,1.),
			(0.,0.,0.),
			(0.8941, 0.1020, 0.1098),
			(0.2157, 0.4941, 0.7216),
			(0.3020, 0.6863, 0.2902),
			(0.5961, 0.3059, 0.6392),
			(1.0000, 0.4980, 0.0000),
			(0.6510, 0.3373, 0.1569),
			(0.9686, 0.5059, 0.7490),
			(0.6000, 0.6000, 0.6000),
			(1.0000, 1.0000, 0.2000)];

impl SphereNodes {
	/// New set of spheres
	pub fn new<'a, T : Iterator<&'a Sphere>>(sphere_iter : T, window : &mut kiss3d::window::Window) -> SphereNodes {
		let sphere_set = vec!();
		let mut sn = SphereNodes { spheres : sphere_set };
		sn.update(sphere_iter, window);
		sn
	}
	
	/// Update the drawn spheres to match the data
	pub fn update<'a, T : Iterator<&'a Sphere>>(&mut self, sphere_iter : T, window : &mut kiss3d::window::Window) {
		let mut maxn = 0;
		for (n, &sphere) in sphere_iter.enumerate() {
			match n >= self.spheres.len() {
				false => {}
				true => {
					// new sphere!
					let news = window.add_sphere(sphere.radius);
					self.spheres.push(news);
				}
			};
			
			let (r,g,b) = match sphere.color {
				Some((r,g,b)) => (r as f32 / 255., g as f32 / 255., b as f32 / 255.),
				None => DEFAULT_COLORS[n % DEFAULT_COLORS.len()]
			};
			
			let s : &mut kiss3d::scene::SceneNode = self.spheres.get_mut(n);
			s.set_color(r,g,b);
			s.set_local_translation(sphere.x());
			s.set_local_scale(sphere.radius, sphere.radius, sphere.radius);
			
			maxn = n;
		}
		
		for s in self.spheres.slice_from_mut(maxn).iter_mut() {
			s.unlink();
		}
		
		self.spheres.truncate(maxn);
	}
}

///
pub struct Timer {
	dts : Vec<f32>, // possible dt values
	/// DEBUG
	pub dti : int, // which of dts we're talking about. 0 is stop, 1 => dts[0], -1 => -dts[0]
	len : Option<uint>, // length of what we're iterating over
	/// DEBUG
	pub t : f32, // current index
	//~ loop_around : enum {
		//~ Loop,
		//~ Stop,
		//~ Reverse
	//~ }
}

impl Timer {
	/// Make a new timer
	pub fn new(dts : Vec<f32>, len : Option<uint>) -> Timer {
		let new_dts = if dts.len() == 0 {
			vec!(1f32)
		} else {
			dts
		};
		
		Timer {
			dts: new_dts,
			dti: 1,
			len: len,
			t : 0.0,
		}
	}
	
	/// Switch forwards vs. backwards. If stopped, it stays stopped.
	pub fn switch_direction(&mut self) {
		self.dti = -self.dti;
	}
	
	/// Increment faster
	pub fn faster(&mut self) {
		self.dti = match self.dti {
			0 => 1,
			i if i >= self.dts.len() as int => self.dts.len() as int,
			i if i <= -(self.dts.len() as int) => -(self.dts.len() as int),
			i if i > 0 => i+1,
			i => i-1
		};
	}
	
	/// Increment slower
	pub fn slower(&mut self) {
		self.dti = match self.dti {
			0 => 0,
			i if i > 0 => i-1,
			i => i+1
		};
	}
	
	/// Get current dt
	pub fn get_dt(&self) -> f32{
		match self.dti {
			0 => 0.,
			i if i > 0 => self.dts[(i-1) as uint],
			i => -self.dts[(1-i) as uint]
		}
	}
	
	
	/// Increment the timer, and return current index
	pub fn incr(&mut self) -> uint {
		self.t += self.get_dt();
		
		let ix = self.t as uint;
		match self.len {
			None => {ix},
			Some(l) => ix % l
		}
	}
}
