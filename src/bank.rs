use std::{
    borrow::Cow,
    mem::replace,
    sync::Arc,
    time::Duration,
};

use bevy_ecs::prelude::{Changed, Entity, Query, Res};
use bevy_internal::{core::Time, utils::HashMap};
use bevy_math::{Quat, Vec3};



use crate::{player::{Channel, Player, Scale}, skeleton::Channel as EChan};

#[derive(Debug, Clone, Default)]
pub struct AnimationBank {
    // Make sure vec is sorted
    animations: HashMap<String, Animation>,
    // whether loop and animation name
    setting_animation: Option<(bool, Cow<'static, str>)>,
    // Animation name and time of stop from startup if not looping
    current_animation: Option<(Cow<'static, str>, Option<Duration>)>,
    complete_animation: Option<Cow<'static, str>>,
}
impl AnimationBank {
    pub fn new() -> AnimationBank {
        Default::default()
    }
    pub fn start_animation<S: Into<Cow<'static, str>>>(&mut self, name: S, looping: bool) {
        self.setting_animation = Some((looping, name.into()));
    }
    pub fn active_animation(&self) -> Option<&str> {
        match &self.current_animation {
            Some((cow, _)) => Some(&**cow),
            None => None,
        }
    }
    pub fn check_complete(&self) -> Option<&str> {
        match &self.complete_animation {
            Some(cow) => Some(&**cow),
            None => None,
        }
    }
    pub fn add_animation(&mut self, name: &str, bone: Entity, channel: EChan) {
        if let Some(animation) = self.animations.get_mut(name) {
            animation.add_channel(bone, channel)
        } else {
            let time = match &channel {
                EChan::Translation(s) => s.iter().map(|l| l.len()).sum(),
                EChan::Rotation(s) => s.iter().map(|l| l.len()).sum(),
                EChan::Scale(s) => s.iter().map(|l| l.len()).sum(),
            };
            let duration = Duration::from_secs_f32(time);
            let mut anim = Animation {
                duration,
                bones: Vec::new()
            };
            anim.add_channel(bone, channel);
            self.animations.insert(name.to_string(), anim);
        }
    }
}

#[derive(Debug, Clone)]
struct Animation {
    duration: Duration,
    bones: Vec<BoneAnimation>,
}
impl Animation {
    fn add_channel(&mut self, entity: Entity, channel: EChan) {
        let bone_anim = match self.bones.binary_search_by_key(&entity, |a| a.entity) {
            Ok(i) => &mut self.bones[i],
            Err(i) =>{
                self.bones.insert(i, BoneAnimation {
                    entity,
                    translation: None,
                    rotation: None,
                    scale: None
                });
                &mut self.bones[i]
            }
        };
        bone_anim.add_channel(channel)
    }
}
#[derive(Debug, Clone)]
struct BoneAnimation {
    entity: Entity,
    translation: Option<Arc<Channel<Vec3>>>,
    rotation: Option<Arc<Channel<Quat>>>,
    scale: Option<Arc<Channel<Scale>>>,
}
impl BoneAnimation {
    fn add_channel(&mut self, channel: EChan) {
        match channel {
            EChan::Translation(s) => self.translation = Some(Arc::new(Channel::new(s))),
            EChan::Rotation(s) => self.rotation = Some(Arc::new(Channel::new(s))),
            EChan::Scale(s) => self.scale = Some(Arc::new(Channel::new(s))),
        }
    }
}

pub(crate) fn animation_starter(
    time: Res<Time>,
    banks: Query<&mut AnimationBank, Changed<AnimationBank>>,
    mut players: Query<(&mut Player<Vec3>, &mut Player<Quat>, &mut Player<Scale>)>,
) {
    banks.for_each_mut(|mut bank| {
        if let Some((looping, name)) = replace(&mut bank.setting_animation, None) {
            if let Some(animations) = bank.animations.get(&*name) {
                for animation in &animations.bones {
                    if let Ok((mut translation, mut rotation, mut scale)) = players.get_mut(animation.entity) {
                        match &animation.translation {
                            Some(c) => translation.set_channel(c, looping),
                            None => translation.kill(),
                        }
                        match &animation.rotation {
                            Some(c) => rotation.set_channel(c, looping),
                            None => rotation.kill(),
                        }
                        match &animation.scale {
                            Some(c) => scale.set_channel(c, looping),
                            None => scale.kill(),
                        }
                    }
                }
                let end_time = (!looping).then(|| time.time_since_startup() + animations.duration);
                let last_current =
                    replace(&mut bank.current_animation, Some((name, end_time))).map(|(n, _)| n);
                bank.complete_animation = last_current;
            }
        }
    });
}

pub(crate) fn animation_stopper(time: Res<Time>, banks: Query<&mut AnimationBank>) {
    let now = time.time_since_startup();
    banks.for_each_mut(|mut bank| {
        if let Some((_name, Some(end_time))) = &bank.current_animation {
            if now >= *end_time {
                if let Some((name, _)) = replace(&mut bank.current_animation, None) {
                    bank.complete_animation = Some(name)
                }
            }
        }
    })
}

pub(crate) fn animation_cleaner(mut banks: Query<&mut AnimationBank, Changed<AnimationBank>>) {
    let iter = banks.iter_mut()
        .filter(|bank| bank.complete_animation.is_some());
    for mut bank in iter {
        bank.complete_animation = None;
    }
}