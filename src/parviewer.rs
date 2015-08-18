//! The main entry point for a simulation.

use std;

use serde_json;
use na;
use kiss3d;
use glfw;

use flate2::read::GzDecoder;
use kiss3d::window::Window;
use glfw::{WindowEvent,Key};

use std::path::Path;
use std::fs::File;

use timer::Timer;
use misc;
use palette::Palette;
use objects::{Sphere,Frame,ObjectTracker};
use std::f32::consts::PI;

/// The configuration options for a Parviewer instance.
#[derive(Debug, RustcDecodable, RustcEncodable, Serialize, Deserialize)]
pub struct Config {
    /// Pitch of the camera, in degrees
    pub pitch : f32,
    /// Yaw of the camera, in degrees
    pub yaw : f32,
    /// Field-of-view range of the camera, in degrees
    pub fov : f32,
    /// Distance of the camera from the origin
    pub distance : f32,
    /// Window width
    pub width : u32,
    /// Window height
    pub height : u32,
    /// How long to pause before looping. None indicates no looping.
    pub pauseloop : Option<f32>,
    /// framerate limit
    pub framerate : f32,
    /// rotation
    pub rotate : f32
}

/// Open a `json` or `json.gz` file, and deserialize it into a `Vec<Frame>`
pub fn open_file(path : &Path) -> Result<Vec<Frame>, serde_json::error::Error> {
    let mut buf : std::io::BufReader<File> = std::io::BufReader::new(try!(File::open(path)));
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
            serde_json::de::from_reader(&mut gzbuf)
        },
        _ => {
            serde_json::de::from_reader(&mut buf)
            }
    };

    coded_opt
}

fn err_print(err : &std::error::Error) {
    println!("Cause: {}", err.description());
    println!("{}", err);
    println!("{:?}", err);
    if let Some(e) = err.cause() {
        err_print(e);
    }
}

/// The main entry point,maintaining a window, a Config, objects, etc.
pub struct Parviewer {
    config : Config,
    frames : Vec<Frame>,
    palette : Palette,
    /// Track where we are
    pub timer : Timer,
    window : Window,
    camera : kiss3d::camera::ArcBall,
    nodes : ObjectTracker<Sphere>,
    font : std::rc::Rc<kiss3d::text::Font>
}

impl Parviewer {
    /// Create a new Parviewer instance from a give Config
    pub fn new(frames : Vec<Frame>, palette : Palette, config : Config) 
                -> Result<Parviewer, Box<std::error::Error>> {
        
        // TODO: this is also a configuration option
        let title : String = format!("Parviewer");
        let width : u32 = config.width;
        let height : u32 = config.height;
        let mut window = Window::new_with_size(&*title, width, height);
        // TODO: cube configuration
        let _ = misc::draw_cube(&mut window);

        let eye              = na::Pnt3::new(0.0f32, 0.0, config.distance);
        let at               = na::orig();
        let mut arc_ball     = kiss3d::camera::ArcBall::new_with_frustrum(config.fov * PI / 180., 0.1, 1024.0, eye, at);
        
        arc_ball.set_yaw(config.yaw * PI / 180.);
        arc_ball.set_pitch(config.pitch * PI / 180.);

        //window.set_background_color(1.0, 1.0, 1.0);
        window.set_light(kiss3d::light::Light::StickToCamera);
        window.set_framerate_limit(Some(config.framerate as u64));

        let nodes = ObjectTracker::new(&mut window);
        
        // TODO: config?
        let dts_first = vec![1., 2., 3., 4., 6., 8., 12., 16., 24., 32., 48., 64., 96., 128.];
        let mut dts = dts_first.iter().rev().map(|n|{1./n}).collect::<Vec<f32>>();
        dts.extend(dts_first);
        dts.dedup();
        
        let mut timer = Timer::new(dts, Some(frames.len()));
        
        timer.loop_pause = config.pauseloop;
        timer.fps = config.framerate;

        let fontsize = 48;
        let font = misc::inconsolata(fontsize);
        
        Ok(Parviewer {
            config : config,
            frames : frames,
            palette : palette,
            timer : timer,
            window : window,
            nodes : nodes,
            camera : arc_ball,
            font : font
        })
    }
    
    /// View the current config.
    pub fn get_config(&self) -> &Config {
        &self.config
    }
    
    /// Start the whole running sequence.
    pub fn run(&mut self) {
        let mut lastframe : isize = -1;
        let mut text = None;
        while self.window.render_with_camera(&mut self.camera) {
            for mut event in self.window.events().iter() {
                match event.value {
                    WindowEvent::Key(key, _, glfw::Action::Release, _) => {
                        // Default to inhibiting, although this can be overridden
                        let mut inhibit = true;
                        match key {
                            Key::Q => {return;},
                            Key::Comma => {self.timer.slower();},
                            Key::Period => {self.timer.faster();},
                            Key::F => {self.timer.switch_direction();},
                            Key::Up => {
                                self.camera.set_pitch(PI/3.);
                                self.camera.set_yaw(PI/4.);
                            },
                            Key::Down => {
                                self.camera.set_pitch(PI/2.);
                                self.camera.set_yaw(0.);
                            },
                            Key::W => {
                                println!("yaw: {:6.2}, pitch: {:6.2}, distance: {:6.2}", 
                                    self.camera.yaw() * 180. / PI, 
                                    self.camera.pitch() * 180. / PI,
                                    self.camera.dist()
                                );
                            },
                            Key::Num1 => {self.palette.toggle_partial(0);},
                            Key::Num2 => {self.palette.toggle_partial(1);},
                            Key::Num3 => {self.palette.toggle_partial(2);},
                            Key::Num4 => {self.palette.toggle_partial(3);},
                            Key::Num5 => {self.palette.toggle_partial(4);},
                            Key::Num6 => {self.palette.toggle_partial(5);},
                            Key::Num7 => {self.palette.toggle_partial(6);},
                            Key::Num8 => {self.palette.toggle_partial(7);},
                            Key::Num9 => {self.palette.set_all_partial(true);},
                            Key::Num0 => {self.palette.set_all_partial(false);},
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

            let i = self.timer.incr();

            if lastframe != (i as isize) {
                let ref frame = self.frames[i];
                text = frame.text.clone();
                self.nodes.update(frame.spheres.iter(), &mut self.palette);
                lastframe = i as isize;
            }
            
            if self.config.rotate.abs() > 1e-6 {
                let new_yaw = self.camera.yaw() + PI*self.config.rotate / 180.0;
                self.camera.set_yaw(new_yaw);
            }
            
            // TODO: add config
            let text_color = na::Pnt3::new(1.0, 1.0, 1.0);
            match text {
                Some(ref t) => {
                    self.window.draw_text(t, &na::orig(), &self.font, &text_color);
                }
                None => {}
            }
            
            // TODO: Figure out why the bottom is window.height() * 2.
            // Could be HiDPI
            let text_loc = na::Pnt2::new(0.0, self.window.height() * 2. - (self.font.height() as f32));
            let dt = self.timer.get_dt();
            let dt_text = if dt >= 0.6 || dt.abs() < 1e-6 {
                format!("{}", dt)
            } else {
                format!("1/{}", 1./dt)
            };
            
            self.window.draw_text(
                &*format!(
                    "t:{:6.2}, dt:{}, coloring: {}", 
                    self.timer.get_time(),
                    dt_text,
                    self.palette.partials_string()
                ),
                &text_loc, &self.font, &text_color
            );
        };
    }
}
