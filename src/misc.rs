//! Miscellaneous functions for use in parview.

use kiss3d;
use na;
use std;
use toml;

use flate2::read::GzDecoder;
use kiss3d::window::Window;
use rand::random;
use serde::Deserialize;

use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::Path;

use objects;
use palette;

/// A random Vector3<f32>, with coordinates in (-0.5, 0.5)
pub fn rand_vec() -> na::Vector3<f32> {
    na::Vector3::new(random::<f32>(), random(), random()) - na::Vector3::new(0.5f32, 0.5f32, 0.5f32)
}

/// Inconsolata font
pub fn inconsolata() -> std::rc::Rc<kiss3d::text::Font> {
    let inconsolata_otf: &[u8] = include_bytes!("Inconsolata.otf");
    kiss3d::text::Font::from_bytes(inconsolata_otf).unwrap()
}

/// Deserialize a function from a `.json` or `.json.gz` file
pub fn deserialize_by_ext<T>(path: &Path) -> Result<T, Box<Error>>
where
    for<'de> T: Deserialize<'de>,
{
    let mut buf: io::BufReader<File> = io::BufReader::new(File::open(path)?);
    // let f = try!(File::open(path));

    //~ let coded = json::from_reader(&mut buf).unwrap();
    //~ let mut decoder =json::Decoder::new(coded);
    //~ let frames: Vec<Frame> = Decodable::decode(&mut decoder).unwrap();

    let ext: Option<&str> = path.extension().and_then(|s| s.to_str());

    let coded = match ext {
        Some("gz") => {
            let mut gzbuf = GzDecoder::new(buf);
            serde_json::de::from_reader(&mut gzbuf)?
        }
        _ => serde_json::de::from_reader(&mut buf)?,
    };
    Ok(coded)
}

/// Generate an example json file
pub fn generate_frame(path: &Path) -> Result<Vec<objects::Frame>, Box<Error>> {
    let spheres = (0..16)
        .map(|n| {
            let loc: na::Vector3<f32> = rand_vec();
            let s: f32 = random();
            let names = objects::ObjectID(vec![format!("{}", n / 4 + 1), format!("{}", n % 4 + 1)]);
            objects::Sphere {
                loc: (loc.x, loc.y, loc.z),
                diameter: s * 0.2,
                names: names,
            }
        })
        .collect();

    let f = objects::Frame {
        spheres: spheres,
        spherocylinders: vec![],
        text: String::new(),
    };

    let mut framevec: Vec<objects::Frame> = vec![];

    for i in 0usize..40usize {
        let mut f2 = objects::Frame {
            spheres: f
                .spheres
                .iter()
                .enumerate()
                .map(|(n, ref s)| {
                    let mut newr = s.diameter;
                    if n == 0 {
                        newr = random::<f32>() * 0.2f32;
                    }
                    let (sx, sy, sz) = s.loc;
                    let v = na::Vector3::new(sx, sy, sz) + (rand_vec() * 0.1f32);
                    objects::Sphere {
                        loc: (v[0], v[1], v[2]),
                        diameter: newr,
                        names: s.names.clone(),
                    }
                })
                .collect(),

            spherocylinders: vec![],
            text: format!("Frame {} with {} spheres", i, f.spheres.len()),
        };

        if i > 10 && i < 20 {
            let l = f2.spheres.len();
            f2.spheres.truncate(l - 8);
            f2.text = format!("Frame {} with {} spheres", i, f2.spheres.len());
        }
        framevec.push(f2);
    }

    let mut file = File::create(&path)?;

    serde_json::ser::to_writer_pretty(&mut file, &framevec)?;
    Ok(framevec)
}

/// Write a default palette, as an example to be used when creating a new one
pub fn generate_palette(path: &Path) -> Result<palette::Palette, Box<Error>> {
    let mut file: File = try!(File::create(path));
    let mut default_palette = palette::Palette::default();
    let _ = default_palette.assigned.insert(
        objects::ObjectID(vec!["A".into()]),
        palette::Color(255, 0, 0),
    );
    let _ = default_palette.assigned.insert(
        objects::ObjectID(vec!["B".into()]),
        palette::Color(0, 255, 0),
    );
    let s: String = toml::to_string_pretty(&default_palette)?;
    write!(file, "{}", s)?;
    Ok(default_palette)
}

/// Load from a file, using toml-rs and serialize
pub fn load_toml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, Box<Error>> {
    let mut file: File = try!(File::open(path));
    let mut s = String::new();
    let _ = try!(file.read_to_string(&mut s));
    Ok(toml::from_str(&*s)?)
}

/// Draw a cube in the current window. Cube will be a red skeleton.
pub fn draw_cube(window: &mut Window) -> kiss3d::scene::SceneNode {
    let rotations: Vec<Option<na::Vector3<f32>>> =
        vec![None, Some(na::Vector3::x()), Some(na::Vector3::z())];
    let translations = vec![(-0.5f32, -0.5), (-0.5, 0.5), (0.5, -0.5), (0.5, 0.5)];
    let translations: Vec<na::Translation3<f32>> = translations
        .iter()
        .map(|&(x, z)| na::Translation3::<f32>::new(x, 0.0, z))
        .collect();

    let mut cube = window.add_group();
    for rot in &rotations {
        for t in &translations {
            let mut caps = cube.add_capsule(0.01, 1.0);
            caps.append_translation(t.into());
            match *rot {
                Some(r) => {
                    caps.append_rotation(&na::UnitQuaternion::new(r * std::f32::consts::FRAC_PI_2))
                }
                None => {}
            }
            //caps.append_translation(&Vec3::new(-0.5, 0.0, -0.5));
        }
    }
    // TODO: this color should be configurable
    cube.set_color(1.0, 0.0, 0.0);

    cube
}

/// Turn an error into a print message.
pub fn err_print(err: &Error) {
    println!("Description: {}", err.description());
    println!("Debug version: {:?}", err);

    if let Some(e) = err.source() {
        println!("Cause.");
        err_print(e);
    }
}
