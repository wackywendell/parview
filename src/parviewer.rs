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
use std::f32::consts::PI;

use timer::Timer;
use misc;
use palette::{Color,Palette};
use objects::{Frame,ObjectTracker};

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

/// The main entry point,maintaining a window, a Config, objects, etc.
pub struct Parviewer {
    config : Config,
    frames : Vec<Frame>,
    /// Palette
    pub palette : Palette,
    /// Track where we are
    pub timer : Timer,
    /// kiss3d Window
    pub window : Window,
    /// Camera
    pub camera : kiss3d::camera::ArcBall,
    nodes : ObjectTracker,
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
        
        // let mut capsule = window.add_capsule(0.25, 0.5);
        // // capsule.set_local_scale(1.0, 1.0, 1.0);
        // 
        // let diam = 0.5;
        // let axnorm = 1.0;
        // let h = axnorm - diam;
        // 
        // let mut capsule2 = window.add_capsule(0.5, h / diam);
        // capsule2.set_local_scale(diam, diam, diam);
        // capsule2.set_local_translation(na::Vec3::new(0.5, 0., 0.));
        
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

    /// Borrow current frame
    pub fn get_frame(&self) -> &Frame {
        let ix = self.timer.get_index();
        &self.frames[ix]
    }
    
    /// Draw some text in the window, with coordinates in the window frame (i.e., 0 to 1).
    pub fn draw_text(&mut self, t: &str, x: f32, y: f32, color: Color) {
        let max_width = self.window.width() * 2.;
        // TODO: Figure out why the bottom is window.height() * 2.
        let max_height = self.window.height() * 2. - (self.font.height() as f32);
        let text_loc = na::Pnt2::new(x * max_width, y * max_height);
        let text_color = color.to_pnt3();
        
        self.window.draw_text(t, &text_loc, &self.font, &text_color);
    }
    
    /// Draw some text in the window, with coordinates in the window frame (i.e., 0 to 1).
    pub fn draw_frame_text(&mut self, x: f32, y: f32, color: Color) {
        let ix = self.timer.get_index();
        let frame = &self.frames[ix];
        if frame.text.len() > 0 {
            let max_width = self.window.width() * 2.;
            // TODO: Figure out why the bottom is window.height() * 2.
            let max_height = self.window.height() * 2. - (self.font.height() as f32);
            let text_loc = na::Pnt2::new(x * max_width, y * max_height);
            let text_color = color.to_pnt3();
            
            self.window.draw_text(&*frame.text, &text_loc, &self.font, &text_color);
        }
    }
    
    /// Standard key handling, called by run.
    pub fn handle_events(&mut self) {
        for mut event in self.window.events().iter() {
            match event.value {
                WindowEvent::Key(key, _, glfw::Action::Release, _) => {
                    // Default to inhibiting, although this can be overridden
                    let mut inhibit = true;
                    match key {
                        Key::Q => {
                            self.window.close();
                            return;
                        },
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
    }
    
    /// Start the whole running sequence.
    pub fn run<F>(&mut self, mut update: F)
            where F: FnMut(&mut Parviewer, bool) {
        {
            // Set it to the first position, and then return the borrow of `self` for
            // the render function to use
            let ref frame = self.frames[0];
            self.nodes.update(frame, &mut self.palette);
        }
        
        let mut lastframe : isize = 0;
        while self.window.render_with_camera(&mut self.camera) {
            self.timer.incr();
            let ix = self.timer.get_index();
            
            let new_index = lastframe != (ix as isize);
            if new_index {
                let ref frame = self.frames[ix];
                self.nodes.update(frame, &mut self.palette);
                lastframe = ix as isize;
            }
            
            update(self, new_index);
            
            self.handle_events();
        };
    }
}
