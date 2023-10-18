//! Game project.
use fyrox::plugin::{Plugin, PluginConstructor, PluginContext, PluginRegistrationContext};

pub struct GameConstructor;

impl PluginConstructor for GameConstructor {
    fn register(&self, context: PluginRegistrationContext) {
        fyrox_scripts::register(&context.serialization_context.script_constructors);
    }

    fn create_instance(&self, scene_path: Option<&str>, context: PluginContext) -> Box<dyn Plugin> {
        Box::new(Game::new(scene_path, context))
    }
}

pub struct Game;

impl Game {
    pub fn new(scene_path: Option<&str>, context: PluginContext) -> Self {
        context
            .async_scene_loader
            .request(scene_path.unwrap_or("data/Sponza.rgs"));

        Self
    }
}

impl Plugin for Game {}
