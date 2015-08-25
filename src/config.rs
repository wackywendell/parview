//! Serializable configuration for Parview.
use parviewer::Config;

/// Configuration to be loaded from the TOML file
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

impl TomlConfig {
    /// Convert to `parviewer::config` instance
    pub fn to_parviewer_config(&self) -> Config {
        Config {
            pitch : self.pitch,
            yaw : self.yaw,
            fov : self.fov,
            width : self.width,
            height : self.height.unwrap_or(self.width),
            distance : self.distance,
            pauseloop : self.pauseloop,
            rotate : self.rotate,
            framerate : self.framerate
        }
    }
}
