/*!
# ParView
*/

#![feature(plugin)]
#![feature(custom_derive)]
#![plugin(docopt_macros)]

#![feature(concat_idents)]

#![deny(non_camel_case_types)]
#![deny(unused_parens)]
#![deny(non_upper_case_globals)]
#![deny(unused_qualifications)]
#![deny(missing_docs)]
#![deny(unused_results)]
// Don't warn about dead code in this module when testing, its annoying.
#![cfg_attr(test, allow(dead_code))]

extern crate rustc_serialize;
extern crate docopt;

extern crate parview;

use std::path::Path;

use parview::{misc,Palette,Config,Frame,Parviewer};

// Write the Docopt usage string.
docopt!(Args derive Debug, "
Usage: parview [options] [--] [<file>]

Options:
    -h, --help              Help and usage
    -g, --generate          Generate test_frames.json
    -p, --palette FILE      Use palette file (toml file), instead of default.
    -c, --config FILE       Use config file (toml file), instead of default.
    

Arguments:
    <file>      json file representing the frames. json.gz also accepted, if the extension is \".gz\".
",
flag_palette : Option<String>,
flag_config : Option<String>,
arg_file : Option<String>,
);

/// Configuration to be loaded from the TOML file
#[allow(unused_attributes)]
#[derive(RustcDecodable,RustcEncodable)]
#[derive(Serialize,Deserialize)]
pub struct TomlConfig {
    /// Set initial pitch (degrees) [default: 90]
    pub pitch : f32,
    ///Set initial yaw (degrees) [default: 0]
    pub yaw : f32,
    /// Set camera field-of-view angle (degrees) [default: 45]
    pub fov : f32,
    /// Set distance between camera and box (L) [default: 2]
    pub distance : f32,
    /// Set window width (pixels) [default: 600]
    pub width : u32,
    /// Set window height, if different from width (pixels)
    pub height : Option<u32>,
    /// When finished, pause for FRAMES frames, then loop. None does not loop. [default: None]
    pub pauseloop : Option<f32>,
    /// Continuous box rotation (angle / frame) [default: 0]
    pub rotate : f32,
    /// Framerate: sets maximum framerate of drawing, independent of actual frames. [default: 24]
    pub framerate : f32,
    /// Rate of drawing frames. [default: 2.0]
    pub fps : f32
}

impl Default for TomlConfig {
    fn default() -> Self {
        TomlConfig {
            pitch : 90.,
            yaw : 0.,
            fov : 45.,
            distance : 2.,
            width : 800,
            height : None,
            pauseloop : None,
            rotate : 0.0,
            fps : 2.0,
            framerate : 24.0
        }
    }
}
    
/// Create a Config instance from these Args
#[allow(unused_variables)]
fn args_toml_to_config(args: &Args, toml_config: &TomlConfig) -> Config {
    Config {
        pitch : toml_config.pitch,
        yaw : toml_config.yaw,
        fov : toml_config.fov,
        width : toml_config.width,
        height : toml_config.height.unwrap_or(toml_config.width),
        distance : toml_config.distance,
        pauseloop : toml_config.pauseloop,
        rotate : toml_config.rotate,
        framerate : toml_config.framerate
    }
}

fn err_print(err : &std::error::Error) {
    println!("Description: {}", err.description());
    println!("Debug version: {:?}", err);
    
    if let Some(e) = err.cause() {
        println!("Cause.");
        err_print(e);
    }
}

fn run() -> Result<(), Box<std::error::Error>> {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    let toml_config : TomlConfig = match args.flag_config {
        None => Default::default(),
        Some(ref fname) => {
            let path : &Path = Path::new(&fname[..]);
            try!(misc::load_toml::<TomlConfig>(path))
        }
    };
    
    let config : Config = args_toml_to_config(&args, &toml_config);
    
    let fname : &str = match args.arg_file {
        Some(ref s) => s,
        None => "test_frame.json"
    };
    let path : &Path = Path::new(fname);
    
    let (frames, palette) : (Vec<Frame>, Palette) = if args.flag_generate {
        let frames = try!(misc::generate_frame(path));
        let palette = match args.flag_palette {
            None => Default::default(),
            Some(fname) => {
                let path : &Path = Path::new(&fname[..]);
                try!(misc::generate_palette(path))
            }
        };
        (frames, palette)
    } else {
        let frames = try!(misc::deserialize_by_ext(path));
        let palette = match args.flag_palette {
            None => Default::default(),
            Some(fname) => {
                let path : &Path = Path::new(&fname[..]);
                try!(misc::load_toml::<Palette>(path))
            }
        };
        (frames, palette)
    };
    
    // println!("config: {:?}", config);
    
    let mut viewer = try!(Parviewer::new(frames, palette, config));
    let _ = viewer.timer.at_least(toml_config.fps);
    viewer.run();
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        println!("ERROR.");
        
        err_print(&*err);
    }
}
