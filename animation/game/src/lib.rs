//! Game project.
use crate::player::Player;
use fyrox::{
    core::{algebra::Vector2, log::Log, pool::Handle},
    engine::GraphicsContext,
    event::{Event, WindowEvent},
    gui::{
        grid::{Column, GridBuilder, Row},
        message::MessageDirection,
        progress_bar::{ProgressBarBuilder, ProgressBarMessage},
        stack_panel::StackPanelBuilder,
        text::{TextBuilder, TextMessage},
        widget::{WidgetBuilder, WidgetMessage},
        HorizontalAlignment, Thickness, UiNode, VerticalAlignment,
    },
    plugin::{Plugin, PluginConstructor, PluginContext, PluginRegistrationContext},
    renderer::QualitySettings,
    resource::texture::{loader::TextureLoader, CompressionOptions, TextureImportOptions},
    scene::Scene,
};
use std::path::Path;

mod player;

pub struct GameConstructor;

impl PluginConstructor for GameConstructor {
    fn register(&self, context: PluginRegistrationContext) {
        context
            .serialization_context
            .script_constructors
            .add::<Player>("Player");
    }

    fn create_instance(&self, scene_path: Option<&str>, context: PluginContext) -> Box<dyn Plugin> {
        Box::new(Game::new(scene_path, context))
    }
}

pub struct Game {
    scene: Handle<Scene>,
    progress_bar: Handle<UiNode>,
    overlay_grid: Handle<UiNode>,
    debug_text: Handle<UiNode>,
}

impl Game {
    pub fn new(scene_path: Option<&str>, context: PluginContext) -> Self {
        context
            .resource_manager
            .state()
            .loaders
            .find_mut::<TextureLoader>()
            .unwrap()
            .default_import_options = TextureImportOptions::default()
            .with_anisotropy(1.0)
            .with_compression(CompressionOptions::Quality);

        context
            .async_scene_loader
            .request(scene_path.unwrap_or("data/scene.rgs"));

        let ctx = &mut context.user_interface.build_ctx();
        let progress_bar;
        let overlay_grid = GridBuilder::new(
            WidgetBuilder::new().with_child(
                StackPanelBuilder::new(
                    WidgetBuilder::new()
                        .on_row(1)
                        .on_column(1)
                        .with_vertical_alignment(VerticalAlignment::Center)
                        .with_child(
                            TextBuilder::new(WidgetBuilder::new())
                                .with_horizontal_text_alignment(HorizontalAlignment::Center)
                                .with_text("Loading... Please wait.")
                                .build(ctx),
                        )
                        .with_child({
                            progress_bar = ProgressBarBuilder::new(
                                WidgetBuilder::new()
                                    .with_height(25.0)
                                    .with_margin(Thickness::uniform(2.0)),
                            )
                            .build(ctx);
                            progress_bar
                        }),
                )
                .build(ctx),
            ),
        )
        .add_column(Column::stretch())
        .add_column(Column::strict(200.0))
        .add_column(Column::stretch())
        .add_row(Row::stretch())
        .add_row(Row::strict(100.0))
        .add_row(Row::stretch())
        .build(ctx);

        let debug_text = TextBuilder::new(WidgetBuilder::new()).build(ctx);

        Self {
            scene: Handle::NONE,
            progress_bar,
            overlay_grid,
            debug_text,
        }
    }

    fn handle_resize(&self, context: &mut PluginContext, new_size: Vector2<f32>) {
        context.user_interface.send_message(WidgetMessage::width(
            self.overlay_grid,
            MessageDirection::ToWidget,
            new_size.x,
        ));
        context.user_interface.send_message(WidgetMessage::height(
            self.overlay_grid,
            MessageDirection::ToWidget,
            new_size.y,
        ));
    }
}

impl Plugin for Game {
    fn update(&mut self, context: &mut PluginContext) {
        let progress = context.resource_manager.state().loading_progress() as f32 / 100.0;
        context
            .user_interface
            .send_message(ProgressBarMessage::progress(
                self.progress_bar,
                MessageDirection::ToWidget,
                progress,
            ));

        if let GraphicsContext::Initialized(graphics_context) = context.graphics_context {
            context.user_interface.send_message(TextMessage::text(
                self.debug_text,
                MessageDirection::ToWidget,
                format!("{}", graphics_context.renderer.get_statistics()),
            ))
        }
    }

    fn on_os_event(&mut self, event: &Event<()>, mut context: PluginContext) {
        match event {
            Event::WindowEvent { event, .. } => {
                if let WindowEvent::Resized(size) = event {
                    self.handle_resize(
                        &mut context,
                        Vector2::new(size.width as f32, size.height as f32),
                    )
                }
            }
            _ => (),
        }
    }

    fn on_graphics_context_initialized(&mut self, mut context: PluginContext) {
        let graphics_context = context.graphics_context.as_initialized_mut();

        let mut quality_settings = QualitySettings::high();

        quality_settings.point_shadows_distance = 6.0;
        quality_settings.spot_shadows_distance = 6.0;

        Log::verify(
            graphics_context
                .renderer
                .set_quality_settings(&quality_settings),
        );

        let inner_size = graphics_context.window.inner_size();
        self.handle_resize(
            &mut context,
            Vector2::new(inner_size.width as f32, inner_size.height as f32),
        );
    }

    fn on_scene_loaded(
        &mut self,
        _path: &Path,
        scene: Handle<Scene>,
        _data: &[u8],
        context: &mut PluginContext,
    ) {
        self.scene = scene;

        context
            .user_interface
            .send_message(WidgetMessage::visibility(
                self.overlay_grid,
                MessageDirection::ToWidget,
                false,
            ));
    }
}
