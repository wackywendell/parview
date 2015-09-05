//! A palette for use with Objects

use serde;
use rustc_serialize; // needed for toml
use na;

#[cfg(test)]
use toml;
#[cfg(test)]
use serde_json;

use std::error::Error;
use std::io::Read;

use std::collections::HashMap;
use std::iter::{repeat, FromIterator};

use kiss3d::scene::SceneNode;

use objects::ObjectID;

/// An RGB color
#[derive(Eq,PartialEq,Ord,PartialOrd,Hash,Copy,Clone,Debug)]
pub struct Color(pub u8, pub u8, pub u8);

/// Default colors to use, when no others are specified.
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
    /// Convert to a 3-tuple, from 0 to 1
    pub fn to_floats(self) -> (f32, f32, f32) {
        let Color(r,g,b) = self;
        ((r as f32) / (u8::max_value() as f32),
         (g as f32) / (u8::max_value() as f32),
         (b as f32) / (u8::max_value() as f32))
    }

    /// Convert to `na::Pnt3`, needed for `kiss3d::window::Window::draw_text`
    pub fn to_pnt3(self) -> na::Pnt3<f32> {
        let (r,g,b) = self.to_floats();

        na::Pnt3::new(r, g, b)
    }
}

/// A [bool] for keeping track of which part of the objectID should be used
#[derive(Debug,Eq,PartialEq,Ord,PartialOrd,Hash,Clone)]
#[derive(Serialize,Deserialize)]
#[derive(RustcDecodable,RustcEncodable)]
pub struct PartialIDer {
    /// Which sections of an ObjectID should be converted
    pub bools: Vec<bool>,
}

impl PartialIDer {
    /// New, with all set to given boolean value
    pub fn new(n: usize, value: bool) -> Self {
        PartialIDer { bools: repeat(value).take(n).collect() }
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
        ObjectID(names
            .iter()
            .zip(self.bools.iter())
            .filter_map(|(n, &b)| if b {Some(n.clone())} else {None})
            .collect())
    }

    /// Get a string representing the current state
    pub fn as_string(&self) -> String {
        let mut s = String::with_capacity(self.bools.len());
        let pieces: Vec<String> = self.bools.iter().enumerate().map(
            |(n, &b)| if b {format!("{}", (n+1) % 10)} else {String::from("_")}
        ).collect();
        s.extend(pieces.iter().map(|p| &p[0..1]));
        s
    }
}

/// A way to convert string names to colors
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette {
    default_colors: Vec<Color>,
    partials: PartialIDer,

    /// Mapping of names -> colors, when found. names not found will be given default colors.
    pub assigned: HashMap<ObjectID, Color>,
    next_color: usize,
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
        let (next_color, default_colors, assigned) = (&mut self.next_color,
                                                      &self.default_colors,
                                                      &mut self.assigned);

        *assigned.entry(partial).or_insert_with(|| {
            let col = default_colors[*next_color];
            *next_color = (*next_color + 1) % default_colors.len();
            col
        })
    }

    /// Toggle whether or not to use a certain partial value
    pub fn toggle_partial(&mut self, n: usize) {
        if n > self.partials.bools.len() {
            return;
        };
        self.partials.bools[n] = !self.partials.bools[n];
        self.next_color = 0;
    }

    /// Check value of a certain partial
    pub fn get_partial(&mut self, n: usize) -> bool {
        self.partials.bools[n]
    }

    /// Check value of a certain partial
    pub fn set_partial(&mut self, n: usize, value: bool) {
        self.partials.bools[n] = value;
        self.next_color = 0;
    }

    /// Check value of a certain partial
    pub fn set_all_partial(&mut self, value: bool) {
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
            default_colors: FromIterator::from_iter(DEFAULT_COLORS.iter().map(
                |&(r,g,b)|{Color(r,g,b)}
            )),
            partials: PartialIDer::new(8, true),
            assigned: HashMap::new(),
            next_color: 0,
        }
    }
}

#[derive(Serialize, RustcEncodable, Copy, Clone)]
struct AssignmentRef<'a> {
    names: &'a ObjectID,
    color: &'a Color,
}

fn to_assignments(assigned: &HashMap<ObjectID, Color>) -> Vec<AssignmentRef> {
    assigned.into_iter().map(|(k, v)| {
        AssignmentRef{
            names : k,
            color : v
        }
    }).collect()
}

/// A reference to a palette, with minimal data, for serialization purposes
#[derive(Serialize, RustcEncodable, Clone)]
pub struct PaletteRef<'a> {
    default_colors: &'a Vec<Color>,
    partials: &'a PartialIDer,
    assigned: Vec<AssignmentRef<'a>>,
}

impl<'a> From<&'a Palette> for PaletteRef<'a> {
    fn from(palette: &'a Palette) -> Self {
        PaletteRef {
            default_colors: &palette.default_colors,
            partials: &palette.partials,
            assigned: to_assignments(&palette.assigned),
        }
    }
}

#[derive(Deserialize, RustcDecodable, Clone)]
struct Assignment {
    names: ObjectID,
    color: Color,
}

fn from_assignments(assigned: Vec<Assignment>) -> HashMap<ObjectID, Color> {
    assigned.into_iter().map(|a| {
        (a.names, a.color)
    }).collect()
}


/// A palette with all fields optional, for deserializing a config.
#[derive(Deserialize, RustcDecodable, Clone)]
struct PaletteOpt {
    default_colors: Option<Vec<Color>>,
    partials: Option<PartialIDer>,
    assigned: Option<Vec<Assignment>>,
    next_color: Option<usize>,
}

impl PaletteOpt {
    fn into_palette(self) -> Palette {
        let default_palette = Palette::default();
        
        Palette {
            default_colors: self.default_colors.unwrap_or(default_palette.default_colors),
            partials: self.partials.unwrap_or(default_palette.partials),
            assigned: self.assigned.map(from_assignments).unwrap_or(default_palette.assigned),
            next_color: self.next_color.unwrap_or(default_palette.next_color),
        }
    }
}

impl serde::Serialize for Color {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        let &Color(r,g,b) = self;
        serde::Serialize::serialize(&(r,g,b), serializer)
    }
}

impl serde::Deserialize for Color {
    fn deserialize<D: serde::Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
        let (r,g,b) = try!(serde::Deserialize::deserialize(deserializer));
        Ok(Color(r, g, b))
    }
}

impl rustc_serialize::Encodable for Color {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        let &Color(r,g,b) = self;
        rustc_serialize::Encodable::encode(&(r,g,b), s)
    }
}

impl rustc_serialize::Decodable for Color {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Self, D::Error> {
        let (r,g,b) = try!(rustc_serialize::Decodable::decode(d));
        Ok(Color(r, g, b))
    }
}

impl serde::Serialize for Palette {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        PaletteRef::from(self).serialize(serializer)
    }
}

impl serde::Deserialize for Palette {
    fn deserialize<D: serde::Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
        let p = try!(PaletteOpt::deserialize(deserializer));
        Ok(p.into_palette())
    }
}

impl rustc_serialize::Encodable for Palette {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        PaletteRef::from(self).encode(s)
    }
}

impl rustc_serialize::Decodable for Palette {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Self, D::Error> {
        let p = try!(PaletteOpt::decode(d));
        Ok(p.into_palette())
    }
}

#[test]
fn palette_toml_round_trip() {
    let p = Palette::default();

    let s = toml::encode_str(&p);
    let p2 = toml::decode_str(&*s).unwrap();
    assert_eq!(p, p2);
}

#[test]
fn palette_toml_empty_string() {
    let p = Palette::default();
    let p2 = toml::decode_str("").unwrap();
    assert_eq!(p, p2);
}

#[test]
fn palette_json_round_trip() {
    let p = Palette::default();

    let s = serde_json::to_string_pretty(&p).unwrap();
    let p2 = serde_json::from_str(&*s).unwrap();
    assert_eq!(p, p2);
}
