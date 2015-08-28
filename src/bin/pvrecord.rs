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

extern crate rustc_serialize;
extern crate docopt;
extern crate kiss3d_recording;
extern crate glfw;

extern crate parview;

use std::path::Path;

use kiss3d_recording::Recorder;
use glfw::{WindowEvent,Key};

use parview::{misc,Palette,Color,Config,TomlConfig,Frame,Parviewer};

// Write the Docopt usage string.
docopt!(Args derive Debug, "
Usage: pvrecord [options] [--] <particlefile> <moviefile>

Options:
    -h, --help              Help and usage
    -p, --palette FILE      Use palette file (toml file), instead of default.
    -c, --config FILE       Use config file (toml file), instead of default.
    

Arguments:
    <file>      json file representing the frames. json.gz also accepted, if the extension is \".gz\".
",
flag_palette : Option<String>,
flag_config : Option<String>,
arg_particlefile : String,
arg_moviefile : String,
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
    
    let framerate = toml_config.framerate;
    let config : Config = toml_config.to_parviewer_config();
    
    let fname : &str = &args.arg_particlefile;
    let path : &Path = Path::new(fname);
    
    let frames : Vec<Frame> = try!(misc::deserialize_by_ext(path));
    let palette : Palette = match args.flag_palette {
        None => Default::default(),
        Some(fname) => {
            let path : &Path = Path::new(&fname[..]);
            try!(misc::load_toml::<Palette>(path))
        }
    };
    
    // println!("config: {:?}", config);
    
    let mut viewer = try!(Parviewer::new(frames, palette, config));
    let _ = viewer.timer.at_least(toml_config.fps);
    // Record as fast as possible
    viewer.window.set_framerate_limit(None);
    let text_color = Color(255, 255, 255);
    
    let mut recorder = Recorder::new_with_params(
        &args.arg_moviefile,
        viewer.window.width()  as usize,
        viewer.window.height() as usize,
        None, // bit_rate
        Some((1, framerate as usize)), //time base
        None, None, None
    );
    println!("Sizes: {}, {}", viewer.window.width() as usize,
    viewer.window.height() as usize);
    
    let mut lastix = 0;
    
    viewer.run(|viewer, _| {
        let ix = viewer.timer.get_index();
        if ix < lastix {
            viewer.window.close();
            return;
        };
        
        lastix = ix;
        
        for mut event in viewer.window.events().iter() {
            match event.value {
                WindowEvent::Key(key, _, glfw::Action::Release, _) => {
                    // Default to inhibiting, although this can be overridden
                    let inhibit = true;
                    match key {
                        Key::Q => {
                            viewer.window.close();
                            return;
                        },
                        _ => {}
                    }
                    // ignore all other keys
                    event.inhibited = inhibit;
                },
                glfw::WindowEvent::CursorPos(_, _) => {
                    // ignore drag events
                    event.inhibited = true;
                },
                _ => {}
            }
        }
        
        viewer.draw_frame_text(0., 0., text_color);
        recorder.snap(&mut viewer.window);
        
        let frames_per_tick = viewer.timer.get_dt() / framerate;
        let total = viewer.timer.total_loop_time().map(
            |n| format!("{}", (n / frames_per_tick + 0.5) as usize)
        ).unwrap_or("?".into());
        
        let title = format!("Parviewer ({} / {})", 
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