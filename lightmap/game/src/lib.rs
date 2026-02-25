//! Game project.
use fyrox::core::{reflect::prelude::*, visitor::prelude::*};
use fyrox::plugin::error::GameResult;
use fyrox::plugin::{Plugin, PluginContext, PluginRegistrationContext};

#[derive(Visit, Reflect, Clone, Default, Debug)]
pub struct Game;

impl Plugin for Game {
    fn register(&self, context: PluginRegistrationContext) -> GameResult {
        fyrox_scripts::register(&context.serialization_context.script_constructors);
        Ok(())
    }

    fn init(&mut self, scene_path: Option<&str>, mut context: PluginContext) -> GameResult {
        context.load_scene_or_ui::<Game>(scene_path.unwrap_or("data/Sponza/sponza.rgs"));
        Ok(())
    }
}
