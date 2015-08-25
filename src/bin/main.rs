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

use parview::{misc,Palette,Config,TomlConfig,Frame,Parviewer};

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

fn run() -> Result<(), Box<std::error::Error>> {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    let toml_config : TomlConfig = match args.flag_config {
        None => Default::default(),
        Some(ref fname) => {
            let path : &Path = Path::new(&fname[..]);
            try!(misc::load_toml::<TomlConfig>(path))
        }
    };
    
    let config : Config = toml_config.to_parviewer_config();
    
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
        
        misc::err_print(&*err);
    }
}
