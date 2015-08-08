/*!
# ParView
*/
// #![feature(plugin)]
// #![plugin(docopt_macros)]

#![feature(concat_idents)]

#![deny(non_camel_case_types)]
#![deny(unused_parens)]
#![deny(non_upper_case_globals)]
#![deny(unused_qualifications)]
#![deny(missing_docs)]
#![deny(unused_results)]

// extern crate docopt_macros;
extern crate docopt;

extern crate rand;
extern crate rustc_serialize; // for docopt, toml
extern crate serde; // for json
extern crate toml;
extern crate flate2;

extern crate nalgebra as na;
extern crate kiss3d;
extern crate glfw;
extern crate parview;

use serde::{Error};
use flate2::read::GzDecoder;
use rand::random;
use std::fs::File;
use std::path::Path;
use std::io;
use std::io::{Read,Write};
use std::f32::consts::PI;

use kiss3d::window::Window;
use glfw::{WindowEvent,Key};

use parview::{Sphere,Frame,rand_vec};

/// Generate an example json file
pub fn generate_frame(path : &Path) -> io::Result<()> {
    let spheres = (0..16).map(|n| {
        let loc : na::Vec3<f32> = rand_vec();
        let s : f32 = random();
        let names = parview::objects::ObjectID(vec![
            format!("{}", n / 4 + 1),
            format!("{}", n % 4 + 1)
        ]);
        Sphere{loc:(loc.x, loc.y, loc.z), diameter:s*0.2, names:names}
    }).collect();

    let f = Frame {
        spheres : spheres,
        text : None
    };

    let mut framevec : Vec<Frame> = vec!();

    for i in (0usize..40usize){
        let mut f2 = Frame {
            spheres : f.spheres.iter().enumerate().map(|(n, ref s)| {
                    let mut newr = s.diameter;
                    if n == 0 {
                        newr = random::<f32>() * 0.2f32;
                    }
                    let (sx, sy, sz) = s.loc;
                    let na::Vec3{x, y, z} = na::Vec3::new(sx, sy, sz) + 
                        (rand_vec() * 0.1f32);
                    Sphere {
                        loc: (x, y, z), 
                        diameter: newr,
                        names: s.names.clone()
                    }
                }).collect(),
            text : Some(format!("Frame {} with {} spheres", i, f.spheres.len()))
        };

        if i > 10 && i < 20 {
            let l = f2.spheres.len();
            f2.spheres.truncate(l - 8);
            f2.text = Some(format!("Frame {} with {} spheres", i, f2.spheres.len()));
        }
        framevec.push(f2);
    }

    let mut file = File::create(&path).unwrap();
    
    serde::json::ser::to_writer_pretty(&mut file, &framevec)
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

fn open_file(path : &Path) -> Result<Vec<Frame>, serde::json::error::Error> {
    let mut buf : io::BufReader<File> = io::BufReader::new(try!(File::open(path)));
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
            serde::json::de::from_reader(&mut gzbuf)
        },
        _ => {
            serde::json::de::from_reader(&mut buf)
            }
    };

    coded_opt
}

macro_rules! make_docs {
        ($($name:ident, $flagname:ident : $typ:ty),*) => {
        
        /// Structure for converting command-line arguments into a useful form.
        /// Can be converted to Config for use with parview.
        #[derive(RustcDecodable, Debug)]
        pub struct Args {
            $(
                /// See usage string for field details.
                pub $flagname : $typ,
            )*
        }
        
        /// Structure for holding configuration options from command-line or config file.
        /// Can be converted from Args.
        #[derive(Debug)]
        pub struct Config {
            $(
                /// See usage string for field details.
                pub $name : $typ,
            )*
        }
        
        impl From<Args> for Config {
            fn from(args : Args) -> Config {
                Config {
                    $(
                        $name : args.$flagname,
                    )*
                }
            }
        }
        
        impl Config {
            /// Convert Docopt usage to Config, using argv from an iterator
            pub fn from_docopt<I, S>(usage : &str, argv : I) -> Result<Config, docopt::Error>
                where I: Iterator<Item=S>, S: AsRef<str> {
                    docopt::Docopt::new(usage)
                                    .map(|d| d.argv(argv))
                                    .and_then(|d| d.decode())
                                    .map(|args : Args| Config::from(args))
            }
            
            /// Convert Docopt usage to Config, using command-line argv
            pub fn from_usage(usage : &str) -> Result<Config, docopt::Error> {
                docopt::Docopt::new(usage)
                                .and_then(|d| d.decode())
                                .map(|args : Args| Config::from(args))
            }
        }
                
        impl Default for Config {
            fn default() -> Config {
                let argv : &[&str] = &[];
                Config::from_docopt(USAGE, argv.iter()).unwrap()
            }
        }
    }
}

// Generate Args, and Config.
// Args is the structure for Docopt, and Config is the corresponding configuration we actually use.
// Note that Config::from(args : Args) exists.
make_docs!{
    generate, flag_g : bool,
    pitch, flag_pitch : f32,
    yaw, flag_yaw : f32,
    fov, flag_fov : f32,
    distance, flag_distance : f32,
    width, flag_width : u32,
    height, flag_height : Option<u32>,
    palette, flag_p : Option<String>,
    pauseloop, flag_pauseloop : Option<f32>,
    file, arg_file : Option<String>
}

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: parview [options] [--] [<file>]

Options:
    -h, --help            Help and usage
    -g, --generate        Generate test_frames.json
    -p, --palette FILE    Use palette file (toml file)
    --pitch ANGLE         Set initial pitch (degrees) [default: 90]
    --yaw ANGLE           Set initial yaw (degrees) [default: 0]
    --fov ANGLE           Set camera field-of-view angle (degrees) [default: 45]
    --width PIXELS        Set window width (pixels) [default: 600]
    --height PIXELS       Set window height, if different from width (pixels)
    --distance D          Set distance from box (L) [default: 2]
    --pauseloop SECONDS   When finished, pause for SECONDS, then loop. By default, does not loop.

Arguments:
    <file>      json file representing the frames. json.gz also accepted, if the extension is \".gz\".
";
/// Main entry point, now using test_frame.json
pub fn main() {
    let config: Config = Config::from_usage(USAGE)
                            .unwrap_or_else(|e| e.exit());
    // with docopt! macro
    // let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    
    let fname : String = match config.file {
        Some(s) => s,
        None => std::str::FromStr::from_str("test_frame.json").unwrap()
    };
    let path : &Path = Path::new(&*fname);
    
    if config.generate {
        generate_frame(path).unwrap();
        let _ = config.palette.as_ref().map(|fname| {
            let path : &Path = Path::new(fname);
            let mut file : File = File::create(path).unwrap();
            println!("Default...");
            let mut default_palette = parview::Palette::default();
            println!("insert...");
            let _ = default_palette.assigned.insert(
                parview::ObjectID(vec!("A".into())), 
                parview::Color(255, 0, 0),
            );
            println!("insert 2...");
            let _ = default_palette.assigned.insert(
                parview::ObjectID(vec!("B".into())), 
                parview::Color(0, 255, 0),
            );
            println!("string...");
            let s : String = toml::encode_str(&default_palette);
            println!("Write!");
            write!(file, "{}", s).unwrap();
        });
    }
    
    let frames = open_file(path).unwrap();
    
    let title : String = format!("Parviewer: {}", path.to_string_lossy());
    let width : u32 = config.width;
    let height : u32 = config.height.unwrap_or(width);
    let mut window = Window::new_with_size(&*title, width, height);
    let _ = draw_cube(&mut window);

    let eye              = na::Pnt3::new(0.0f32, 0.0, config.distance);
    let at               = na::orig();
    let mut arc_ball     = kiss3d::camera::ArcBall::new_with_frustrum(config.fov * PI / 180., 0.1, 1024.0, eye, at);
    
    arc_ball.set_yaw(config.yaw * PI / 180.);
    arc_ball.set_pitch(config.pitch * PI / 180.);

    //window.set_background_color(1.0, 1.0, 1.0);
    window.set_light(kiss3d::light::Light::StickToCamera);
    window.set_framerate_limit(Some(20));

    let mut nodes = parview::ObjectTracker::new(&mut window);
    let mut palette : parview::Palette = config.palette.map(|fname| {
        let path : &Path = Path::new(&fname[..]);
        let mut file : File = File::open(path).unwrap();
        let mut s = String::new();
        let _ = file.read_to_string(&mut s).unwrap();
        toml::decode_str(&s[..]).unwrap()
    }).unwrap_or_default();
    
    let mut lastframe : isize = -1;
    let mut timer = parview::Timer::new(vec![0.1, 0.2, 0.5, 1., 2., 5., 10.], Some(frames.len()));
    // TODO: add config
    timer.loop_pause = config.pauseloop;
    let mut text = None;

    //TODO: Include this font as an asset
    let fontsize = 48;
    let font = parview::inconsolata(fontsize);

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
                            arc_ball.set_pitch(PI/3.);
                            arc_ball.set_yaw(PI/4.);
                        },
                        Key::Down => {
                            arc_ball.set_pitch(PI/2.);
                            arc_ball.set_yaw(0.);
                        },
                        Key::W => {
                            println!("yaw: {:6.2}, pitch: {:6.2}, distance: {:6.2}", 
                                arc_ball.yaw() * 180. / PI, 
                                arc_ball.pitch() * 180. / PI,
                                arc_ball.dist()
                            );
                        },
                        Key::Num1 => {palette.toggle_partial(0);},
                        Key::Num2 => {palette.toggle_partial(1);},
                        Key::Num3 => {palette.toggle_partial(2);},
                        Key::Num4 => {palette.toggle_partial(3);},
                        Key::Num5 => {palette.toggle_partial(4);},
                        Key::Num6 => {palette.toggle_partial(5);},
                        Key::Num7 => {palette.toggle_partial(6);},
                        Key::Num8 => {palette.toggle_partial(7);},
                        Key::Num9 => {palette.set_all_partial(true);},
                        Key::Num0 => {palette.set_all_partial(false);},
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

        if lastframe != (i as isize) {
            let ref frame = frames[i];
            text = frame.text.clone();
            nodes.update(frame.spheres.iter(), &mut palette);
            lastframe = i as isize;
        }
        
        let text_color = na::Pnt3::new(1.0, 1.0, 1.0);
        match text {
            Some(ref t) => {
                window.draw_text(t, &na::orig(), &font, &text_color);
            }
            None => {}
        }
        
        // TODO: Figure out why the bottom is window.height() * 2.
        // Could be HiDPI
        let text_loc = na::Pnt2::new(0.0, window.height() * 2. - (fontsize as f32));
        window.draw_text(
            &*format!(
                "t:{:6}, dt:{:8.2}, coloring: {}", 
                i,
                timer.get_dt(),
                palette.partials_string()
            ),
            &text_loc, &font, &text_color
        );
    };
}
