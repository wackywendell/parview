/*!
# ParView
*/
// #![feature(plugin)]
// #![plugin(docopt_macros)]

#![deny(non_camel_case_types)]
#![deny(unused_parens)]
#![deny(non_upper_case_globals)]
#![deny(unused_qualifications)]
#![deny(missing_docs)]
#![deny(unused_results)]

// extern crate docopt_macros;
extern crate docopt;

extern crate rand;
extern crate rustc_serialize;
extern crate flate2;

extern crate nalgebra as na;
extern crate kiss3d;
extern crate glfw;
extern crate parview;

use rustc_serialize::{json, Decodable};
use rustc_serialize::json::Json;
use flate2::read::GzDecoder;
use rand::random;
use std::fs::File;
use std::path::Path;
use std::io::{BufReader,Write};

use kiss3d::window::Window;
use glfw::{WindowEvent,Key};

use parview::{Sphere,Frame,rand_vec};


/// Generate an example json file
pub fn generate_frame() {
    let spheres = (0..16).map(|_| {
        let loc : na::Vec3<f32> = rand_vec();
        let s : f32 = random();
        Sphere{loc:(loc.x, loc.y, loc.z), radius:s*0.2, color:None}
    }).collect();

    let f = Frame {
        spheres : spheres,
        text : None
    };

    let mut framevec : Vec<Frame> = vec!();

    for i in (0usize..40usize){
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
    let mut file = File::create(&path).unwrap();

    let val : String = json::encode(&framevec).unwrap();

    let _ = write!(file, "{}", val);
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

fn open_file(path : &Path) -> Result<Vec<Frame>, Box<std::error::Error>> {
    let mut buf : BufReader<File> = BufReader::new(try!(File::open(path)));
    // let f = try!(File::open(path));

    //~ let coded = json::from_reader(&mut buf).unwrap();
    //~ let mut decoder =json::Decoder::new(coded);
    //~ let frames: Vec<Frame> = Decodable::decode(&mut decoder).unwrap();

    let ext : Option<&str> = path.extension().and_then(|s| {s.to_str()});

    if ext == Some("gz") {
        println!("gz ext!");
    }

    let coded_opt = match ext {
        Some("gz") => {
            let mut gzbuf = try!(GzDecoder::new(buf));
            Json::from_reader(&mut gzbuf)
        },
        _ => {
            Json::from_reader(&mut buf)
            }
    };

    let coded = try!(coded_opt);

    let mut decoder = json::Decoder::new(coded);
    let json_result = try!(Decodable::decode(&mut decoder));

    return Ok(json_result);
}

// docopt!

#[derive(RustcDecodable, Debug)]
struct Args {
    flag_g : bool,
    arg_file : Option<String>
}

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: parview [-h | --help] [-g] [<file>]

Options:
    [-h | --help]   Help and usage
    -g

Arguments:
    <file>      json file representing the frames. json.gz also accepted, if the extension is \".gz\".
";
/// Main entry point, now using test_frame.json
pub fn main() {
    let args: Args = docopt::Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    // with docopt! macro
    // let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    if args.flag_g {
        generate_frame()
    }

    let fname : String = match args.arg_file {
        Some(s) => s,
        None => std::str::FromStr::from_str("test_frame.json").unwrap()
    };

    let path = Path::new(&*fname);
    let frames = open_file(&path).unwrap();

    let ref f : Frame = frames[0];

    let title : String = format!("Parviewer: {}", path.to_string_lossy());
    let mut window = Window::new(&*title);
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

    //TODO: Include this font as an asset
    let fontsize = 48;
    let font = kiss3d::text::Font::new(&Path::new("/usr/share/fonts/OTF/Inconsolata.otf"), fontsize);

    while window.render_with_camera(&mut arc_ball) {
        for mut event in window.events().iter() {
            match event.value {
                WindowEvent::Key(key, _, glfw::Action::Release, _) => {
                    // Default to inhibiting, although this can be overridden
                    let mut inhibit = true;
                    match key {
                        Key::Q => {return;},
                        Key::Comma => {timer.slower();},
                        Key::Period => {timer.faster();},
                        Key::F => {timer.switch_direction();},
                        Key::Up => {
                            arc_ball.set_pitch(3.14159/3.);
                            arc_ball.set_yaw(3.14159/4.);
                        },
                        Key::Down => {
                            arc_ball.set_pitch(3.14159/2.);
                            arc_ball.set_yaw(3.14159/2.);
                        },
                        Key::W => {
                            println!("yaw: {:6.2}, pitch: {:6.2}", arc_ball.yaw(), arc_ball.pitch());
                        },
                        code => {
                            println!("You released the key with code: {:?}", code);
                            inhibit = false;
                        }

                    }
                    event.inhibited = inhibit;
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
                window.draw_text(t, &na::orig(), &font, &na::Pnt3::new(1.0, 1.0, 1.0));
            }
            None => {}
        }

        let text_loc = na::Pnt2::new(0.0, window.height() * 2. - (fontsize as f32));
        window.draw_text(&*format!("t:{:6}, dt:{:8.2}", i, timer.get_dt()),
                &text_loc, &font, &na::Pnt3::new(1.0, 1.0, 1.0));
    };
}
