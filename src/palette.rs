//! A palette for use with Objects

use std::hash;
use std::hash::{Hash,Hasher};
use std::iter::FromIterator;

use kiss3d::scene::SceneNode;

use objects::ObjectID;

/// An RGB color
#[derive(RustcDecodable, RustcEncodable,Eq,PartialEq,Ord,PartialOrd,Hash,Copy,Clone)]
pub struct Color(u8,u8,u8);

pub static DEFAULT_COLORS : [(u8, u8, u8); 11] = [
    (255, 255, 255), // White
    (  0,   0,   0), // Black
    (228,  26,  28), // Red
    ( 55, 126, 184), // Blue
    ( 77, 175,  74), // Green
    (152,  78, 163), // Purple
    (255, 127,   0), // Orange
    (166,  86,  40), // Brown
    (247, 129, 191), // Pink
    (153, 153, 153), // Gray
    (255, 255,  51), // Yellow
];

impl Color {
    fn to_floats(self) -> (f32, f32, f32) {
        let Color(r,g,b) = self;
        (
            (r as f32) / (u8::max_value() as f32),
            (g as f32) / (u8::max_value() as f32),
            (b as f32) / (u8::max_value() as f32),
        )
    }
}

/// A way to convert string names to colors
#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct Palette {
    default_colors : Vec<Color>,
    // TODO: have a way to map names → colors
    // HashMap<&str, Color>;
}

impl Palette {
    /// Set the color of an object to match what the palette says
    pub fn set_color(&self, name: &ObjectID, node: &mut SceneNode) {
        let (r, g, b) = self.get_default(name).to_floats();
        node.set_color(r, g, b);
    }
    
    /// Get the default color for an object with a given name. Based on hashing.
    pub fn get_default(&self, name: &ObjectID) -> Color {
        let mut hasher = hash::SipHasher::new();
        name.hash(&mut hasher);
        let hash = hasher.finish();
        let n = (hash as usize) % self.default_colors.len();
        self.default_colors[n]
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            default_colors : FromIterator::from_iter(DEFAULT_COLORS.iter().map(
                |&(r,g,b)|{Color(r,g,b)}
            ))
        }
    }
}
