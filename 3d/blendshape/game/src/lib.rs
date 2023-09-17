//! Game project.
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector2, Vector3},
        log::Log,
        pool::Handle,
    },
    engine::GraphicsContext,
    event::{ElementState, Event, WindowEvent},
    event_loop::ControlFlow,
    gui::{
        grid::{Column, GridBuilder, Row},
        message::{MessageDirection, UiMessage},
        scroll_bar::{ScrollBarBuilder, ScrollBarMessage},
        scroll_viewer::ScrollViewerBuilder,
        text::{TextBuilder, TextMessage},
        widget::WidgetBuilder,
        window::{WindowBuilder, WindowTitle},
        UiNode, UserInterface,
    },
    keyboard::KeyCode,
    plugin::{Plugin, PluginConstructor, PluginContext},
    scene::{loader::AsyncSceneLoader, node::Node, Scene},
};
use std::collections::BTreeSet;

pub struct GameConstructor;

impl PluginConstructor for GameConstructor {
    fn create_instance(
        &self,
        override_scene: Handle<Scene>,
        context: PluginContext,
    ) -> Box<dyn Plugin> {
        Box::new(Game::new(override_scene, context))
    }
}

struct InputController {
    rotate_left: bool,
    rotate_right: bool,
}

pub struct Game {
    scene: Handle<Scene>,
    loader: Option<AsyncSceneLoader>,
    model_handle: Handle<Node>,
    input_controller: InputController,
    debug_text: Handle<UiNode>,
    model_angle: f32,
    sliders: Vec<(String, Handle<UiNode>)>,
}

impl Game {
    pub fn new(override_scene: Handle<Scene>, context: PluginContext) -> Self {
        let mut loader = None;
        let scene = if override_scene.is_some() {
            override_scene
        } else {
            loader = Some(AsyncSceneLoader::begin_loading(
                "data/scene.rgs".into(),
                context.serialization_context.clone(),
                context.resource_manager.clone(),
            ));
            Default::default()
        };

        let mut game = Self {
            scene,
            loader,
            model_handle: Default::default(),
            input_controller: InputController {
                rotate_left: false,
                rotate_right: false,
            },
            debug_text: TextBuilder::new(WidgetBuilder::new())
                .build(&mut context.user_interface.build_ctx()),
            model_angle: 180.0f32.to_radians(),
            sliders: vec![],
        };

        if override_scene.is_some() {
            game.on_scene_loaded(&mut context.scenes[override_scene], context.user_interface);
        }

        game
    }

    pub fn on_scene_loaded(&mut self, scene: &mut Scene, ui: &mut UserInterface) {
        let head = scene.graph.find_by_name_from_root("Head_Mesh").unwrap().0;
        let blend_shape = scene.graph[head].as_mesh_mut();

        let mut blend_shape_names = BTreeSet::new();
        for surface in blend_shape.surfaces_mut() {
            let data = surface.data();
            let data = data.lock();
            if let Some(container) = data.blend_shapes_container.as_ref() {
                for blend_shape in container.blend_shapes.iter() {
                    blend_shape_names.insert(blend_shape.name.clone());
                }
            }
        }

        let ctx = &mut ui.build_ctx();

        let mut children = Vec::new();
        let mut sliders = Vec::new();

        for (row, blend_shape_name) in blend_shape_names.iter().enumerate() {
            let short_name = blend_shape_name
                .strip_prefix("ExpressionBlendshapes.")
                .map(|n| n.to_owned())
                .unwrap_or_else(|| blend_shape_name.clone());

            let name = TextBuilder::new(WidgetBuilder::new().on_row(row))
                .with_text(short_name)
                .build(ctx);
            let slider = ScrollBarBuilder::new(WidgetBuilder::new().on_row(row).on_column(1))
                .with_min(0.0)
                .with_max(100.0)
                .with_step(1.0)
                .build(ctx);
            children.push(name);
            children.push(slider);
            sliders.push((blend_shape_name.clone(), slider));
        }

        WindowBuilder::new(
            WidgetBuilder::new()
                .with_width(250.0)
                .with_height(400.0)
                .with_desired_position(Vector2::new(5.0, 50.0)),
        )
        .with_title(WindowTitle::text("Blend Shapes"))
        .with_content(
            ScrollViewerBuilder::new(WidgetBuilder::new())
                .with_content(
                    GridBuilder::new(WidgetBuilder::new().with_children(children))
                        .add_column(Column::auto())
                        .add_column(Column::stretch())
                        .add_rows(
                            blend_shape_names
                                .iter()
                                .map(|_| Row::strict(20.0))
                                .collect(),
                        )
                        .build(ctx),
                )
                .build(ctx),
        )
        .build(ctx);

        self.model_handle = scene
            .graph
            .find_by_name_from_root("Gunan_animated2.fbx")
            .map(|(h, _)| h)
            .unwrap_or_default();
        self.sliders = sliders;
    }
}

impl Plugin for Game {
    fn update(&mut self, context: &mut PluginContext, _control_flow: &mut ControlFlow) {
        if let Some(loader) = self.loader.as_ref() {
            if let Some(result) = loader.fetch_result() {
                match result {
                    Ok(mut scene) => {
                        self.on_scene_loaded(&mut scene, &mut context.user_interface);
                        self.scene = context.scenes.add(scene);
                    }
                    Err(err) => Log::err(err),
                }
            }
        }

        if let Some(scene) = context.scenes.try_get_mut(self.scene) {
            // Rotate model according to input controller state
            if self.input_controller.rotate_left {
                self.model_angle -= 5.0f32.to_radians();
            } else if self.input_controller.rotate_right {
                self.model_angle += 5.0f32.to_radians();
            }

            scene.graph[self.model_handle]
                .local_transform_mut()
                .set_rotation(UnitQuaternion::from_axis_angle(
                    &Vector3::y_axis(),
                    self.model_angle,
                ));

            if let GraphicsContext::Initialized(ref graphics_context) = context.graphics_context {
                context.user_interface.send_message(TextMessage::text(
                    self.debug_text,
                    MessageDirection::ToWidget,
                    format!(
                        "Example - Blend Shapes\nUse [A][D] keys to rotate the model and sliders to select facial expression.\nFPS: {}",
                        graphics_context.renderer.get_statistics().frames_per_second
                    ),
                ));
            }
        }
    }

    fn on_os_event(
        &mut self,
        event: &Event<()>,
        _context: PluginContext,
        _control_flow: &mut ControlFlow,
    ) {
        if let Event::WindowEvent {
            event: WindowEvent::KeyboardInput { event: input, .. },
            ..
        } = event
        {
            match input.physical_key {
                KeyCode::KeyA => {
                    self.input_controller.rotate_left = input.state == ElementState::Pressed
                }
                KeyCode::KeyD => {
                    self.input_controller.rotate_right = input.state == ElementState::Pressed
                }
                _ => (),
            }
        }
    }

    fn on_ui_message(
        &mut self,
        context: &mut PluginContext,
        message: &UiMessage,
        _control_flow: &mut ControlFlow,
    ) {
        if let Some(ScrollBarMessage::Value(value)) = message.data() {
            if message.direction() == MessageDirection::FromWidget {
                for (name, slider) in self.sliders.iter() {
                    if message.destination() == *slider {
                        let scene = &mut context.scenes[self.scene];
                        let sphere = scene.graph.find_by_name_from_root("Head_Mesh").unwrap().0;
                        for blend_shape in scene.graph[sphere]
                            .as_mesh_mut()
                            .blend_shapes_mut()
                            .iter_mut()
                        {
                            if &blend_shape.name == name {
                                blend_shape.weight = *value;
                            }
                        }
                    }
                }
            }
        }
    }
}
