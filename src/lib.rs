//! Reader and writer for Warcraft III camera files, `war3map.w3c`: the
//! cameras a map's World Editor places (`gg_cam_*` in the map's script).
//!
//! ```no_run
//! use w3c_rs::{Layout, parse};
//!
//! let bytes = std::fs::read("war3map.w3c").unwrap();
//! let w3i = std::fs::read("war3map.w3i").ok();
//! let layout = w3i.as_deref().and_then(Layout::from_w3i).unwrap_or_default();
//! let file = parse(&bytes, layout).unwrap();
//! for camera in &file.cameras {
//!     println!("{}: {} {}", camera.name, camera.target_x, camera.target_y);
//! }
//! assert_eq!(file.to_bytes().unwrap(), bytes);
//! ```
//!
//! The format doesn't say which layout it has: since 1.31 every camera
//! carries a local pitch, yaw and roll, and which game a map is for is in
//! its `war3map.w3i` ([`Layout::from_w3i`]). Without it, [`parse_any`]
//! tries the older layout (1.27a and before, the default), then the newer.
//! Reading and writing are exact: `to_bytes` gives back the bytes `parse`
//! read. The format is described in the README.
//!
//! For people, a file has a text form ([`text`]): BNI, one section per
//! camera, that reads back to the same bytes.

pub mod text;

use parser_rs::bin::{Reader, Writer};
pub use parser_rs::{BinError, BinResult};

/// The format version this crate reads and writes: the only one there is.
pub const VERSION: u32 = 0;

/// Which fields a camera has: since 1.31 a local pitch, yaw and roll too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layout {
    /// Reign of Chaos to 1.30 (and 1.27a, the default when nothing says).
    #[default]
    Classic,
    /// 1.31 and later.
    Local,
}

impl Layout {
    /// The layout for a map whose `war3map.w3i` has format `version`:
    /// 28 (1.31) and later have the local rotation. Reign of Chaos writes
    /// 18, The Frozen Throne 25, 1.32 31.
    pub fn from_w3i_version(version: u32) -> Layout {
        if version >= 28 {
            Layout::Local
        } else {
            Layout::Classic
        }
    }

    /// The layout from a `war3map.w3i`'s bytes: its first four are its
    /// format version. `None` when it's shorter.
    pub fn from_w3i(w3i: &[u8]) -> Option<Layout> {
        let version = u32::from_le_bytes(w3i.get(..4)?.try_into().ok()?);
        Some(Layout::from_w3i_version(version))
    }

    /// The game versions it's for, in words.
    pub fn describe(self) -> &'static str {
        match self {
            Layout::Classic => "Reign of Chaos to 1.30",
            Layout::Local => "1.31 and later",
        }
    }
}

/// One camera, as World Editor's Camera Palette shows it.
#[derive(Debug, Clone)]
pub struct Camera {
    /// Where it looks: the target's X and Y, and the height over it.
    pub target_x: f32,
    pub target_y: f32,
    pub z_offset: f32,
    /// Degrees.
    pub rotation: f32,
    /// Degrees.
    pub angle_of_attack: f32,
    pub distance: f32,
    pub roll: f32,
    /// Degrees.
    pub field_of_view: f32,
    pub far_z: f32,
    pub near_z: f32,
    /// Since 1.31 ([`Layout::Local`]); 0 and not written before.
    pub local_pitch: f32,
    pub local_yaw: f32,
    pub local_roll: f32,
    /// `Camera 001`: the script names it `gg_cam_Camera_001`.
    pub name: String,
}

impl Camera {
    /// The numbers, in file order, with their names; the local rotation
    /// last.
    pub fn fields(&self) -> [(&'static str, f32); 13] {
        [
            ("target_x", self.target_x),
            ("target_y", self.target_y),
            ("z_offset", self.z_offset),
            ("rotation", self.rotation),
            ("angle_of_attack", self.angle_of_attack),
            ("distance", self.distance),
            ("roll", self.roll),
            ("field_of_view", self.field_of_view),
            ("far_z", self.far_z),
            ("near_z", self.near_z),
            ("local_pitch", self.local_pitch),
            ("local_yaw", self.local_yaw),
            ("local_roll", self.local_roll),
        ]
    }

    /// The number field called `name` ([`fields`](Self::fields)).
    pub fn field_mut(&mut self, name: &str) -> Option<&mut f32> {
        Some(match name {
            "target_x" => &mut self.target_x,
            "target_y" => &mut self.target_y,
            "z_offset" => &mut self.z_offset,
            "rotation" => &mut self.rotation,
            "angle_of_attack" => &mut self.angle_of_attack,
            "distance" => &mut self.distance,
            "roll" => &mut self.roll,
            "field_of_view" => &mut self.field_of_view,
            "far_z" => &mut self.far_z,
            "near_z" => &mut self.near_z,
            "local_pitch" => &mut self.local_pitch,
            "local_yaw" => &mut self.local_yaw,
            "local_roll" => &mut self.local_roll,
            _ => return None,
        })
    }

    /// A camera at `x`, `y` with World Editor's defaults: looking down at
    /// 304 degrees from 1650 away, a 70 degree field of view.
    pub fn new(name: impl Into<String>, x: f32, y: f32) -> Camera {
        Camera {
            target_x: x,
            target_y: y,
            z_offset: 0.0,
            rotation: 90.0,
            angle_of_attack: 304.0,
            distance: 1650.0,
            roll: 0.0,
            field_of_view: 70.0,
            far_z: 5000.0,
            near_z: 100.0,
            local_pitch: 0.0,
            local_yaw: 0.0,
            local_roll: 0.0,
            name: name.into(),
        }
    }
}

/// Numbers compare by their bits, so a NaN equals itself and equal values
/// write equal bytes.
impl PartialEq for Camera {
    fn eq(&self, other: &Camera) -> bool {
        self.name == other.name
            && self
                .fields()
                .iter()
                .zip(other.fields())
                .all(|(a, b)| a.1.to_bits() == b.1.to_bits())
    }
}

/// A whole `war3map.w3c`.
#[derive(Debug, Clone, PartialEq)]
pub struct Cameras {
    /// Always [`VERSION`].
    pub version: u32,
    pub layout: Layout,
    pub cameras: Vec<Camera>,
}

impl Cameras {
    pub fn new(layout: Layout) -> Cameras {
        Cameras {
            version: VERSION,
            layout,
            cameras: Vec::new(),
        }
    }

    /// The file's bytes, in its layout.
    pub fn to_bytes(&self) -> BinResult<Vec<u8>> {
        let mut w = Writer::new();
        if self.version != VERSION {
            return Err(BinError::new(
                0,
                format!("unknown w3c version {}", self.version),
            ));
        }
        w.u32(self.version);
        w.count(self.cameras.len())?;
        for c in &self.cameras {
            let fields = c.fields();
            let count = if self.layout == Layout::Local { 13 } else { 10 };
            for (_, v) in &fields[..count] {
                w.f32(*v);
            }
            w.cstr(&c.name)?;
        }
        Ok(w.into_bytes())
    }
}

/// The smallest a camera can be: ten numbers and an empty name.
const MIN_CAMERA: usize = 10 * 4 + 1;

/// Reads a `war3map.w3c` in `layout`. Every byte must be read: a file in
/// the other layout fails, at the first camera that doesn't fit or with
/// bytes left over.
pub fn parse(bytes: &[u8], layout: Layout) -> BinResult<Cameras> {
    let mut r = Reader::new(bytes);
    let version = r.u32()?;
    if version != VERSION {
        return Err(BinError::new(0, format!("unknown w3c version {version}")));
    }
    let cameras = r.list(MIN_CAMERA, |r| {
        let mut c = Camera::new(String::new(), 0.0, 0.0);
        for v in [
            &mut c.target_x,
            &mut c.target_y,
            &mut c.z_offset,
            &mut c.rotation,
            &mut c.angle_of_attack,
            &mut c.distance,
            &mut c.roll,
            &mut c.field_of_view,
            &mut c.far_z,
            &mut c.near_z,
        ] {
            *v = r.f32()?;
        }
        if layout == Layout::Local {
            c.local_pitch = r.f32()?;
            c.local_yaw = r.f32()?;
            c.local_roll = r.f32()?;
        } else {
            (c.local_pitch, c.local_yaw, c.local_roll) = (0.0, 0.0, 0.0);
        }
        c.name = r.cstr()?;
        Ok(c)
    })?;
    r.finish()?;
    Ok(Cameras {
        version,
        layout,
        cameras,
    })
}

/// Reads a `war3map.w3c` whose layout isn't known (no `war3map.w3i`):
/// the older one first, as 1.27a writes it, then the newer. The error is
/// the older layout's, when neither fits.
pub fn parse_any(bytes: &[u8]) -> BinResult<Cameras> {
    parse(bytes, Layout::Classic).or_else(|err| parse(bytes, Layout::Local).map_err(|_| err))
}

#[cfg(test)]
#[path = "lib.test.rs"]
mod tests;
