//! Game project.
use crate::player::Player;
use fyrox::{
    core::{algebra::Vector2, log::Log, pool::Handle, reflect::prelude::*, visitor::prelude::*},
    engine::GraphicsContext,
    event::{Event, WindowEvent},
    gui::{
        grid::{Column, Grid, GridBuilder, Row},
        progress_bar::{ProgressBar, ProgressBarBuilder, ProgressBarMessage},
        stack_panel::StackPanelBuilder,
        text::{Text, TextBuilder, TextMessage},
        widget::{WidgetBuilder, WidgetMessage},
        HorizontalAlignment, Thickness, VerticalAlignment,
    },
    plugin::{error::GameResult, Plugin, PluginContext, PluginRegistrationContext},
    renderer::QualitySettings,
    resource::texture::{loader::TextureLoader, CompressionOptions, TextureImportOptions},
    scene::Scene,
};
use std::path::Path;

mod player;

#[derive(Default, Debug, Visit, Reflect)]
#[reflect(non_cloneable)]
pub struct Game {
    scene: Handle<Scene>,
    progress_bar: Handle<ProgressBar>,
    overlay_grid: Handle<Grid>,
    debug_text: Handle<Text>,
}

impl Game {
    fn handle_resize(&self, context: &mut PluginContext, new_size: Vector2<f32>) {
        let ui = context.user_interfaces.first();
        ui.send(self.overlay_grid, WidgetMessage::Width(new_size.x));
        ui.send(self.overlay_grid, WidgetMessage::Height(new_size.y));
    }
}

impl Plugin for Game {
    fn register(&self, context: PluginRegistrationContext) -> GameResult {
        context
            .serialization_context
            .script_constructors
            .add::<Player>("Player");
        Ok(())
    }

    fn init(&mut self, scene_path: Option<&str>, context: PluginContext) -> GameResult {
        context
            .resource_manager
            .state()
            .loaders
            .lock()
            .find_mut::<TextureLoader>()
            .unwrap()
            .default_import_options = TextureImportOptions::default()
            .with_anisotropy(1.0)
            .with_compression(CompressionOptions::Quality);

        context
            .async_scene_loader
            .request(scene_path.unwrap_or("data/Sponza/sponza.rgs"));

        let ctx = &mut context.user_interfaces.first_mut().build_ctx();
        self.overlay_grid = GridBuilder::new(
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
                            self.progress_bar = ProgressBarBuilder::new(
                                WidgetBuilder::new()
                                    .with_height(25.0)
                                    .with_margin(Thickness::uniform(2.0)),
                            )
                            .build(ctx);
                            self.progress_bar
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

        self.debug_text = TextBuilder::new(WidgetBuilder::new()).build(ctx);

        Ok(())
    }

    fn update(&mut self, context: &mut PluginContext) -> GameResult {
        let ui = context.user_interfaces.first();
        let progress = context.resource_manager.state().loading_progress() as f32 / 100.0;
        ui.send(self.progress_bar, ProgressBarMessage::Progress(progress));

        if let GraphicsContext::Initialized(graphics_context) = context.graphics_context {
            ui.send(
                self.debug_text,
                TextMessage::Text(format!("{}", graphics_context.renderer.get_statistics())),
            )
        }

        Ok(())
    }

    fn on_os_event(&mut self, event: &Event<()>, mut context: PluginContext) -> GameResult {
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
        Ok(())
    }

    fn on_graphics_context_initialized(&mut self, mut context: PluginContext) -> GameResult {
        let graphics_context = context.graphics_context.as_initialized_mut();

        let mut quality_settings = QualitySettings::ultra();

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

        Ok(())
    }

    fn on_scene_loaded(
        &mut self,
        _path: &Path,
        scene: Handle<Scene>,
        _data: &[u8],
        context: &mut PluginContext,
    ) -> GameResult {
        self.scene = scene;

        context
            .user_interfaces
            .first()
            .send(self.overlay_grid, WidgetMessage::Visibility(false));

        Ok(())
    }
}
