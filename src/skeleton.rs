use std::collections::HashMap;

use bevy_ecs::{prelude::{Commands, Entity}, system::EntityCommands};
use bevy_internal::{prelude::{BuildChildren, DespawnRecursiveExt, Handle, Transform}};
use bevy_math::{Quat, Vec3};
use bevy_pbr::{PbrBundle, prelude::StandardMaterial};
use bevy_render::mesh::Mesh;

use crate::{bank::AnimationBank, interpolation::Line, player::{PlayerBundle, Scale}};

#[derive(Debug, Default)]
pub struct Bone {
    pub id: usize,
    pub children: Vec<Bone>,
    pub mesh: Option<Drawable>,
    pub transform: Transform
}
impl Bone {
    // maybe record start so no empty runs at the beginning in the future?
    fn instance(&self, commands: &mut EntityCommands) -> Vec<Option<Entity>> {
        let mut buff = Vec::new();
        self.instance_record(commands, &mut buff);
        let max = match buff.iter().map(|(idx, _e)| *idx).max() {
            Some(a) => a,
            None => return Vec::new()
        };
        let mut new_buff = vec![None; max+1];
        for (idx, e) in buff {
            new_buff[idx] = Some(e)
        }
        new_buff
    }
    fn instance_record(&self, commands: &mut EntityCommands, record: &mut Vec<(usize, Entity)>) {
        record.push((self.id, commands.id()));
        commands.with_children(|parent| {
            for bone in &self.children {
                let mut ec = parent.spawn();
                bone.instance_record(&mut ec, record)
            }
        })
        .insert_bundle(PlayerBundle::new());
        if let Some(drawable) = &self.mesh {
            commands.insert_bundle(PbrBundle{
                mesh: drawable.mesh.to_owned(),
                material: drawable.material.to_owned(),
                transform: self.transform,
                ..Default::default()
            });
        } else {
            commands.insert(self.transform);
        }
    }
}

#[derive(Debug, Default)]
pub struct Drawable {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>
}

pub struct Animations {
    root: Bone,
    animations: HashMap<String, Vec<(usize, Channel)>>
}
impl Animations {
    pub fn new(root: Bone) -> Animations {
        Animations {
            root,
            animations: HashMap::new()
        }
    }
    pub fn add_animation(&mut self, name: String, channels: Vec<(usize, Channel)>) {
        self.animations.insert(name, channels);
    }
    pub fn instance(self, mut commands: Commands) -> Result<Entity, ()> {
        let mut root_coms = commands.spawn();
        let id = root_coms.id();
        let bone_ids = self.root.instance(&mut root_coms);
        let mut bank = AnimationBank::new();
        for (name, animation) in self.animations {
            for (idx, channel) in animation {
                let id = match bone_ids.get(idx) {
                    Some(&Some(e)) => e,
                    _ => {
                        root_coms.despawn_recursive();
                        return Err(())
                    }
                };
                bank.add_animation(&name, id, channel);
            }
        }
        Ok(id)
    }
}

pub enum Channel {
    Translation(Vec<Line<Vec3>>),
    Rotation(Vec<Line<Quat>>),
    Scale(Vec<Line<Scale>>)
}