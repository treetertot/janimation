use bevy_ecs::{prelude::IntoSystem, schedule::ParallelSystemDescriptorCoercion};
use bevy_internal::prelude::Plugin;

pub use crate::interpolation::*;

use crate::{bank::{animation_starter, animation_stopper, animation_cleaner}, player::{rotation, scale, translation}};

pub struct SkeletonPlugin;
impl Plugin for SkeletonPlugin {
    fn build(&self, app: &mut bevy_internal::prelude::AppBuilder) {
        app
            .add_system(animation_starter.system().label("animation_starter"))
            .add_system(animation_stopper.system().label("animation_stopper").before("animation_starter"))
            .add_system(animation_cleaner.system().label("animation_cleaner").before("animation_stopper"))
            .add_system(translation.system().after("animation_starter"))
            .add_system(rotation.system().after("animation_starter"))
            .add_system(scale.system().after("animation_starter"));
    }
}