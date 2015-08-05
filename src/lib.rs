/*!
# ParView
*/

#![crate_type = "lib"]
#![deny(non_camel_case_types)]
#![deny(unused_parens)]
#![deny(non_upper_case_globals)]
#![deny(unused_qualifications)]
#![deny(missing_docs)]
#![deny(unused_results)]

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate rand;
extern crate serde;

extern crate nalgebra as na;
extern crate kiss3d;

//use na::{Indexable,Iterable};

pub use serde::{Serialize, Deserialize};
use rand::random;
//use std::io;

pub mod objects;
pub use objects::{ObjectTracker,ObjectID,Sphere};
pub mod palette;
pub use palette::Palette;

#[derive(Deserialize, Serialize, Clone)]
/// A single frame, which is a series of spheres
pub struct Frame {
    /// the spheres
    pub spheres : Vec<Sphere>,
    /// Text to display
    pub text : Option<String>
}

/// A random Vec3<f32>, with coordinates in (-0.5, 0.5)
pub fn rand_vec() -> na::Vec3<f32> {
    na::Vec3::new(random::<f32>(), random(), random()) - na::Vec3::new(0.5f32, 0.5f32, 0.5f32)
}

/// Timer
pub struct Timer {
    dts : Vec<f32>, // possible dt values
    /// DEBUG
    pub dti : isize, // which of dts we're talking about. 0 is stop, 1 => dts[0], -1 => -dts[0]
    len : Option<usize>, // length of what we're iterating over
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
    pub fn new(dts : Vec<f32>, len : Option<usize>) -> Timer {
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
            i if i >= self.dts.len() as isize => self.dts.len() as isize,
            i if i <= -(self.dts.len() as isize) => -(self.dts.len() as isize),
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
            i if i > 0 => self.dts[(i-1) as usize],
            i => -self.dts[(1-i) as usize]
        }
    }


    /// Increment the timer, and return current index
    pub fn incr(&mut self) -> usize {
        self.t += self.get_dt();

        let ix = self.t as usize;
        match self.len {
            None => {ix},
            Some(l) => ix % l
        }
    }
}
