//! Game project.
use fyrox::core::{reflect::prelude::*, visitor::prelude::*};
use fyrox::plugin::{Plugin, PluginContext, PluginRegistrationContext};

#[derive(Visit, Reflect, Default, Debug)]
pub struct Game;

impl Plugin for Game {
    fn register(&self, context: PluginRegistrationContext) {
        fyrox_scripts::register(&context.serialization_context.script_constructors);
    }

    fn init(&mut self, scene_path: Option<&str>, context: PluginContext) {
        context
            .async_scene_loader
            .request(scene_path.unwrap_or("data/Sponza.rgs"));
    }
}
