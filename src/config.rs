//! Serializable configuration for Parview.
use parviewer::Config;
use serde;
use rustc_serialize;
use toml;

/// Configuration to be loaded from the TOML file
/// TomlConfig is deserialized by first deserializing one of these, and then 
/// filling in with defaults
#[derive(RustcDecodable)]
#[derive(Deserialize)]
struct TomlConfigOpt {
    pub pitch: Option<f32>,
    pub yaw: Option<f32>,
    pub fov: Option<f32>,
    pub distance: Option<f32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub pauseloop: Option<f32>,
    pub rotate: Option<f32>,
    pub framerate: Option<f32>,
    pub fps: Option<f32>,
}

/// Configuration to be loaded from the TOML file
#[derive(RustcEncodable,Serialize,Debug,PartialEq,PartialOrd)]
pub struct TomlConfig {
    /// Set initial pitch (degrees) [default: 90]
    pub pitch: f32,
    ///Set initial yaw (degrees) [default: 0]
    pub yaw: f32,
    /// Set camera field-of-view angle (degrees) [default: 45]
    pub fov: f32,
    /// Set distance between camera and box (L) [default: 2]
    pub distance: f32,
    /// Set window width (pixels) [default: 600]
    pub width: u32,
    /// Set window height, if different from width (pixels)
    pub height: Option<u32>,
    /// When finished, pause for FRAMES frames, then loop. None does not loop. [default: None]
    pub pauseloop: Option<f32>,
    /// Continuous box rotation (angle / frame) [default: 0]
    pub rotate: f32,
    /// Framerate: sets maximum framerate of drawing, independent of actual frames. [default: 24]
    pub framerate: f32,
    /// Rate of drawing frames. [default: 2.0]
    pub fps: f32,
}

impl Default for TomlConfig {
    fn default() -> Self {
        TomlConfig {
            pitch: 90.,
            yaw: 0.,
            fov: 45.,
            distance: 2.,
            width: 800,
            height: None,
            pauseloop: None,
            rotate: 0.0,
            fps: 2.0,
            framerate: 24.0,
        }
    }
}

impl serde::Deserialize for TomlConfig {
    fn deserialize<D: serde::Deserializer>(d: &mut D) -> Result<Self, D::Error> {
        TomlConfigOpt::deserialize(d).map(|tco| Self::from(tco))
    }
}

impl rustc_serialize::Decodable for TomlConfig {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Self, D::Error> {
        TomlConfigOpt::decode(d).map(|tco| Self::from(tco))
    }
}

impl TomlConfig {
    fn from(tco : TomlConfigOpt) -> Self {
        let default_opts = Self::default();
        TomlConfig {
            pitch : tco.pitch.unwrap_or(default_opts.pitch),
            yaw : tco.yaw.unwrap_or(default_opts.yaw),
            fov : tco.fov.unwrap_or(default_opts.fov),
            distance : tco.distance.unwrap_or(default_opts.distance),
            width : tco.width.unwrap_or(default_opts.width),
            height : tco.height,
            pauseloop : tco.pauseloop,
            rotate : tco.rotate.unwrap_or(default_opts.rotate),
            fps : tco.fps.unwrap_or(default_opts.fps),
            framerate : tco.framerate.unwrap_or(default_opts.framerate)
        }
    }
    
    /// Convert to `parviewer::config` instance
    pub fn to_parviewer_config(&self) -> Config {
        Config {
            pitch: self.pitch,
            yaw: self.yaw,
            fov: self.fov,
            width: self.width,
            height: self.height.unwrap_or(self.width),
            distance: self.distance,
            pauseloop: self.pauseloop,
            framerate: self.framerate,
        }
    }
}



#[test]
fn config_toml_empty_string() {
    let c : TomlConfig = TomlConfig::default();
    let c2 : TomlConfig = toml::decode_str("").unwrap();
    assert_eq!(c, c2);
}
