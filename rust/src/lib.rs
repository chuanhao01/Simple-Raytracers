pub mod camera;
pub mod cli;
pub mod helper;
pub mod hittable;
pub mod materials;
pub mod objects;
pub mod ray;
pub mod textures;
pub mod vec3;

pub use camera::{Camera, CameraParams};
pub use cli::Cli;
pub use helper::{color_to_rgb, rgb_to_color, Interval};
pub use hittable::{HitRecord, Hittable, HittableWithBBox, HittablesList, AABB, BVH};
pub use materials::{Dielectric, Diffuse, Lambertain, Materials, Metal, Scattered};
pub use objects::{construct_planar_quad_box, Disk, Quad, Rotation, Sphere, Translation, Triangle};
pub use ray::Ray;
pub use textures::{CheckeredTexture, ColorTexture, Image, SolidColor, SpatialCheckeredTexture};
pub use vec3::{Vec3, Vec3Axis};
