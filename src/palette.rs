//! A palette for use with Objects

use std::collections::HashMap;
use std::iter::{repeat,FromIterator};

use kiss3d::scene::SceneNode;

use objects::ObjectID;

/// An RGB color
#[derive(RustcDecodable, RustcEncodable,Eq,PartialEq,Ord,PartialOrd,Hash,Copy,Clone)]
pub struct Color(u8,u8,u8);

pub static DEFAULT_COLORS : [(u8, u8, u8); 11] = [
    ( 77, 175,  74), // Green
    (152,  78, 163), // Purple
    (255, 127,   0), // Orange
    (228,  26,  28), // Red
    ( 55, 126, 184), // Blue
    (166,  86,  40), // Brown
    (247, 129, 191), // Pink
    (153, 153, 153), // Gray
    (255, 255,  51), // Yellow
    (255, 255, 255), // White
    (  0,   0,   0), // Black
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

/// A [bool] for keeping track of which part of the objectID should be used
#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct PartialIDer {
    /// Which sections of an ObjectID should be converted
    pub bools : Vec<bool>,
}

impl PartialIDer {
    /// New, with all set to given boolean value
    pub fn new(n : usize, value : bool) -> Self {
        PartialIDer {
            bools : repeat(value).take(n).collect(),
        }
    }
    
    /// Get the portion of an ID
    pub fn partial<'a>(&self, name: &'a ObjectID) -> Vec<&'a str> {
        let &ObjectID(ref names) = name;
        names.iter()
            .zip(self.bools.iter())
            .filter_map(|(n, &b)| if b {Some(&n[..])} else {None})
            .collect()
    }
    
    /// Get the portion of an ID
    pub fn as_id(&self, name: &ObjectID) -> ObjectID {
        let &ObjectID(ref names) = name;
        ObjectID(
            names
            .iter()
            .zip(self.bools.iter())
            .filter_map(|(n, &b)| if b {Some(n.clone())} else {None})
            .collect()
        )
    }
    
    /// Get a string representing the current state
    pub fn as_string(&self) -> String {
        let mut s = String::with_capacity(self.bools.len());
        let pieces : Vec<String> = self.bools.iter().enumerate().map(
            |(n, &b)| if b {format!("{}", (n+1) % 10)} else {String::from("_")}
        ).collect();
        s.extend(pieces.iter().map(|p| &p[0..1]));
        s
    }
}

/// A way to convert string names to colors
#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct Palette {
    default_colors : Vec<Color>,
    partials : PartialIDer,
    // TODO: have a way to map names → colors
    
    assigned : HashMap<ObjectID, Color>,
    next_color : usize
}

impl Palette {
    /// Set the color of an object to match what the palette says
    pub fn set_color(&mut self, name: &ObjectID, node: &mut SceneNode) {
        let (r, g, b) = self.get_color(name).to_floats();
        node.set_color(r, g, b);
    }
    
    /// Get the color for a particular ID (using partials mask)
    pub fn get_color(&mut self, name: &ObjectID) -> Color {
        let partial = self.partials.as_id(name);
        let (next_color, default_colors, assigned) = 
            (&mut self.next_color, &self.default_colors, &mut self.assigned);
        let color = *assigned.entry(partial).or_insert_with(|| {
            let col = default_colors[*next_color];
            *next_color = (*next_color + 1) % default_colors.len();
            col
        });
        return color
    }
    
    /// Toggle whether or not to use a certain partial value
    pub fn toggle_partial(&mut self, n : usize){
        self.partials.bools[n] = !self.partials.bools[n];
        self.next_color = 0;
    }
    
    /// Check value of a certain partial
    pub fn get_partial(&mut self, n : usize) -> bool {
        self.partials.bools[n]
    }
    
    /// Check value of a certain partial
    pub fn set_partial(&mut self, n : usize, value : bool){
        self.partials.bools[n] = value;
        self.next_color = 0;
    }
    
    /// Check value of a certain partial
    pub fn set_all_partial(&mut self, value : bool){
        let n = self.partials.bools.len();
        for i in 0..n {
            self.partials.bools[i] = value;
        }
        self.next_color = 0;
    }
    
    /// Get a string like '_23____' for which parts of the ObjectID we are coloring with respect to
    pub fn partials_string(&self) -> String {
        self.partials.as_string()
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            default_colors : FromIterator::from_iter(DEFAULT_COLORS.iter().map(
                |&(r,g,b)|{Color(r,g,b)}
            )),
            partials : PartialIDer::new(8, true),
            assigned : HashMap::new(),
            next_color : 0
        }
    }
}
