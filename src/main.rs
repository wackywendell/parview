/*!
# ParView
*/

#![deny(non_camel_case_types)]
#![deny(unnecessary_parens)]
#![deny(non_uppercase_statics)]
#![deny(unnecessary_qualification)]
#![deny(missing_doc)]
#![deny(unused_result)]
#![deny(unnecessary_typecast)]
#![deny(visible_private_types)]

extern crate serialize;

extern crate nalgebra;
extern crate kiss3d;

use serialize::{json, Encodable, Decodable};
use std::rand::random;
use std::io::{File,BufferedWriter,BufferedReader};

use nalgebra::na;

#[deriving(Decodable, Encodable, Clone)]
/// A single frame, containing spheres
pub struct Frame {
    spheres : Vec<(na::Vec3<f32>, f32)>
}

/// A random Vec3<f32>, with coordinates in (-0.5, 0.5)
pub fn rand_vec() -> na::Vec3<f32> {
    na::Vec3::new(random(), random(), random()) - na::Vec3::new(0.5f32, 0.5f32, 0.5f32)
}

fn generate_frame() {
    let f = Frame {
        spheres : Vec::from_fn(16, |_| {
            let loc : na::Vec3<f32> = rand_vec();
            let s : f32 = random();
            (loc, s*0.1)
        })
            
    };
    
    let mut framevec = vec!();
    let mut f2 = f.clone();
    
    for _ in range(0,200){
        f2 = Frame {
            spheres : f2.spheres.iter().map(|&(v, s)| {
                (v + (rand_vec() * 0.1f32), s)
            }).collect()
        };
        framevec.push(f2.clone());
    }
    
    let path = Path::new("test_frame.json");
    let mut file = BufferedWriter::new(File::create(&path));
    {
        let mut encoder = json::PrettyEncoder::new(&mut file);
        match framevec.encode(&mut encoder) {
            Ok(()) => (),
            Err(e) => fail!("json encoding error: {}", e)
        };
    }
}

fn draw_cube(window : &mut kiss3d::window::Window) -> kiss3d::scene::SceneNode {
    let rotations : Vec<Option<na::Vec3<f32>>> = vec!(
        None, Some(na::Vec3::x()), Some(na::Vec3::z()));
    let translations = vec!((-0.5f32, -0.5),
                            (-0.5,0.5),
                            (0.5, -0.5),
                            (0.5, 0.5));
    let translations : Vec<na::Vec3<f32>> = translations
        .iter()
        .map(|&(x,z)| na::Vec3::new(x,0.0,z))
        .collect();
    
    let mut cube = window.add_group();
    for rot in rotations.iter() {
        for t in translations.iter() {
            let mut caps = cube.add_capsule(0.01, 1.0);
            caps.append_translation(t);
            match rot {
                &Some(ref r) => {caps.append_rotation(&(r * std::f32::consts::FRAC_PI_2))},
                &None => {}
            }
            //caps.append_translation(&Vec3::new(-0.5, 0.0, -0.5));
        }
    }
    cube.set_color(1.0, 0.0, 0.0);
    
    cube
}

fn main() {
    generate_frame();
    let path = Path::new("test_frame.json");
    let mut file = BufferedReader::new(File::open(&path));
    let coded = json::from_reader(&mut file).unwrap();
    let mut decoder =json::Decoder::new(coded);
    let frames: Vec<Frame> = Decodable::decode(&mut decoder).unwrap();
    
    let cols = [(1.,1.,1.),
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
    
    let f : Frame = frames.get(0).clone();
    
    kiss3d::window::Window::spawn("Kiss3d: draw_sphere", |window| {
        let cube = draw_cube(window);

        let eye              = na::Vec3::new(0.0f32, 0.0, 2.0);
        let at               = na::zero();
        let mut arc_ball     = kiss3d::camera::ArcBall::new(eye, at);

        window.set_camera(&mut arc_ball as &mut kiss3d::camera::Camera);

        //window.set_background_color(1.0, 1.0, 1.0);
        window.set_light(kiss3d::light::StickToCamera);
        window.set_framerate_limit(Some(20));
        
        
        let mut sphere_set = vec!();
        for (n, &(loc, diam)) in f.spheres.iter().enumerate() {
            let mut s = window.add_sphere(diam / 2.0);
            let (r,g,b) = cols[n % cols.len()];
            s.set_color(r,g,b);
            s.append_translation(&loc);
            sphere_set.push(s);
        }
        
        let mut t = 0;
        
        let font = kiss3d::text::Font::new(&Path::new("/usr/share/fonts/OTF/Inconsolata.otf"), 120);
        
        window.render_loop(|window|{
            t += 1;
            
            let i = (t / 10) % frames.len();
            window.draw_text(format!("t = {}", i).as_slice(), 
                &na::zero(), &font, &na::Vec3::new(0.0, 1.0, 1.0));
            
            if t % 10 == 0 {
                let frame = frames.get(i);
                for (&(loc, diam), s) in frame.spheres.iter().zip(sphere_set.mut_iter()) {
                    s.set_local_translation(loc);
                }
            }
        });
    })
}
