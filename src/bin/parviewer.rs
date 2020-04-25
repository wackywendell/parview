/*!
# ParView
*/
#![deny(non_camel_case_types)]
#![deny(unused_parens)]
#![deny(non_upper_case_globals)]
#![deny(unused_qualifications)]
#![deny(missing_docs)]
#![deny(unused_results)]

extern crate docopt;
extern crate parview;
extern crate serde;

use docopt::Docopt;
use serde::Deserialize;

use std::error::Error;
use std::path::Path;

use parview::{misc, Color, Config, Frame, Palette, Parviewer, TomlConfig, EPSILON};
use std::f32::consts::PI;

// Write the Docopt usage string.
const USAGE: &str = "
Usage: parview [options] [--] [<file>]

Options:
    -h, --help              Help and usage
    -g, --generate          Generate test_frames.json
    -p, --palette FILE      Use palette file (toml file), instead of default.
    -c, --config FILE       Use config file (toml file), instead of default.


Arguments:
    <file>      json file representing the frames. json.gz also accepted,
                if the extension is \".gz\".
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_palette: Option<String>,
    flag_config: Option<String>,
    flag_generate: bool,
    arg_file: Option<String>,
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    let toml_config: TomlConfig = match args.flag_config {
        None => Default::default(),
        Some(ref fname) => {
            let path: &Path = Path::new(&fname[..]);
            misc::load_toml::<TomlConfig>(path)?
        }
    };

    let config: Config = toml_config.to_parviewer_config();

    let fname: &str = match args.arg_file {
        Some(ref s) => s,
        None => "test_frame.json",
    };
    let path: &Path = Path::new(fname);

    let (frames, palette): (Vec<Frame>, Palette) = if args.flag_generate {
        let frames = misc::generate_frame(path)?;
        let palette = match args.flag_palette {
            None => Default::default(),
            Some(fname) => {
                let palette_path: &Path = Path::new(&fname[..]);
                misc::generate_palette(palette_path)?
            }
        };
        (frames, palette)
    } else {
        let frames = misc::deserialize_by_ext(path)?;
        let palette = match args.flag_palette {
            None => Default::default(),
            Some(fname) => {
                let palette_path: &Path = Path::new(&fname[..]);
                misc::load_toml::<Palette>(palette_path)?
            }
        };
        (frames, palette)
    };

    // println!("config: {:?}", config);

    let mut viewer = Parviewer::new(frames, palette, config)?;
    let _ = viewer.timer.at_least(toml_config.fps);
    let text_color = Color(255, 255, 255);

    viewer.run(|viewer, _| {
        if toml_config.rotate.abs() > EPSILON {
            let new_yaw = viewer.camera.yaw() + (toml_config.rotate * PI / 180.);
            viewer.camera.set_yaw(new_yaw);
        }

        viewer.draw_frame_text(0., 0., text_color);

        let dt = viewer.timer.get_dt();
        let dt_text = if dt >= 0.6 || dt.abs() < 1e-6 || dt <= -0.6 {
            format!("{}", dt)
        } else if dt > 0. {
            format!("1/{}", 1. / dt)
        } else {
            format!("-1/{}", -1. / dt)
        };

        let text = format!(
            "t:{:6.2}, dt:{}, coloring: {}",
            viewer.timer.get_time(),
            dt_text,
            viewer.palette.partials_string()
        );

        viewer.draw_text(&*text, 0., 1., text_color);
    });
    Ok(())
}

/// The main entry point.
pub fn main() {
    if let Err(err) = run() {
        println!("ERROR.");

        misc::err_print(&*err);
    }
}
