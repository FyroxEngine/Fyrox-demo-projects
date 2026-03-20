//! Game project.
use crate::player::Player;
use fyrox::{
    core::{algebra::Vector2, pool::Handle, reflect::prelude::*, visitor::prelude::*},
    engine::GraphicsContext,
    gui::{
        grid::{Column, GridBuilder, Row},
        progress_bar::{ProgressBar, ProgressBarBuilder, ProgressBarMessage},
        screen::{Screen, ScreenBuilder},
        stack_panel::StackPanelBuilder,
        text::{Text, TextBuilder, TextMessage},
        widget::{WidgetBuilder, WidgetMessage},
        HorizontalAlignment, Thickness, UserInterface, VerticalAlignment,
    },
    plugin::{
        error::GameResult, Plugin, PluginContext, PluginRegistrationContext, SceneLoaderResult,
    },
    renderer::QualitySettings,
    resource::texture::{loader::TextureLoader, CompressionOptions, TextureImportOptions},
    scene::Scene,
};

mod player;

#[derive(Default, Debug, Visit, Reflect)]
#[reflect(non_cloneable)]
pub struct Game {
    scene: Handle<Scene>,
    progress_bar: Handle<ProgressBar>,
    overlay_screen: Handle<Screen>,
    debug_text: Handle<Text>,
}

impl Game {
    fn on_scene_loading_result(
        &mut self,
        result: SceneLoaderResult,
        ctx: &mut PluginContext,
    ) -> GameResult {
        self.scene = ctx.scenes.add(result?.payload);
        ctx.user_interfaces
            .first()
            .send(self.overlay_screen, WidgetMessage::Visibility(false));
        Ok(())
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

    fn init(&mut self, scene_path: Option<&str>, mut context: PluginContext) -> GameResult {
        context
            .user_interfaces
            .add(UserInterface::new(Vector2::repeat(100.0)));

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

        context.load_scene(
            scene_path.unwrap_or("data/Sponza/sponza.rgs"),
            false,
            |result, game: &mut Game, ctx| game.on_scene_loading_result(result, ctx),
        );

        let ctx = &mut context.user_interfaces.first_mut().build_ctx();
        self.overlay_screen = ScreenBuilder::new(
            WidgetBuilder::new().with_child(
                GridBuilder::new(
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
                .build(ctx),
            ),
        )
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

    fn on_graphics_context_initialized(&mut self, context: PluginContext) -> GameResult {
        let graphics_context = context.graphics_context.as_initialized_mut();
        let quality_settings = QualitySettings::ultra();
        graphics_context
            .renderer
            .set_quality_settings(&quality_settings)?;
        Ok(())
    }
}
