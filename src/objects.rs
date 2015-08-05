//! The types of objects that can be drawn by ParView

extern crate nalgebra as na;
extern crate serde;

//use rustc_serialize::json::{self, Json, ToJson};
use serde::{Serialize, Serializer, Deserialize, Deserializer, Error};
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::iter::FromIterator;

use palette::Palette;

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

impl Deserialize for ObjectID {
    fn deserialize<D: Deserializer>(d: &mut D) -> Result<Self, D::Error> {
        Vec::deserialize(d).map(|n| ObjectID(n))
    }
}

impl ObjectID{
    /// Create a new `ObjectTracker` associated with a given `Window`.
    pub fn new(names : Vec<String>) -> ObjectID {
        ObjectID(names)
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
    fn new_node(&self, window : &mut SceneNode) -> SceneNode;
    /// If `self` is the "old" sphere, and `other` is the new one, then for each difference,
    /// Update `self` and the corresponding SceneNode to match
    fn update(&mut self, other: &Self, nodes: &mut SceneNode);
}

/// Keeps track of what `Object` maps to what `SceneNode`, and handles updates
pub struct ObjectTracker<'a, T : Object> {
    /// The set of objects
    objects : HashMap<&'a ObjectID, (T, SceneNode)>,
    /// The scene to which to attach new objects
    parent : SceneNode
}

// TODO: Should this be something like this:
/*
/// Tracks all objects, regardless of type
pub struct ObjectTracker<'a> {
    objects : AnyMap<HashMap<&'a ObjectID, (T, SceneNode)>>,
}
*/

impl<'a, T : Object + 'a> ObjectTracker<'a, T> {
    /// Create a new `ObjectTracker` associated with a given `Window`.
    pub fn new(window: &mut Window) -> ObjectTracker<'a, T> {
        ObjectTracker{
            objects: HashMap::new(),
            parent: window.add_group()
        }
    }
    
    /// The meat of `ObjectTracker`. Update old objects and the scene to match
    /// new objects
    pub fn update<I : Iterator<Item=&'a T>>(&mut self, iter : I, palette : &mut Palette) {
        let mut seen : HashSet<&ObjectID> = FromIterator::from_iter(self.objects.keys().map(|&k|{k}));
        
        for new_object in iter {
            let name : &'a ObjectID = new_object.id();
            match self.objects.entry(name) {
                Entry::Occupied(mut entry) => {
                    let &mut (ref mut obj, ref mut node) = entry.get_mut();
                    obj.update(&new_object, node);
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
                    let _ = v.insert((new_object.clone(), node));
                }
            }
        }
        
        for k in seen {
            let _ = self.objects.get_mut(k).map(|&mut (_, ref mut node)| {
                let _ = node.unlink();
            });
            let _ = self.objects.remove(k);
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
/// A single frame, containing spheres
/// format is (location, radius, Option(color triple))
pub struct Sphere {
    /// location of the sphere
    pub loc : (f32,f32,f32),
    /// Radius
    pub radius : f32,
    /// Color. if none, one will be assigned
    pub names : ObjectID,
}

impl Sphere {
    /// get the location as a Vec3
    pub fn x(&self) -> na::Vec3<f32> {
        let (x,y,z) = self.loc;
        na::Vec3::new(x,y,z)
    }
}

impl Object for Sphere {
    fn id(&self) -> &ObjectID {&self.names}
    
    fn new_node(&self, parent : &mut SceneNode) -> SceneNode {
        let mut node = parent.add_sphere(self.radius);
        node.set_local_translation(self.x());
        node.set_local_scale(self.radius, self.radius, self.radius);
        
        node
    }
    
    fn update(&mut self, other: &Self, node: &mut SceneNode){
        if self.radius != other.radius {
            self.radius = other.radius;
            node.set_local_scale(self.radius, self.radius, self.radius);
        }
        
        if self.loc != other.loc {
            self.loc = other.loc;
            node.set_local_translation(self.x());
        }
    }
}
