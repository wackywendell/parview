/*!
# ParView
*/
#![feature(phase)]

#![deny(non_camel_case_types)]
#![deny(unused_parens)]
#![deny(non_upper_case_globals)]
#![deny(unused_qualifications)]
#![deny(missing_docs)]
#![deny(unused_results)]
#![deny(unused_typecasts)]

#[phase(plugin)]
extern crate docopt_macros;
extern crate docopt;

extern crate "rustc-serialize" as rustc_serialize;
extern crate flate2;

extern crate "nalgebra" as na;
extern crate kiss3d;
extern crate glfw;
extern crate parview;

use rustc_serialize::{json, Encodable, Decodable};
use flate2::reader::GzDecoder;
use std::rand::random;
use std::io::{File,BufferedWriter,BufferedReader};

use kiss3d::window::Window;
use glfw::{WindowEvent,Key};

use parview::{Sphere,Frame,rand_vec};


/// Generate an example json file
pub fn generate_frame() {
    let f = Frame {
        spheres : Vec::from_fn(16, |_| {
            let loc : na::Vec3<f32> = rand_vec();
            let s : f32 = random();
            Sphere{loc:(loc.x, loc.y, loc.z), radius:s*0.2, color:None}
        }),
        text : None
    };
    
    let mut framevec : Vec<Frame> = vec!();
    
    for i in range(0u,40u){
        let mut f2 = Frame {
            spheres : f.spheres.iter().enumerate().map(|(n, &s)| {
                    let mut newr = s.radius;
                    let mut color = s.color;
                    if n == 0 {
                        newr = random::<f32>() * 0.2f32;
                        color = Some((random::<u8>(), random::<u8>(), random::<u8>()));
                    }
                    Sphere::new(s.x() + (rand_vec() * 0.1f32), newr, color)
                }).collect(),
            text : Some(format!("Frame {} with {} spheres", i, f.spheres.len()))
        };
        
        if i > 10 && i < 20 {
            let l = f2.spheres.len();
            f2.spheres.truncate(l - 8);
        }
        framevec.push(f2);
    }
    
    let path = Path::new("test_frame.json");
    let mut file = BufferedWriter::new(File::create(&path));
    {
        let mut encoder = json::PrettyEncoder::new(&mut file);
        match framevec.encode(&mut encoder) {
            Ok(()) => (),
            Err(e) => panic!("json encoding error: {}", e)
        };
    }
}

fn draw_cube(window : &mut Window) -> kiss3d::scene::SceneNode {
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
                &Some(r) => {caps.append_rotation(&(r * std::f32::consts::FRAC_PI_2))},
                &None => {}
            }
            //caps.append_translation(&Vec3::new(-0.5, 0.0, -0.5));
        }
    }
    cube.set_color(1.0, 0.0, 0.0);
    
    cube
}

fn open_file(path : &Path) -> std::io::IoResult<Vec<Frame>> {
    let mut buf = BufferedReader::new(File::open(path));
    //~ let coded = json::from_reader(&mut buf).unwrap();
    //~ let mut decoder =json::Decoder::new(coded);
    //~ let frames: Vec<Frame> = Decodable::decode(&mut decoder).unwrap();
    
    let coded_opt = match path.extension(){
        Some(b"gz") => match GzDecoder::new(buf) {
                Err(e) => {return Err(e);}
                Ok(mut gzbuf) => {json::from_reader(&mut gzbuf)}
        },
        _ => {
            json::from_reader(&mut buf)
            }
    };
    
    let json_result = match coded_opt {
        Err(e) => {
            return Err(std::io::IoError{
                kind: std::io::InvalidInput,
                desc: "Parser Error",
                detail: Some(format!("{}", e))
            });
        }
        Ok(coded) => {
            let mut decoder =json::Decoder::new(coded);
            Decodable::decode(&mut decoder)
        }
    };
    
    match json_result {
        Ok(frames) => Ok(frames),
        Err(e) =>  Err(
            std::io::IoError{
                kind: std::io::InvalidInput,
                desc: "Decoder Error",
                detail: Some(format!("{}", e))
            }
        )
    }
}

docopt!(Args, "
Usage: parview [-h | --help] [-g] [<file>]

Options:
    [-h | --help]   Help and usage
    -g

Arguments:
    <file>      json file representing the frames. json.gz also accepted, if the extension is \".gz\".
", 
    flag_g : bool, 
    arg_file : Option<String>);

/// Main entry point, now using test_frame.json
pub fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    if args.flag_g {
        generate_frame()
    }
    
    let path = Path::new(args.arg_file.unwrap_or("test_frame.json".to_string()));
    let frames = open_file(&path).unwrap();
    
    let ref f : Frame = frames[0];
    
    let mut window = Window::new(format!("Parviewer: {}", path.as_str()).as_slice());
    let _ = draw_cube(&mut window);

    let eye              = na::Pnt3::new(0.0f32, 0.0, 2.0);
    let at               = na::orig();
    let mut arc_ball     = kiss3d::camera::ArcBall::new(eye, at);

    //window.set_background_color(1.0, 1.0, 1.0);
    window.set_light(kiss3d::light::Light::StickToCamera);
    window.set_framerate_limit(Some(20));
    
    let mut nodes = parview::SphereNodes::new(f.spheres.iter(), &mut window);
    
    let mut lastframe = -1;
    let mut timer = parview::Timer::new(vec![1./16., 1./8., 1./4., 1./2.,1., 2., 5., 10.], Some(frames.len()));
    let mut text = None;
    
    let fontsize =48;
    let font = kiss3d::text::Font::new(&Path::new("/usr/share/fonts/OTF/Inconsolata.otf"), fontsize);
    
    while window.render_with_camera(&mut arc_ball) {
        for mut event in window.events().iter() {
            match event.value {
                WindowEvent::Key(Key::Q, _, glfw::Action::Release, _) => {
                    return;
                },
                WindowEvent::Key(Key::Comma, _, glfw::Action::Release, _) => {
                    timer.slower();
                    event.inhibited = true; // override the default keyboard handler
                },
                WindowEvent::Key(Key::Period, _, glfw::Action::Release, _) => {
                    timer.faster();
                    event.inhibited = true; // override the default keyboard handler
                },
                WindowEvent::Key(Key::F, _, glfw::Action::Release, _) => {
                    timer.switch_direction();
                    event.inhibited = true; // override the default keyboard handler
                }
                WindowEvent::Key(Key::Up, _, glfw::Action::Release, _) => {
                    arc_ball.set_pitch(3.14159/3.);
                    arc_ball.set_yaw(3.14159/4.);
                    event.inhibited = true // override the default keyboard handler
                },
                WindowEvent::Key(Key::Down, _, glfw::Action::Release, _) => {
                    arc_ball.set_pitch(3.14159/2.);
                    arc_ball.set_yaw(3.14159/2.);
                    event.inhibited = true // override the default keyboard handler
                },
                WindowEvent::Key(Key::W, _, glfw::Action::Release, _) => {
                    println!("yaw: {:6.2}, pitch: {:6.2}", arc_ball.yaw(), arc_ball.pitch());
                    //~ println!("Do not try to press escape: the event is inhibited!");
                    event.inhibited = true // override the default keyboard handler
                },
                WindowEvent::Key(code, _, glfw::Action::Release, _) => {
                    println!("You released the key with code: {}", code);
                    //~ println!("Do not try to press escape: the event is inhibited!");
                    event.inhibited = true // override the default keyboard handler
                },
                _ => {}
            }
        }
                
        let i = timer.incr();
        
        if lastframe != i {
            let ref frame = frames[i];
            text = frame.text.clone();
            nodes.update(frame.spheres.iter(), &mut window);
            lastframe = i;
        }
        
        match text {
            Some(ref t) => {
                window.draw_text(t.as_slice(), &na::orig(), &font, &na::Pnt3::new(1.0, 1.0, 1.0));
            }
            None => {}
        }
        
        let text_loc = na::Pnt2::new(0.0, window.height() * 2. - (fontsize as f32));
        window.draw_text(format!("t:{:6}, dt:{:8.2}", i, timer.get_dt()).as_slice(),
                &text_loc, &font, &na::Pnt3::new(1.0, 1.0, 1.0));
    };
}
