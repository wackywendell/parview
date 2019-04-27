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

extern crate serde;
extern crate rand;
extern crate flate2;
extern crate toml;

extern crate nalgebra as na;
extern crate kiss3d;

//use na::{Indexable,Iterable};

pub use serde::{Serialize, Deserialize};
//use std::io;

pub mod objects;
pub mod palette;
pub mod misc;
pub mod timer;
pub mod parviewer;
pub mod config;

pub use objects::{ObjectTracker, ObjectID, Sphere, Frame, EPSILON};
pub use palette::{Color, Palette};
pub use timer::Timer;
pub use parviewer::{Config, Parviewer};
pub use config::TomlConfig;
