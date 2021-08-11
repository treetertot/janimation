use std::collections::HashMap;

use bevy_internal::{prelude::{Handle, Transform}};
use bevy_math::{Quat, Vec3};
use bevy_pbr::prelude::StandardMaterial;
use bevy_render::mesh::Mesh;

#[derive(Debug, Default)]
pub struct Bone {
    pub id: u32,
    pub children: Vec<Bone>,
    pub mesh: Option<Drawable>,
    pub transform: Transform,
    pub animate: bool
}

#[derive(Debug, Default)]
pub struct Drawable {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>
}

pub struct Animations {
    root: Bone,
    animations: HashMap<String, Vec<(u32, Channel)>>
}
impl Animations {
    pub fn new(root: Bone) -> Animations {
        Animations {
            root,
            animations: HashMap::new()
        }
    }
    pub fn add_animation(&mut self, name: String, channels: Vec<(u32, Channel)>) {
        self.animations.insert(name, channels);
    }
}

pub enum Channel {
    Translation(Vec<Slide<Vec3>>),
    Rotation(Vec<Slide<Quat>>),
    Scale(Vec<Slide<Vec3>>)
}

pub struct Slide<T> {
    start: T,
    end: T,
    duration: f32
}