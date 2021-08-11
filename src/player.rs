use std::{
    ops::{Add, Mul},
    sync::Arc,
};

use crate::interpolation::Line;

use bevy_ecs::prelude::*;
use bevy_internal::prelude::*;
use bevy_math::Vec3;

#[derive(Debug, Clone)]
pub struct Channel<T> {
    lines: Vec<Line<T>>,
    duration: f32,
}
impl<T> Channel<T> {
    pub(crate) fn new(lines: Vec<Line<T>>, duration: f32) -> Self {
        Channel { lines, duration }
    }
}

// Maybe add a chache that refills when empty?
//add looping
pub struct Player<T> {
    looping: bool,
    channel: Option<Arc<Channel<T>>>,
    idx: usize,
    time: f32,
}
impl<T: Clone + Add<T, Output = T> + Mul<f32, Output = T>> Player<T> {
    fn play(&mut self, delta: f32) -> Option<T> {
        let (length, channel) = match &self.channel {
            Some(a) => (a.duration, &a.lines[..]),
            None => return None,
        };
        self.time += delta;
        loop {
            let line = match channel.get(self.idx) {
                Some(l) => l,
                None if self.looping => {
                    self.idx = 0;
                    self.time = self.time - length;
                    channel.get(self.idx)?
                }
                None => {
                    self.channel = None;
                    return None;
                }
            };
            match line.sample(self.time) {
                Ok(v) => return Some(v),
                Err(new_time) => {
                    self.time = new_time;
                    self.idx += 1
                }
            }
        }
    }
    pub fn set_channel(&mut self, channel: &Arc<Channel<T>>, looping: bool) {
        self.channel = Some(channel.clone());
        self.looping = looping
    }
    pub fn kill(&mut self) {
        self.channel = None;
    }
}

#[derive(Debug, Clone)]
pub struct Scale(Vec3);
impl Scale {
    fn new(axes: Vec3) -> Scale {
        Scale(axes)
    }
}
impl Add<Scale> for Scale {
    type Output = Scale;
    fn add(self, rhs: Scale) -> Self::Output {
        Scale(self.0 + rhs.0)
    }
}
impl Mul<f32> for Scale {
    type Output = Scale;
    fn mul(self, rhs: f32) -> Self::Output {
        Scale(self.0 * rhs)
    }
}

// Add for rotation and scale as well
pub(crate) fn translation(time: Res<Time>, query: Query<(&mut Player<Vec3>, &mut Transform)>) {
    let delta = time.delta_seconds();
    query.for_each_mut(|(mut player, mut transform)| {
        if let Some(translation) = player.play(delta) {
            transform.translation = translation;
        }
    });
}

pub(crate) fn rotation(time: Res<Time>, query: Query<(&mut Player<Quat>, &mut Transform)>) {
    let delta = time.delta_seconds();
    query.for_each_mut(|(mut player, mut transform)| {
        if let Some(rotation) = player.play(delta) {
            transform.rotation = rotation;
        }
    });
}

pub(crate) fn scale(time: Res<Time>, query: Query<(&mut Player<Scale>, &mut Transform)>) {
    let delta = time.delta_seconds();
    query.for_each_mut(|(mut player, mut transform)| {
        if let Some(scale) = player.play(delta) {
            transform.scale = scale.0;
        }
    });
}