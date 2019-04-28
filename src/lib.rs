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

extern crate flate2;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate toml;

extern crate kiss3d;
extern crate nalgebra as na;

//use na::{Indexable,Iterable};

pub use serde::{Deserialize, Serialize};
//use std::io;

pub mod config;
pub mod misc;
pub mod objects;
pub mod palette;
pub mod parviewer;
pub mod timer;

pub use config::TomlConfig;
pub use objects::{Frame, ObjectID, ObjectTracker, Sphere, EPSILON};
pub use palette::{Color, Palette};
pub use parviewer::{Config, Parviewer};
pub use timer::Timer;
