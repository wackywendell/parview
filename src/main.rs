/*!
# ParView
*/

#![feature(plugin)]
#![plugin(docopt_macros)]

#![feature(concat_idents)]

#![deny(non_camel_case_types)]
#![deny(unused_parens)]
#![deny(non_upper_case_globals)]
#![deny(unused_qualifications)]
#![deny(missing_docs)]
#![deny(unused_results)]

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
    -p, --palette FILE      Use palette file (toml file)
    --pitch ANGLE           Set initial pitch (degrees) [default: 90]
    --yaw ANGLE             Set initial yaw (degrees) [default: 0]
    --fov ANGLE             Set camera field-of-view angle (degrees) [default: 45]
    --width PIXELS          Set window width (pixels) [default: 600]
    --height PIXELS         Set window height, if different from width (pixels)
    --distance D            Set distance from box (L) [default: 2]
    --pauseloop FRAMES      When finished, pause for FRAMES frames, then loop.
                            By default, does not loop.

Arguments:
    <file>      json file representing the frames. json.gz also accepted, if the extension is \".gz\".
",
flag_pitch : f32,
flag_yaw : f32,
flag_fov : f32,
flag_distance : f32,
flag_width : u32,
flag_height : Option<u32>,
flag_pauseloop : Option<f32>,
flag_palette : Option<String>,
arg_file : Option<String>,
);

impl Args {
    /// Create a Config instance from these Args
    fn to_config(&self) -> Config {
        Config {
            pitch : self.flag_pitch,
            yaw : self.flag_yaw,
            fov : self.flag_fov,
            width : self.flag_width,
            height : self.flag_height.unwrap_or(self.flag_width),
            distance : self.flag_distance,
            pauseloop : self.flag_pauseloop,
            fps : 24.
        }
    }
}

fn run() -> Result<(), Box<std::error::Error>> {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    let config : Config = args.to_config();
    
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
                let p : Palette = try!(Palette::load(path));
                p
            }
        };
        (frames, palette)
    };
    
    // println!("config: {:?}", config);
    
    let mut viewer = try!(Parviewer::new(frames, palette, config));
    viewer.run();
    Ok(())
}

fn main() {
    run().unwrap();
}
