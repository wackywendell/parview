/*!
# ParView
*/
#![feature(phase)]

#![deny(non_camel_case_types)]
#![deny(unnecessary_parens)]
#![deny(non_uppercase_statics)]
#![deny(unnecessary_qualification)]
#![deny(missing_doc)]
#![deny(unused_result)]
#![deny(unnecessary_typecast)]

#[phase(plugin)]
extern crate docopt_macros;
extern crate docopt;

extern crate serialize;
extern crate flate2;

extern crate "nalgebra" as na;
extern crate kiss3d;
extern crate parview;

use serialize::{json, Encodable, Decodable};
use flate2::reader::GzDecoder;
use std::rand::random;
use std::io::{File,BufferedWriter,BufferedReader};

use kiss3d::window::Window;

use parview::{Sphere,Frame,rand_vec};

/// Generate an example json file
pub fn generate_frame() {
    let f = Frame {
        spheres : Vec::from_fn(16, |_| {
            let loc : na::Vec3<f32> = rand_vec();
            let s : f32 = random();
            Sphere{loc:(loc.x, loc.y, loc.z), radius:s*0.2, color:None}
        })
            
    };
    
    let mut framevec : Vec<Frame> = vec!();
    
    for _ in range(0u,200u){
        let f2 = Frame {
            spheres : f.spheres.iter().map(|&s| {
				Sphere::new(s.x() + (rand_vec() * 0.1f32), s.radius, s.color)
            }).collect()
        };
        framevec.push(f2.clone());
    }
    
    let path = Path::new("test_frame.json");
    let mut file = BufferedWriter::new(File::create(&path));
    {
        let mut encoder = json::PrettyEncoder::new(&mut file);
        match framevec.encode(&mut encoder) {
            Ok(()) => (),
            Err(e) => fail!("json encoding error: {}", e)
        };
    }
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
                &Some(ref r) => {caps.append_rotation(&(r * std::f32::consts::FRAC_PI_2))},
                &None => {}
            }
            //caps.append_translation(&Vec3::new(-0.5, 0.0, -0.5));
        }
    }
    cube.set_color(1.0, 0.0, 0.0);
    
    cube
}

fn open_file(path : &Path) -> std::io::IoResult<Vec<Frame>> {
	let mut buf = BufferedReader::new(File::open(path));
	//~ let coded = json::from_reader(&mut buf).unwrap();
    //~ let mut decoder =json::Decoder::new(coded);
    //~ let frames: Vec<Frame> = Decodable::decode(&mut decoder).unwrap();
    
	let coded_opt = match path.extension(){
		Some(b"gz") => match GzDecoder::new(buf) {
				Err(e) => {return Err(e);}
				Ok(mut gzbuf) => {json::from_reader(&mut gzbuf)}
		},
		_ => {
			json::from_reader(&mut buf)
			}
	};
	
	let json_result = match coded_opt {
		Err(e) => {
			return Err(std::io::IoError{
				kind: std::io::InvalidInput,
				desc: "Parser Error",
				detail: Some(format!("{}", e))
			});
		}
		Ok(coded) => {
			let mut decoder =json::Decoder::new(coded);
			Decodable::decode(&mut decoder)
		}
	};
	
	match json_result {
		Ok(frames) => Ok(frames),
		Err(e) =>  Err(
			std::io::IoError{
				kind: std::io::InvalidInput,
				desc: "Decoder Error",
				detail: Some(format!("{}", e))
			}
		)
	}
}

docopt!(Args, "
Usage: parview [-h | --help] [-g] [<file>]

Options:
	[-h | --help]   Help and usage
	-g

Arguments:
    <file>     	json file representing the frames. json.gz also accepted, if the extension is \".gz\".
", 
	flag_g : bool, 
	arg_file : Option<String>)

/// Main entry point, now using test_frame.json
pub fn main() {
	let args: Args = docopt::FlagParser::parse().unwrap_or_else(|e| e.exit());
    if args.flag_g {
		generate_frame()
	}
    
    let path = Path::new(args.arg_file.unwrap_or("test_frame.json".to_string()));
    let frames = open_file(&path).unwrap();
    
    let cols = [(1.,1.,1.),
                (0.,0.,0.),
                (0.8941, 0.1020, 0.1098),
                (0.2157, 0.4941, 0.7216),
                (0.3020, 0.6863, 0.2902),
                (0.5961, 0.3059, 0.6392),
                (1.0000, 0.4980, 0.0000),
                (0.6510, 0.3373, 0.1569),
                (0.9686, 0.5059, 0.7490),
                (0.6000, 0.6000, 0.6000),
                (1.0000, 1.0000, 0.2000)];
    
    let ref f : Frame = frames[0];
    
    let mut window = Window::new("Kiss3d: draw_sphere");
	let _ = draw_cube(&mut window);

	let eye              = na::Pnt3::new(0.0f32, 0.0, 2.0);
	let at               = na::orig();
	let mut arc_ball     = kiss3d::camera::ArcBall::new(eye, at);

	//window.set_background_color(1.0, 1.0, 1.0);
	window.set_light(kiss3d::light::StickToCamera);
	window.set_framerate_limit(Some(20));
	
	
	let mut sphere_set = vec!();
	for (n, &sphere) 
			in f.spheres.iter().enumerate() {
		let mut s = window.add_sphere(sphere.radius);
		let (r,g,b) = match sphere.color {
			Some((r,g,b)) => (r as f32 / 255., g as f32 / 255., b as f32 / 255.),
			None => cols[n % cols.len()]
		};
		s.set_color(r,g,b);
		s.append_translation(&sphere.x());
		sphere_set.push(s);
	}
	
	let mut t = 0;
	
	let font = kiss3d::text::Font::new(&Path::new("/usr/share/fonts/OTF/Inconsolata.otf"), 120);
	
	while window.render_with_camera(&mut arc_ball) {
	// for _ in window.iter_with_camera(arc_ball) {
		t += 1;
		
		let i = (t / 10) % frames.len();
		window.draw_text(format!("t = {}", i).as_slice(), 
			&na::orig(), &font, &na::Pnt3::new(0.0, 1.0, 1.0));
		
		if t % 10 == 0 {
			let ref frame = frames[i];
			for (&sphere, s)
					in frame.spheres.iter().zip(sphere_set.iter_mut()) {
				s.set_local_translation(sphere.x());
			}
		}
	};
}
