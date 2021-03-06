//! # ParView
//!
#![deny(non_camel_case_types)]
#![deny(unused_parens)]
#![deny(non_upper_case_globals)]
#![deny(unused_qualifications)]
#![deny(missing_docs)]
#![deny(unused_results)]

extern crate docopt;
extern crate kiss3d;
extern crate mpeg_encoder;
extern crate serde;

extern crate parview;

use std::f32::consts::PI;
use std::path::Path;

use kiss3d::event::{Action, Key, WindowEvent};
use mpeg_encoder::Encoder;
use serde::Deserialize;

use parview::{misc, Color, Config, Frame, Palette, Parviewer, TomlConfig, EPSILON};

// Write the Docopt usage string.
const USAGE: &str = "
Usage: pvrecord [options] [--] <particlefile> <moviefile>

Options:
    -h, --help              Help and usage
    -p, --palette FILE      Use palette file (toml file), instead of default.
    -c, --config FILE       Use config file (toml file), instead of default.


Arguments:
    <file>      json file representing the frames. json.gz also accepted, if
                the extension is \".gz\".
";

#[derive(Deserialize)]
struct Args {
    flag_palette: Option<String>,
    flag_config: Option<String>,
    arg_particlefile: String,
    arg_moviefile: String,
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let docopt = docopt::Docopt::new(USAGE)?;
    let args: Args = docopt.parse()?.deserialize()?;
    let toml_config: TomlConfig = match args.flag_config {
        None => Default::default(),
        Some(ref fname) => {
            let path: &Path = Path::new(&fname[..]);
            misc::load_toml::<TomlConfig>(path)?
        }
    };

    let framerate = toml_config.framerate;
    let config: Config = toml_config.to_parviewer_config();

    let fname: &str = &args.arg_particlefile;
    let path: &Path = Path::new(fname);

    let frames: Vec<Frame> = misc::deserialize_by_ext(path)?;
    let palette: Palette = match args.flag_palette {
        None => Default::default(),
        Some(fname) => {
            let palette_path: &Path = Path::new(&fname[..]);
            misc::load_toml::<Palette>(palette_path)?
        }
    };

    // println!("config: {:?}", config);

    let mut viewer = Parviewer::new(frames, palette, config)?;
    let _ = viewer.timer.at_least(toml_config.fps);
    // Record as fast as possible
    viewer.window.set_framerate_limit(Some(framerate as u64));
    let text_color = Color(255, 255, 255);
    let width = viewer.window.width();
    let height = viewer.window.height();
    let mut buf: Vec<u8> = Vec::with_capacity((width * height) as usize);

    let mut encoder = Encoder::new_with_params(
        &args.arg_moviefile,
        viewer.window.width() as usize,
        viewer.window.height() as usize,
        None,                          // bit_rate
        Some((1, framerate as usize)), // time base
        None,
        None,
        None,
    );

    println!(
        "Sizes: {}, {}",
        viewer.window.width() as usize,
        viewer.window.height() as usize
    );

    let mut lastix = 0;

    viewer.run(|viewer, _| {
        if toml_config.rotate.abs() > EPSILON {
            let new_yaw = viewer.camera.yaw() + (toml_config.rotate * PI / 180.);
            viewer.camera.set_yaw(new_yaw);
        }

        let ix = viewer.timer.get_index();
        if ix < lastix {
            viewer.window.close();
            return;
        };

        lastix = ix;

        for mut event in viewer.window.events().iter() {
            match event.value {
                WindowEvent::Key(key, Action::Release, _) => {
                    // Default to inhibiting, although this can be overridden
                    let inhibit = true;
                    if let Key::Q = key {
                        viewer.window.close();
                        return;
                    }
                    // ignore all other keys
                    event.inhibited = inhibit;
                }
                WindowEvent::CursorPos(_, _, _) => {
                    // ignore drag events
                    event.inhibited = true;
                }
                _ => {}
            }
        }

        viewer.draw_frame_text(0., 0., text_color);
        viewer.window.snap(&mut buf);
        encoder.encode_rgb(
            viewer.window.width() as usize,
            viewer.window.height() as usize,
            &buf,
            false,
        );

        let frames_per_tick = viewer.timer.get_dt() / framerate;
        let total = viewer
            .timer
            .total_loop_time()
            .map(|n| format!("{}", (n / frames_per_tick + 0.5) as usize))
            .unwrap_or_else(|| "?".into());

        let title = format!(
            "Parviewer ({} / {})",
            ((viewer.timer.get_time() / frames_per_tick) + 0.5) as usize,
            total
        );
        viewer.window.set_title(&title);
        // println!("{}", title);
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
