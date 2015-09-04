//! The types of objects that can be drawn by ParView

extern crate nalgebra as na;
extern crate serde;
extern crate rustc_serialize;

//use rustc_serialize::json::{self, Json, ToJson};
use serde::{Serialize, Serializer, Deserialize, Deserializer, Error};
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use std::collections::{HashSet, HashMap};
use std::collections::hash_map::Entry;
use std::iter::FromIterator;
use std::convert::From;

use na::{RotationTo, Rotation, Norm};

use palette::Palette;

/// A minimal value that is close enough to 0 for visual purposes
pub const EPSILON : f32 = 1e-6;

/// The way to ID an object. This is basically a list of strings.
///
/// Each item in the string corresponds to a level in a hierarchy, so that groupings can be made.
/// For example, a protein might have levels:
/// ResidueName → ResidueNumber → Element → AtomName
/// Each must be unique.
#[derive(Debug,Eq,PartialEq,Ord,PartialOrd,Hash,Clone)]
pub struct ObjectID(pub Vec<String>);

impl Serialize for ObjectID {
    fn serialize<S: Serializer>(&self, s: &mut S) -> Result<(), S::Error> {
        let &ObjectID(ref names) = self;
        names.serialize(s)
    }
}

impl rustc_serialize::Encodable for ObjectID {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        let &ObjectID(ref names) = self;
        names.encode(s)
    }
}

impl Deserialize for ObjectID {
    fn deserialize<D: Deserializer>(d: &mut D) -> Result<Self, D::Error> {
        Vec::deserialize(d).map(ObjectID)
    }
}

impl rustc_serialize::Decodable for ObjectID {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Self, D::Error> {
        Vec::decode(d).map(ObjectID)
    }
}

impl ObjectID{
    /// Create a new `ObjectTracker` associated with a given `Window`.
    pub fn new(names: Vec<String>) -> ObjectID {
        ObjectID(names)
    }
}

#[derive(Deserialize, Clone)]
/// A single frame, which is a series of spheres
struct MinimalFrame {
    /// the spheres
    pub spheres: Option<Vec<Sphere>>,
    /// optional spherocylinders
    pub spherocylinders: Option<Vec<Spherocylinder>>,
    /// Text to display
    pub text: Option<String>,
}

#[derive(Serialize, Clone)]
/// A single frame, which is a series of spheres
pub struct Frame {
    /// the spheres
    pub spheres: Vec<Sphere>,
    /// optional spherocylinders
    pub spherocylinders: Vec<Spherocylinder>,
    /// Text to display
    pub text: String,
}

impl Deserialize for Frame {
    fn deserialize<D: Deserializer>(d: &mut D) -> Result<Self, D::Error> {
        let minim = try!(MinimalFrame::deserialize(d));
        Ok(Frame {
            spheres: minim.spheres.unwrap_or(vec![]),
            spherocylinders: minim.spherocylinders.unwrap_or(vec![]),
            text: minim.text.unwrap_or(String::new()),
        })
    }
}

/// An object that will be drawable by Parview.
///
/// This is the trait-based interface so that Parview can manage it.
/// See Sphere, etc. for objects to implement.
pub trait Object : Clone {
    /// The ID of an object. Must be unique.
    fn id(&self) -> &ObjectID;
    /// Create a new node for this object.
    fn new_node(&self, window: &mut SceneNode) -> SceneNode;
    /// If `self` is the "old" sphere, and `other` is the new one, then for each difference,
    /// Update `self` and the corresponding SceneNode to match
    fn update(&mut self, other: &Self, nodes: &mut SceneNode);
}

/// An enum of the possible shapes, for use with the ObjectTracker.
#[derive(Clone)]
pub enum ObjectEnum {
    /// A sphere
    Sphere(Sphere),
    /// a spherocylinder
    Spherocylinder(Spherocylinder),
}

impl Object for ObjectEnum {
    fn id(&self) -> &ObjectID {
        match *self {
            ObjectEnum::Sphere(ref s) => s.id(),
            ObjectEnum::Spherocylinder(ref s) => s.id(),
        }
    }

    fn new_node(&self, window: &mut SceneNode) -> SceneNode {
        match *self {
            ObjectEnum::Sphere(ref s) => s.new_node(window),
            ObjectEnum::Spherocylinder(ref s) => s.new_node(window),
        }
    }

    fn update(&mut self, other: &Self, nodes: &mut SceneNode) {
        match (self, other) {
            (&mut ObjectEnum::Sphere(ref mut s), &ObjectEnum::Sphere(ref o)) => s.update(o, nodes),
            (&mut ObjectEnum::Sphere(_), _) => {
                unimplemented!()
            }
            (&mut ObjectEnum::Spherocylinder(ref mut s), &ObjectEnum::Spherocylinder(ref o)) => {
                s.update(o, nodes)
            }
            (&mut ObjectEnum::Spherocylinder(_), _) => {
                unimplemented!()
            }
        }
    }
}

/// Keeps track of what `Object` maps to what `SceneNode`, and handles updates
pub struct ObjectTracker {
    /// The set of objects
    objects: HashMap<ObjectID, (ObjectEnum, SceneNode)>,
    /// The scene to which to attach new objects
    parent: SceneNode,
}

impl From<Sphere> for ObjectEnum {
    fn from(s: Sphere) -> ObjectEnum {
        ObjectEnum::Sphere(s)
    }
}

impl From<Spherocylinder> for ObjectEnum {
    fn from(s: Spherocylinder) -> ObjectEnum {
        ObjectEnum::Spherocylinder(s)
    }
}

impl ObjectTracker {
    /// Create a new `ObjectTracker` associated with a given `Window`.
    pub fn new(window: &mut Window) -> ObjectTracker {
        ObjectTracker { objects: HashMap::new(), parent: window.add_group() }
    }

    /// The meat of `ObjectTracker`. Update old objects and the scene to match
    /// new objects
    pub fn update(&mut self, frame: &Frame, palette: &mut Palette) {
        // TODO: this used to be &ObjectID, which is probably faster
        let mut seen: HashSet<ObjectID> = FromIterator::from_iter(self.objects.keys()
            .map(|ref k|{(*k).clone()}));

        let iter = frame.spheres.iter().map(|s| ObjectEnum::Sphere(s.clone()))
            .chain(frame.spherocylinders.iter().map(|s| ObjectEnum::Spherocylinder(s.clone())));

        for new_object in iter {
            let name: &ObjectID = new_object.id();
            match self.objects.entry(name.clone()) {
                Entry::Occupied(mut entry) => {
                    let &mut (ref mut obj, ref mut node) = entry.get_mut();
                    obj.update(&ObjectEnum::from(new_object.clone()), node);
                    palette.set_color(obj.id(), node);
                    //let is_invisible = node.data().is_root();
                    // if is_invisible {
                    //     self.parent.add_child(node);
                    // }
                    seen.remove(name);
                }
                Entry::Vacant(v) => {
                    let mut node = new_object.new_node(&mut self.parent);
                    palette.set_color(new_object.id(), &mut node);
                    let _ = v.insert((ObjectEnum::from(new_object.clone()), node));
                }
            }
        }

        for k in seen {
            let _ = self.objects.get_mut(&k).map(|&mut (_, ref mut node)| {
                node.unlink();
            });
            let _ = self.objects.remove(&k);
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
/// Data object for a spherical particle
pub struct Sphere {
    /// location of the sphere
    pub loc: (f32, f32, f32),
    /// Diameter
    pub diameter: f32,
    /// Color. if none, one will be assigned
    pub names: ObjectID,
}

impl Sphere {
    /// get the location as a Vec3
    pub fn x(&self) -> na::Vec3<f32> {
        let (x,y,z) = self.loc;
        na::Vec3::new(x, y, z)
    }
}

impl Object for Sphere {
    fn id(&self) -> &ObjectID {
        &self.names
    }

    fn new_node(&self, parent: &mut SceneNode) -> SceneNode {
        let mut node = parent.add_sphere(self.diameter / 2.0);
        node.set_local_translation(self.x());

        node
    }

    fn update(&mut self, other: &Self, node: &mut SceneNode) {
        if (self.diameter - other.diameter).abs() > EPSILON {
            self.diameter = other.diameter;
            node.set_local_scale(self.diameter, self.diameter, self.diameter);
        }

        if self.loc != other.loc {
            self.loc = other.loc;
            node.set_local_translation(self.x());
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
/// Data object for a spherocylindrical particle
pub struct Spherocylinder {
    /// location of the sphere
    pub loc: (f32, f32, f32),
    /// The central axis of the sphere
    pub axis: (f32, f32, f32),
    /// Diameter
    pub diameter: f32,
    /// Color. if none, one will be assigned
    pub names: ObjectID,
}

impl Spherocylinder {
    /// get the location as a Vec3
    pub fn x(&self) -> na::Vec3<f32> {
        let (x,y,z) = self.loc;
        na::Vec3::new(x, y, z)
    }

    /// get the axis as a Vec3
    pub fn get_axis(&self) -> na::Vec3<f32> {
        let (x,y,z) = self.axis;
        na::Vec3::new(x, y, z)
    }
}

impl Object for Spherocylinder {
    fn id(&self) -> &ObjectID {
        &self.names
    }

    fn new_node(&self, parent: &mut SceneNode) -> SceneNode {
        let diam = self.diameter; // radius
        let h = self.get_axis().norm() - self.diameter;
        // We start with a radius of 0.5 (diameter 1), and internal height h / diam.
        // Then we scale by diam.
        let mut node = parent.add_capsule(0.5, h / diam);
        node.set_local_scale(diam, diam, diam);
        node.set_local_translation(self.x());

        let y = na::Vec3::new(0., 1., 0.);
        let rot = y.rotation_to(&self.get_axis()).rotation();
        // println!("new: {:?}", self.names);
        // println!("ax: {:?}, rot: {:?}", self.get_axis(), rot);
        node.set_local_rotation(rot);

        node
    }

    fn update(&mut self, other: &Self, node: &mut SceneNode) {
        let diameter_change = ((other.diameter - self.diameter) / self.diameter).abs();

        let (l, l_new) = (self.get_axis().norm(), other.get_axis().norm());
        let length_change = ((l_new - l) / l).abs();
        let axis_change = (other.get_axis() - self.get_axis()).norm() / self.get_axis().norm();


        if diameter_change > EPSILON || length_change > EPSILON {
            if (diameter_change - length_change).abs() > 1e-3 {
                println!("diameter_change: {:?}, length_change: {:?}",
                        diameter_change, length_change);
                panic!("Don't know how to stretch spherocylinders.")
            }

            self.diameter = other.diameter;
            self.axis = other.axis;
            node.set_local_scale(self.diameter, self.diameter, self.diameter);
        }

        if self.loc != other.loc {
            self.loc = other.loc;
            node.set_local_translation(self.x());
        }

        if axis_change > EPSILON {
            self.axis = other.axis;
            let y = na::Vec3::new(0.0, 1.0, 0.0);
            let rot = y.rotation_to(&self.get_axis()).rotation();
            node.set_local_rotation(rot);
        }
    }
}
