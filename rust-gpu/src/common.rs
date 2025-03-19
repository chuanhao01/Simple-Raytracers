use std::ops::{Add, AddAssign, Mul, Neg, Sub};

use bytemuck::{Pod, Zeroable};

use crate::gpu_buffer::CameraUniform;

// Common vec used both in rust and memmory safe for buffer use in wgsl
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Vec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vec3f {
    pub fn empty() -> Self {
        Vec3f {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        }
    }
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3f { x, y, z }
    }
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        Vec3f::new(self.x / len, self.y / len, self.z / len)
    }
    /// Calculates the cross product of 2 vectors
    pub fn cross(u: &Vec3f, v: &Vec3f) -> Vec3f {
        Vec3f::new(
            u.y * v.z - u.z * v.y,
            u.z * v.x - u.x * v.z,
            u.x * v.y - u.y * v.x,
        )
    }
}
impl Neg for Vec3f {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Vec3f::new(-self.x, -self.y, -self.z)
    }
}
impl AddAssign for Vec3f {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}
impl Add for Vec3f {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let mut v = self;
        v += rhs;
        v
    }
}
impl Sub for Vec3f {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut v = self;
        v += -rhs;
        v
    }
}
impl Mul<Vec3f> for f32 {
    type Output = Vec3f;
    fn mul(self, rhs: Vec3f) -> Self::Output {
        Vec3f::new(rhs.x * self, rhs.y * self, rhs.z * self)
    }
}

pub struct Camera {
    // Cam params
    look_from: Vec3f,
    look_at: Vec3f,
    v_up: Vec3f,
    theta: f32,
    // Basis Vectors, used for moving around later on
    u: Vec3f, // Right
    v: Vec3f, // Up
    w: Vec3f, // look from to look at (straight)

    // Factor to scale u, v, w vectors when using movement keys
    u_factor: f32,
    v_factor: f32,
    w_factor: f32,
}
impl Camera {
    pub fn from_init_configs(init_config: &InitConfig) -> Self {
        Self::new(
            init_config.look_from,
            init_config.look_at,
            init_config.v_up,
            init_config.camera_theta,
            0.1,
        )
    }

    // factor: used for all u, v and w factors for now
    pub fn new(look_from: Vec3f, look_at: Vec3f, v_up: Vec3f, theta: f32, factor: f32) -> Self {
        let _w = look_at - look_from;
        let w = _w.normalize();
        let u = Vec3f::cross(&v_up, &w).normalize();
        let v = Vec3f::cross(&w, &u);

        Self {
            look_from,
            look_at,
            v_up,
            theta,
            u,
            v,
            w,
            u_factor: factor,
            v_factor: factor,
            w_factor: factor,
        }
    }
    pub fn move_right(&mut self) {
        self.look_from += self.u_factor * self.u;
    }
    pub fn move_left(&mut self) {
        self.look_from += -self.u_factor * self.u;
    }
    pub fn move_forward(&mut self) {
        self.look_from += self.w_factor * self.w;
    }
    pub fn move_backward(&mut self) {
        self.look_from += -self.w_factor * self.w;
    }
    pub fn move_up(&mut self) {
        self.look_from += self.v_factor * self.v;
    }
    pub fn move_down(&mut self) {
        self.look_from += -self.v_factor * self.v;
    }

    pub fn to_camera_uniform(&self) -> CameraUniform {
        CameraUniform::new(self.look_from, self.look_at, self.v_up, self.theta)
    }
}

pub struct InitParam {
    pub vp_width: u32,
    pub aspect_ratio: f32,
    pub camera_theta: f32, // In Degrees
    pub look_at: Vec3f,
    pub look_from: Vec3f,
    pub v_up: Vec3f,
}
impl Default for InitParam {
    fn default() -> Self {
        Self {
            vp_width: 1000u32,
            aspect_ratio: 16f32 / 9f32,
            camera_theta: 70f32,
            look_at: Vec3f::new(0f32, 0f32, -1f32),
            look_from: Vec3f::new(0f32, 0f32, 0f32),
            v_up: Vec3f::new(0f32, 1f32, 0f32),
        }
    }
}

#[derive(Clone)]
pub struct InitConfig {
    pub vp_width: u32,
    pub vp_height: u32,
    pub camera_theta: f32, // In Degrees
    pub look_at: Vec3f,
    pub look_from: Vec3f,
    pub v_up: Vec3f,
}
impl InitConfig {
    pub fn new(init_param: InitParam) -> Self {
        Self {
            vp_width: init_param.vp_width,
            vp_height: (init_param.vp_width as f32 / init_param.aspect_ratio) as u32,
            camera_theta: init_param.camera_theta,
            look_at: init_param.look_at,
            look_from: init_param.look_from,
            v_up: init_param.v_up,
        }
    }
}
