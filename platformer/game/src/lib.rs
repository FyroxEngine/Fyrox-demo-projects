//! Game project.
use fyrox::{
    core::{
        algebra::{Vector2, Vector3},
        pool::Handle,
        reflect::prelude::*,
        visitor::prelude::*,
        TypeUuidProvider,
        type_traits::prelude::*
    },
    engine::GraphicsContext,
    event::{ElementState, Event, WindowEvent},
    gui::{
        button::ButtonMessage,
        message::{MessageDirection, UiMessage},
        text::TextMessage,
        widget::WidgetMessage,
        UiNode, UserInterface,
    },
    keyboard::{KeyCode, PhysicalKey},
    plugin::{Plugin, PluginContext, PluginRegistrationContext},
    scene::{
        animation::spritesheet::SpriteSheetAnimation,
        dim2::{rectangle::Rectangle, rigidbody::RigidBody},
        node::Node,
        Scene,
    },
    script::{ScriptContext, ScriptTrait},
};
use std::path::Path;
use fyrox::core::ComponentProvider;
use fyrox::graph::SceneGraph;

#[derive(Visit, Reflect, Debug, Default)]
pub struct Game {
    scene: Handle<Scene>,
    debug_text: Handle<UiNode>,
    new_game: Handle<UiNode>,
    exit: Handle<UiNode>,
}

impl Plugin for Game {
    fn register(&self, context: PluginRegistrationContext) {
        let script_constructors = &context.serialization_context.script_constructors;
        script_constructors.add::<Player>("Player");
    }

    fn init(&mut self, scene_path: Option<&str>, ctx: PluginContext) {
        ctx.async_scene_loader
            .request(scene_path.unwrap_or("data/scene.rgs"));

        ctx.task_pool.spawn_plugin_task(
            UserInterface::load_from_file("data/menu.ui", ctx.resource_manager.clone()),
            |result, game: &mut Game, ctx| {
                *ctx.user_interfaces.first_mut() = result.unwrap();
                game.new_game = ctx.user_interfaces.first().find_handle_by_name_from_root("NewGame");
                game.exit = ctx.user_interfaces.first().find_handle_by_name_from_root("Exit");
                game.debug_text = ctx.user_interfaces.first().find_handle_by_name_from_root("DebugText");
            },
        );
    }

    fn update(&mut self, context: &mut PluginContext) {
        if let GraphicsContext::Initialized(graphics_context) = context.graphics_context {
            context.user_interfaces.first().send_message(TextMessage::text(
                self.debug_text,
                MessageDirection::ToWidget,
                format!("{}", graphics_context.renderer.get_statistics()),
            ));
        }
    }

    fn on_ui_message(&mut self, context: &mut PluginContext, message: &UiMessage) {
        if let Some(ButtonMessage::Click) = message.data() {
            if message.destination() == self.new_game {
                context
                    .user_interfaces.first()
                    .send_message(WidgetMessage::visibility(
                        context.user_interfaces.first().root(),
                        MessageDirection::ToWidget,
                        false,
                    ));
            } else if message.destination() == self.exit {
                if let Some(window_target) = context.window_target {
                    window_target.exit();
                }
            }
        }
    }

    fn on_scene_loaded(
        &mut self,
        _path: &Path,
        scene: Handle<Scene>,
        _data: &[u8],
        _context: &mut PluginContext,
    ) {
        self.scene = scene;
    }
}

#[derive(Visit, Reflect, Debug, Clone, TypeUuidProvider, ComponentProvider)]
#[type_uuid(id = "c5671d19-9f1a-4286-8486-add4ebaadaec")]
#[visit(optional)]
struct Player {
    sprite: Handle<Node>,
    move_left: bool,
    move_right: bool,
    jump: bool,
    animations: Vec<SpriteSheetAnimation>,
    current_animation: u32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            sprite: Handle::NONE,
            move_left: false,
            move_right: false,
            jump: false,
            animations: Default::default(),
            current_animation: 0,
        }
    }
}

impl ScriptTrait for Player {
    // Called everytime when there is an event from OS (mouse click, key press, etc.)
    fn on_os_event(&mut self, event: &Event<()>, _context: &mut ScriptContext) {
        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::KeyboardInput { event: input, .. } = event {
                let is_pressed = input.state == ElementState::Pressed;

                if let PhysicalKey::Code(code) = input.physical_key {
                    match code {
                        KeyCode::KeyA => self.move_left = is_pressed,
                        KeyCode::KeyD => self.move_right = is_pressed,
                        KeyCode::Space => self.jump = is_pressed,
                        _ => (),
                    }
                }
            }
        }
    }

    // Called every frame at fixed rate of 60 FPS.
    fn on_update(&mut self, context: &mut ScriptContext) {
        // The script can be assigned to any scene node, but we assert that it will work only with
        // 2d rigid body nodes.
        if let Some(rigid_body) = context.scene.graph[context.handle].cast_mut::<RigidBody>() {
            let x_speed = if self.move_left {
                3.0
            } else if self.move_right {
                -3.0
            } else {
                0.0
            };

            if x_speed != 0.0 {
                self.current_animation = 0;
            } else {
                self.current_animation = 1;
            }

            if self.jump {
                rigid_body.set_lin_vel(Vector2::new(x_speed, 4.0))
            } else {
                rigid_body.set_lin_vel(Vector2::new(x_speed, rigid_body.lin_vel().y))
            };

            // It is always a good practice to check whether the handles are valid, at this point we don't know
            // for sure what's the value of the `sprite` field. It can be unassigned and the following code won't
            // execute. A simple `context.scene.graph[self.sprite]` would just panicked in this case.
            if let Some(sprite) = context.scene.graph.try_get_mut(self.sprite) {
                // We want to change player orientation only if he's moving.
                if x_speed != 0.0 {
                    let local_transform = sprite.local_transform_mut();

                    let current_scale = **local_transform.scale();

                    local_transform.set_scale(Vector3::new(
                        // Just change X scaling to mirror player's sprite.
                        current_scale.x.copysign(-x_speed),
                        current_scale.y,
                        current_scale.z,
                    ));
                }
            }
        }

        if let Some(current_animation) = self.animations.get_mut(self.current_animation as usize) {
            current_animation.update(context.dt);

            if let Some(sprite) = context
                .scene
                .graph
                .try_get_mut(self.sprite)
                .and_then(|n| n.cast_mut::<Rectangle>())
            {
                // Set new frame to the sprite.
                sprite
                    .material()
                    .data_ref()
                    .set_texture(&"diffuseTexture".into(), current_animation.texture().into())
                    .unwrap();
                sprite.set_uv_rect(
                    current_animation
                        .current_frame_uv_rect()
                        .unwrap_or_default(),
                );
            }
        }
    }
}
