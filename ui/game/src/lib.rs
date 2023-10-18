//! Game project.
use fyrox::{
    asset::manager::ResourceManager,
    core::{
        algebra::{UnitQuaternion, Vector2, Vector3},
        color::Color,
        curve::{Curve, CurveKey, CurveKeyKind},
        log::Log,
        math::Rect,
        pool::Handle,
    },
    engine::GraphicsContext,
    event_loop::ControlFlow,
    gui::{
        border::BorderBuilder,
        brush::Brush,
        button::{ButtonBuilder, ButtonMessage},
        canvas::CanvasBuilder,
        check_box::CheckBoxBuilder,
        curve::CurveEditorBuilder,
        decorator::DecoratorBuilder,
        dock::{DockingManagerBuilder, TileBuilder, TileContent},
        dropdown_list::DropdownListBuilder,
        expander::ExpanderBuilder,
        formatted_text::WrapMode,
        grid::{Column, GridBuilder, Row},
        image::ImageBuilder,
        inspector::{
            editors::PropertyEditorDefinitionContainer, InspectorBuilder, InspectorContext,
            InspectorMessage, PropertyAction,
        },
        list_view::ListViewBuilder,
        message::{MessageDirection, UiMessage},
        messagebox::{MessageBoxBuilder, MessageBoxButtons, MessageBoxMessage},
        numeric::NumericUpDownBuilder,
        path::PathEditorBuilder,
        range::RangeEditorBuilder,
        rect::RectEditorBuilder,
        scroll_bar::{ScrollBarBuilder, ScrollBarMessage},
        scroll_viewer::ScrollViewerBuilder,
        searchbar::SearchBarBuilder,
        stack_panel::StackPanelBuilder,
        text::{TextBuilder, TextMessage},
        text_box::TextBoxBuilder,
        tree::{TreeBuilder, TreeRootBuilder},
        utils::make_simple_tooltip,
        vec::Vec3EditorBuilder,
        widget::WidgetBuilder,
        widget::WidgetMessage,
        window::{WindowBuilder, WindowTitle},
        wrap_panel::WrapPanelBuilder,
        BuildContext, HorizontalAlignment, Orientation, Thickness, UiNode, VerticalAlignment,
    },
    plugin::{Plugin, PluginConstructor, PluginContext},
    rand::{thread_rng, Rng},
    renderer::{CsmSettings, ShadowMapPrecision},
    resource::texture::Texture,
    scene::{node::Node, Scene},
    utils,
};
use std::path::Path;
use std::rc::Rc;

pub struct GameConstructor;

impl PluginConstructor for GameConstructor {
    fn create_instance(&self, scene_path: Option<&str>, context: PluginContext) -> Box<dyn Plugin> {
        Box::new(Game::new(scene_path, context))
    }
}

pub struct Game {
    scene: Handle<Scene>,
    interface: Option<Interface>,
    paladin: Handle<Node>,
}

impl Game {
    pub fn new(scene_path: Option<&str>, context: PluginContext) -> Self {
        context
            .async_scene_loader
            .request(scene_path.unwrap_or("data/scene.rgs"));

        Self {
            scene: Handle::NONE,
            interface: None,
            paladin: Default::default(),
        }
    }
}

impl Plugin for Game {
    fn update(&mut self, context: &mut PluginContext, _control_flow: &mut ControlFlow) {
        if let Some(interface) = self.interface.as_ref() {
            if let GraphicsContext::Initialized(ctx) = context.graphics_context {
                context.user_interface.send_message(TextMessage::text(
                    interface.debug_text,
                    MessageDirection::ToWidget,
                    format!("FPS: {}", ctx.renderer.get_statistics().frames_per_second),
                ))
            }
        }
    }

    fn on_graphics_context_initialized(
        &mut self,
        mut context: PluginContext,
        _control_flow: &mut ControlFlow,
    ) {
        self.interface = Some(Interface::new(&mut context));
    }

    fn on_ui_message(
        &mut self,
        context: &mut PluginContext,
        message: &UiMessage,
        _control_flow: &mut ControlFlow,
    ) {
        if let Some(interface) = self.interface.as_mut() {
            if let Some(ScrollBarMessage::Value(value)) = message.data() {
                if message.direction() == MessageDirection::FromWidget {
                    if let Some(paladin) = context
                        .scenes
                        .try_get_mut(self.scene)
                        .and_then(|s| s.graph.try_get_mut(self.paladin))
                    {
                        // Some of our scroll bars has changed its value. Check which one.
                        // Each message has source - a handle of UI element that created this message.
                        // It is used to understand from which UI element message has come.
                        if message.destination() == interface.scale {
                            paladin
                                .local_transform_mut()
                                .set_scale(Vector3::repeat(*value));
                        } else if message.destination() == interface.yaw {
                            paladin.local_transform_mut().set_rotation(
                                UnitQuaternion::from_axis_angle(
                                    &Vector3::y_axis(),
                                    value.to_radians(),
                                ),
                            );
                        }
                    }
                }
            } else if let Some(ButtonMessage::Click) = message.data() {
                // Once we received Click event from Reset button, we have to reset angle and scale
                // of model. To do that we borrow each UI element in engine and set its value directly.
                // This is not ideal because there is tight coupling between UI code and model values,
                // but still good enough for example.
                if message.destination() == interface.reset {
                    context.user_interface.send_message(ScrollBarMessage::value(
                        interface.scale,
                        MessageDirection::ToWidget,
                        0.005,
                    ));
                    context.user_interface.send_message(ScrollBarMessage::value(
                        interface.yaw,
                        MessageDirection::ToWidget,
                        180.0f32,
                    ));
                } else if message.destination() == interface.press_me_button {
                    interface.message_box = MessageBoxBuilder::new(
                        WindowBuilder::new(
                            WidgetBuilder::new().with_width(250.0).with_height(100.0),
                        )
                        .with_title(WindowTitle::text("Message Box Widget"))
                        .open(false),
                    )
                    .with_text("Hello!")
                    .with_buttons(MessageBoxButtons::Ok)
                    .build(&mut context.user_interface.build_ctx());

                    context.user_interface.send_message(MessageBoxMessage::open(
                        interface.message_box,
                        MessageDirection::ToWidget,
                        None,
                        None,
                    ));
                }
            } else if let Some(InspectorMessage::PropertyChanged(prop)) = message.data() {
                if let GraphicsContext::Initialized(graphics_context) = context.graphics_context {
                    if message.destination() == interface.quality_inspector {
                        let mut settings = graphics_context.renderer.get_quality_settings();
                        PropertyAction::from_field_kind(&prop.value).apply(
                            &prop.path(),
                            &mut settings,
                            &mut |e| Log::verify(e),
                        );
                        Log::verify(graphics_context.renderer.set_quality_settings(&settings));
                    }
                }
            } else if let Some(MessageBoxMessage::Close(_)) = message.data() {
                if message.destination() == interface.message_box {
                    context.user_interface.send_message(WidgetMessage::remove(
                        interface.message_box,
                        MessageDirection::ToWidget,
                    ));
                }
            }
        }
    }

    fn on_scene_loaded(&mut self, _path: &Path, scene: Handle<Scene>, context: &mut PluginContext) {
        self.scene = scene;

        let scene_ref = &mut context.scenes[scene];
        if let Some((handle, paladin)) = scene_ref.graph.find_by_name_from_root("paladin.fbx") {
            if let Some(interface) = self.interface.as_ref() {
                context.user_interface.send_message(ScrollBarMessage::value(
                    interface.yaw,
                    MessageDirection::ToWidget,
                    paladin
                        .local_transform()
                        .rotation()
                        .euler_angles()
                        .2
                        .to_degrees(),
                ));

                context.user_interface.send_message(ScrollBarMessage::value(
                    interface.scale,
                    MessageDirection::ToWidget,
                    paladin.local_transform().scale().x,
                ));
            }

            self.paladin = handle;
        }
    }
}

struct Interface {
    debug_text: Handle<UiNode>,
    yaw: Handle<UiNode>,
    scale: Handle<UiNode>,
    reset: Handle<UiNode>,
    quality_inspector: Handle<UiNode>,
    press_me_button: Handle<UiNode>,
    message_box: Handle<UiNode>,
}

fn make_potions_images(
    ctx: &mut BuildContext,
    resource_manager: &ResourceManager,
    w: usize,
    h: usize,
) -> Vec<Handle<UiNode>> {
    let mut potions = Vec::new();

    for y in 0..h {
        for x in 0..w {
            potions.push(
                ImageBuilder::new(
                    WidgetBuilder::new()
                        .with_width(32.0)
                        .with_height(32.0)
                        .with_margin(Thickness::uniform(1.0))
                        .with_desired_position(Vector2::new(
                            thread_rng().gen_range(0.0..300.0),
                            thread_rng().gen_range(0.0..200.0),
                        )),
                )
                .with_uv_rect(Rect::new(
                    x as f32 / 6.0,
                    y as f32 / 3.0,
                    1.0 / 6.0,
                    1.0 / 3.0,
                ))
                .with_texture(utils::into_gui_texture(
                    resource_manager.request::<Texture, _>("data/Potions.png"),
                ))
                .build(ctx),
            );
        }
    }

    potions
}

fn make_chests(ctx: &mut BuildContext, resource_manager: &ResourceManager) -> Vec<Handle<UiNode>> {
    let mut chests = Vec::new();

    let w = 8;
    let h = 6;
    for y in 0..h {
        for x in 0..w {
            chests.push(
                DecoratorBuilder::new(BorderBuilder::new(
                    WidgetBuilder::new().with_child(
                        GridBuilder::new(
                            WidgetBuilder::new()
                                .with_child(
                                    ImageBuilder::new(
                                        WidgetBuilder::new()
                                            .with_width(16.0)
                                            .with_height(16.0)
                                            .with_margin(Thickness::uniform(1.0))
                                            .with_desired_position(Vector2::new(
                                                thread_rng().gen_range(0.0..300.0),
                                                thread_rng().gen_range(0.0..200.0),
                                            )),
                                    )
                                    .with_uv_rect(Rect::new(
                                        x as f32 / w as f32,
                                        y as f32 / h as f32,
                                        1.0 / w as f32,
                                        1.0 / h as f32,
                                    ))
                                    .with_texture(utils::into_gui_texture(
                                        resource_manager.request::<Texture, _>("data/chests.png"),
                                    ))
                                    .build(ctx),
                                )
                                .with_child(
                                    TextBuilder::new(WidgetBuilder::new().on_column(1))
                                        .with_text(format!("Chest {}", y * w + x))
                                        .build(ctx),
                                ),
                        )
                        .add_row(Row::stretch())
                        .add_column(Column::auto())
                        .add_column(Column::stretch())
                        .build(ctx),
                    ),
                ))
                .build(ctx),
            )
        }
    }

    chests
}

fn make_tree(
    ctx: &mut BuildContext,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    next: bool,
    resource_manager: &ResourceManager,
) -> Handle<UiNode> {
    TreeBuilder::new(WidgetBuilder::new())
        .with_content(
            GridBuilder::new(
                WidgetBuilder::new()
                    .with_child(
                        ImageBuilder::new(
                            WidgetBuilder::new()
                                .with_width(16.0)
                                .with_height(16.0)
                                .with_margin(Thickness::uniform(1.0))
                                .with_desired_position(Vector2::new(
                                    thread_rng().gen_range(0.0..300.0),
                                    thread_rng().gen_range(0.0..200.0),
                                )),
                        )
                        .with_uv_rect(Rect::new(
                            x as f32 / w as f32,
                            y as f32 / h as f32,
                            1.0 / w as f32,
                            1.0 / h as f32,
                        ))
                        .with_texture(utils::into_gui_texture(
                            resource_manager.request::<Texture, _>("data/armours.png"),
                        ))
                        .build(ctx),
                    )
                    .with_child(
                        TextBuilder::new(WidgetBuilder::new().on_column(1))
                            .with_text(format!("Armor {}", y * w + x))
                            .build(ctx),
                    ),
            )
            .add_row(Row::stretch())
            .add_column(Column::auto())
            .add_column(Column::stretch())
            .build(ctx),
        )
        .with_items(if next {
            vec![make_tree(ctx, x + 1, y, w, h, x + 1 < w, resource_manager)]
        } else {
            vec![]
        })
        .build(ctx)
}

fn make_tree_root(ctx: &mut BuildContext, resource_manager: &ResourceManager) -> Handle<UiNode> {
    let mut items = Vec::new();

    let w = 9;
    let h = 19;
    for y in 0..h {
        items.push(make_tree(ctx, 0, y, w, h, true, resource_manager))
    }

    TreeRootBuilder::new(
        WidgetBuilder::new()
            .with_margin(Thickness::uniform(1.0))
            .with_tooltip(make_simple_tooltip(
                ctx,
                "Tree - used to show hierarchical data",
            )),
    )
    .with_items(items)
    .build(ctx)
}

impl Interface {
    fn new(plugin_ctx: &mut PluginContext) -> Self {
        let ctx = plugin_ctx.graphics_context.as_initialized_ref();
        let window_width = ctx.renderer.get_frame_size().0 as f32;

        let ctx = &mut plugin_ctx.user_interface.build_ctx();

        let yaw;
        let scale;
        let reset;
        let model_options = WindowBuilder::new(
            WidgetBuilder::new()
                // We want the window to be anchored at right top corner at the beginning
                .with_desired_position(Vector2::new(window_width - 300.0, 0.0))
                .with_width(300.0),
        )
        .with_content(
            GridBuilder::new(
                WidgetBuilder::new()
                    .with_child(
                        TextBuilder::new(
                            WidgetBuilder::new()
                                .on_row(0)
                                .on_column(0)
                                .with_margin(Thickness::uniform(1.0))
                                .with_vertical_alignment(VerticalAlignment::Center),
                        )
                        .with_text("Yaw")
                        .build(ctx),
                    )
                    .with_child({
                        yaw = ScrollBarBuilder::new(
                            WidgetBuilder::new()
                                .on_row(0)
                                .on_column(1)
                                // Make sure scroll bar will stay in center of available space.
                                .with_vertical_alignment(VerticalAlignment::Center)
                                // Add some margin so ui element won't be too close to each other.
                                .with_margin(Thickness::uniform(2.0)),
                        )
                        .with_min(0.0)
                        // Our max rotation is 360 degrees.
                        .with_max(360.0)
                        // Set step by which value will change when user will click on arrows.
                        .with_step(5.0)
                        // Make sure scroll bar will show its current value on slider.
                        .show_value(true)
                        // Turn off all decimal places.
                        .with_value_precision(0)
                        .build(ctx);
                        yaw
                    })
                    .with_child(
                        TextBuilder::new(
                            WidgetBuilder::new()
                                .on_row(1)
                                .on_column(0)
                                .with_margin(Thickness::uniform(1.0))
                                .with_vertical_alignment(VerticalAlignment::Center),
                        )
                        .with_wrap(WrapMode::Word)
                        .with_text("Scale")
                        .build(ctx),
                    )
                    .with_child({
                        scale = ScrollBarBuilder::new(
                            WidgetBuilder::new()
                                .on_row(1)
                                .on_column(1)
                                .with_vertical_alignment(VerticalAlignment::Center)
                                .with_margin(Thickness::uniform(2.0)),
                        )
                        .with_min(0.001)
                        .with_max(0.01)
                        .with_step(0.01)
                        .show_value(true)
                        .build(ctx);
                        scale
                    })
                    .with_child(
                        StackPanelBuilder::new(
                            WidgetBuilder::new()
                                .on_row(3)
                                .on_column(1)
                                .with_horizontal_alignment(HorizontalAlignment::Right)
                                .with_child({
                                    reset = ButtonBuilder::new(
                                        WidgetBuilder::new()
                                            .with_width(100.0)
                                            .with_margin(Thickness::uniform(1.0)),
                                    )
                                    .with_text("Reset")
                                    .build(ctx);
                                    reset
                                }),
                        )
                        .with_orientation(Orientation::Horizontal)
                        .build(ctx),
                    ),
            )
            .add_column(Column::strict(100.0))
            .add_column(Column::stretch())
            .add_row(Row::strict(30.0))
            .add_row(Row::strict(30.0))
            .add_row(Row::stretch())
            .add_row(Row::strict(30.0))
            .build(ctx),
        )
        .with_title(WindowTitle::text("Model Options"))
        .can_close(false)
        .build(ctx);

        let quality_settings = plugin_ctx
            .graphics_context
            .as_initialized_ref()
            .renderer
            .get_quality_settings();

        let container = PropertyEditorDefinitionContainer::new();
        container.register_inheritable_inspectable::<CsmSettings>();
        container.register_inheritable_enum::<ShadowMapPrecision, _>();

        // Create another window which will show some graphics options.
        let debug_text;
        let quality_inspector;
        let graphics = WindowBuilder::new(
            WidgetBuilder::new()
                .with_desired_position(Vector2::new(window_width - 670.0, 0.0))
                .with_width(350.0),
        )
        .with_content(
            GridBuilder::new(
                WidgetBuilder::new()
                    .with_child({
                        debug_text = TextBuilder::new(WidgetBuilder::new().on_row(0).on_column(0))
                            .build(ctx);
                        debug_text
                    })
                    .with_child(
                        ScrollViewerBuilder::new(WidgetBuilder::new().on_row(1))
                            .with_content({
                                quality_inspector = InspectorBuilder::new(WidgetBuilder::new())
                                    .with_context(InspectorContext::from_object(
                                        &quality_settings,
                                        ctx,
                                        Rc::new(container),
                                        None,
                                        u64::MAX,
                                        0,
                                        true,
                                        Default::default(),
                                    ))
                                    .build(ctx);
                                quality_inspector
                            })
                            .build(ctx),
                    ),
            )
            .add_row(Row::auto())
            .add_row(Row::stretch())
            .add_column(Column::stretch())
            .build(ctx),
        )
        .with_title(WindowTitle::text("Graphics Options"))
        .can_close(false)
        .build(ctx);

        let press_me_button;
        let controls_expander = ExpanderBuilder::new(WidgetBuilder::new())
            .with_header(
                TextBuilder::new(WidgetBuilder::new())
                    .with_text("Controls")
                    .build(ctx),
            )
            .with_expanded(true)
            .with_content(
                StackPanelBuilder::new(
                    WidgetBuilder::new()
                        .with_margin(Thickness::uniform(2.0))
                        .with_child(
                            GridBuilder::new(
                                WidgetBuilder::new()
                                    .with_child({
                                        press_me_button = ButtonBuilder::new(
                                            WidgetBuilder::new()
                                                .on_column(0)
                                                .with_margin(Thickness::uniform(1.0))
                                                .with_tooltip(make_simple_tooltip(
                                                    ctx,
                                                    "Button - a simplest clickable widget",
                                                )),
                                        )
                                        .with_text("Press Me!")
                                        .build(ctx);
                                        press_me_button
                                    })
                                    .with_child(
                                        ButtonBuilder::new(
                                            WidgetBuilder::new()
                                                .on_column(1)
                                                .with_margin(Thickness::uniform(1.0))
                                                .with_tooltip(make_simple_tooltip(
                                                    ctx,
                                                    "Button with custom content.",
                                                )),
                                        )
                                        .with_content(
                                            GridBuilder::new(
                                                WidgetBuilder::new()
                                                    .with_margin(Thickness::uniform(2.0))
                                                    .with_horizontal_alignment(
                                                        HorizontalAlignment::Center,
                                                    )
                                                    .with_child(
                                                        ImageBuilder::new(
                                                            WidgetBuilder::new()
                                                                .on_column(0)
                                                                .with_width(20.0)
                                                                .with_height(20.0),
                                                        )
                                                        .with_texture(utils::into_gui_texture(
                                                            plugin_ctx
                                                                .resource_manager
                                                                .request::<Texture, _>(
                                                                    "data/Potions.png",
                                                                ),
                                                        ))
                                                        .with_uv_rect(Rect::new(
                                                            4.0 / 6.0,
                                                            0.0,
                                                            1.0 / 6.0,
                                                            1.0 / 3.0,
                                                        ))
                                                        .build(ctx),
                                                    )
                                                    .with_child(
                                                        TextBuilder::new(
                                                            WidgetBuilder::new().on_column(1),
                                                        )
                                                        .with_text("Drink Potion")
                                                        .build(ctx),
                                                    ),
                                            )
                                            .add_column(Column::auto())
                                            .add_column(Column::auto())
                                            .add_row(Row::stretch())
                                            .build(ctx),
                                        )
                                        .build(ctx),
                                    ),
                            )
                            .add_row(Row::stretch())
                            .add_column(Column::stretch())
                            .add_column(Column::stretch())
                            .build(ctx),
                        )
                        .with_child(
                            CheckBoxBuilder::new(
                                WidgetBuilder::new()
                                    .with_margin(Thickness::uniform(1.0))
                                    .with_tooltip(make_simple_tooltip(
                                        ctx,
                                        "CheckBox - an input field for Option<bool>",
                                    )),
                            )
                            .with_content(
                                TextBuilder::new(WidgetBuilder::new())
                                    .with_text("Check Me!")
                                    .build(ctx),
                            )
                            .checked(Some(true))
                            .build(ctx),
                        )
                        .with_child(
                            BorderBuilder::new(
                                WidgetBuilder::new()
                                    .with_margin(Thickness::uniform(1.0))
                                    .with_tooltip(make_simple_tooltip(
                                        ctx,
                                        "Border - container widget with different \
                                        border thicknesses",
                                    ))
                                    .with_foreground(Brush::Solid(Color::opaque(0, 162, 232)))
                                    .with_child(
                                        TextBuilder::new(WidgetBuilder::new())
                                            .with_text(
                                                "Text inside a Border with \
                                            different border thicknesses",
                                            )
                                            .build(ctx),
                                    ),
                            )
                            .with_stroke_thickness(Thickness {
                                left: 2.0,
                                top: 1.0,
                                right: 2.0,
                                bottom: 1.0,
                            })
                            .build(ctx),
                        )
                        .with_child(
                            TextBoxBuilder::new(
                                WidgetBuilder::new()
                                    .with_margin(Thickness::uniform(1.0))
                                    .with_tooltip(make_simple_tooltip(
                                        ctx,
                                        "TextBox - text input field",
                                    )),
                            )
                            .with_text("Text box with some text")
                            .with_multiline(true)
                            .with_wrap(WrapMode::Word)
                            .build(ctx),
                        )
                        .with_child(
                            ScrollBarBuilder::new(
                                WidgetBuilder::new()
                                    .with_height(22.0)
                                    .with_margin(Thickness::uniform(1.0))
                                    .with_tooltip(make_simple_tooltip(
                                        ctx,
                                        "ScrollBar - a bounded range with \
                                        a cursor.",
                                    )),
                            )
                            .build(ctx),
                        )
                        .with_child(
                            Vec3EditorBuilder::<f32>::new(
                                WidgetBuilder::new()
                                    .with_margin(Thickness::uniform(1.0))
                                    .with_tooltip(make_simple_tooltip(
                                        ctx,
                                        "VecEditor - a numeric input field \
                                        for Vector<N, T> type",
                                    )),
                            )
                            .build(ctx),
                        )
                        .with_child(
                            NumericUpDownBuilder::new(
                                WidgetBuilder::new()
                                    .with_margin(Thickness::uniform(1.0))
                                    .with_tooltip(make_simple_tooltip(
                                        ctx,
                                        "NumericUpDown - a numeric input \
                                        field",
                                    )),
                            )
                            .with_value(123.321f32)
                            .build(ctx),
                        )
                        .with_child(
                            RectEditorBuilder::new(
                                WidgetBuilder::new()
                                    .with_margin(Thickness::uniform(1.0))
                                    .with_tooltip(make_simple_tooltip(
                                        ctx,
                                        "RectEditor - an input field for \
                                        Rect<T> type",
                                    )),
                            )
                            .with_value(Rect::new(-1.0, -2.0, 3.0, 4.0))
                            .build(ctx),
                        )
                        .with_child(
                            RangeEditorBuilder::new(
                                WidgetBuilder::new()
                                    .with_margin(Thickness::uniform(1.0))
                                    .with_tooltip(make_simple_tooltip(
                                        ctx,
                                        "RangeEditor - an input field for \
                                        Range<T> type",
                                    )),
                            )
                            .with_value(-123.321..321.123)
                            .build(ctx),
                        )
                        .with_child(
                            PathEditorBuilder::new(
                                WidgetBuilder::new()
                                    .with_margin(Thickness::uniform(1.0))
                                    .with_tooltip(make_simple_tooltip(
                                        ctx,
                                        "PathEditor - an input field for \
                                        PathBuf type",
                                    )),
                            )
                            .with_path("data/Potions.png")
                            .build(ctx),
                        )
                        .with_child(
                            ScrollViewerBuilder::new(WidgetBuilder::new().with_height(300.0))
                                .with_content(make_tree_root(ctx, &plugin_ctx.resource_manager))
                                .build(ctx),
                        )
                        .with_child(
                            SearchBarBuilder::new(
                                WidgetBuilder::new()
                                    .with_margin(Thickness::uniform(1.0))
                                    .with_tooltip(make_simple_tooltip(
                                        ctx,
                                        "SearchBar - an input field search text \
                                        with additional functionality",
                                    )),
                            )
                            .build(ctx),
                        )
                        .with_child(
                            ListViewBuilder::new(
                                WidgetBuilder::new()
                                    .with_margin(Thickness::uniform(1.0))
                                    .with_height(200.0)
                                    .with_tooltip(make_simple_tooltip(
                                        ctx,
                                        "ListView - a container for \
                                        arbitrary widgets",
                                    )),
                            )
                            .with_items(make_chests(ctx, &plugin_ctx.resource_manager))
                            .build(ctx),
                        )
                        .with_child(
                            CurveEditorBuilder::new(
                                WidgetBuilder::new()
                                    .with_margin(Thickness::uniform(1.0))
                                    .with_height(200.0)
                                    .with_width(400.0)
                                    .with_tooltip(make_simple_tooltip(
                                        ctx,
                                        "CurveEditor - helps you to edit \
                                        parametric curves",
                                    )),
                            )
                            .with_curve(Curve::from(vec![
                                CurveKey::new(0.0, 30.0, CurveKeyKind::Constant),
                                CurveKey::new(100.0, -30.0, CurveKeyKind::Linear),
                                CurveKey::new(
                                    200.0,
                                    75.0,
                                    CurveKeyKind::Cubic {
                                        left_tangent: 1.0,
                                        right_tangent: 2.0,
                                    },
                                ),
                                CurveKey::new(
                                    300.0,
                                    -75.0,
                                    CurveKeyKind::Cubic {
                                        left_tangent: 1.0,
                                        right_tangent: 2.0,
                                    },
                                ),
                            ]))
                            .build(ctx),
                        )
                        .with_child(
                            DropdownListBuilder::new(
                                WidgetBuilder::new()
                                    .with_margin(Thickness::uniform(1.0))
                                    .with_height(22.0)
                                    .with_tooltip(make_simple_tooltip(
                                        ctx,
                                        "DropdownList - a container for arbitrary \
                                        widgets with a preview for selected item",
                                    )),
                            )
                            .with_selected(2)
                            .with_items(make_chests(ctx, &plugin_ctx.resource_manager))
                            .build(ctx),
                        ),
                )
                .build(ctx),
            )
            .build(ctx);

        let layout_panels_expander = ExpanderBuilder::new(WidgetBuilder::new())
            .with_header(
                TextBuilder::new(WidgetBuilder::new())
                    .with_text("Layout Panels")
                    .build(ctx),
            )
            .with_expanded(true)
            .with_content(
                StackPanelBuilder::new(
                    WidgetBuilder::new()
                        .with_child(
                            WrapPanelBuilder::new(
                                WidgetBuilder::new()
                                    .with_children(make_potions_images(
                                        ctx,
                                        &plugin_ctx.resource_manager,
                                        6,
                                        3,
                                    ))
                                    .with_tooltip(make_simple_tooltip(
                                        ctx,
                                        "WrapPanel - stacks children either \
                                    horizontally or vertically with overflow",
                                    )),
                            )
                            .with_orientation(Orientation::Horizontal)
                            .build(ctx),
                        )
                        .with_child(
                            StackPanelBuilder::new(
                                WidgetBuilder::new()
                                    .with_children(make_potions_images(
                                        ctx,
                                        &plugin_ctx.resource_manager,
                                        4,
                                        1,
                                    ))
                                    .with_tooltip(make_simple_tooltip(
                                        ctx,
                                        "StackPanel - stacks children either \
                                    horizontally or vertically",
                                    )),
                            )
                            .with_orientation(Orientation::Vertical)
                            .build(ctx),
                        )
                        .with_child(
                            CanvasBuilder::new(
                                WidgetBuilder::new()
                                    .with_width(300.0)
                                    .with_height(200.0)
                                    .with_children(make_potions_images(
                                        ctx,
                                        &plugin_ctx.resource_manager,
                                        6,
                                        3,
                                    ))
                                    .with_tooltip(make_simple_tooltip(
                                        ctx,
                                        "Canvas - allows children widgets \
                                        to have arbitrary position",
                                    )),
                            )
                            .build(ctx),
                        ),
                )
                .build(ctx),
            )
            .build(ctx);

        // Build widget gallery
        let widget_gallery = WindowBuilder::new(WidgetBuilder::new())
            .with_content(
                ScrollViewerBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(2.0)))
                    .with_content(
                        StackPanelBuilder::new(
                            WidgetBuilder::new()
                                .with_child(controls_expander)
                                .with_child(layout_panels_expander),
                        )
                        .build(ctx),
                    )
                    .build(ctx),
            )
            .with_title(WindowTitle::text("Widget Gallery"))
            .build(ctx);

        WindowBuilder::new(
            WidgetBuilder::new()
                .with_width(500.0)
                .with_height(650.0)
                .with_desired_position(Vector2::new(30.0, 30.0)),
        )
        .can_close(false)
        .can_minimize(false)
        .with_title(WindowTitle::text("Docking Manager"))
        .with_content(
            DockingManagerBuilder::new(
                WidgetBuilder::new().with_child(
                    TileBuilder::new(WidgetBuilder::new())
                        .with_content(TileContent::VerticalTiles {
                            tiles: [
                                TileBuilder::new(WidgetBuilder::new())
                                    .with_content(TileContent::HorizontalTiles {
                                        tiles: [
                                            TileBuilder::new(WidgetBuilder::new())
                                                .with_content(TileContent::Window(graphics))
                                                .build(ctx),
                                            TileBuilder::new(WidgetBuilder::new())
                                                .with_content(TileContent::Window(model_options))
                                                .build(ctx),
                                        ],
                                        splitter: 0.5,
                                    })
                                    .build(ctx),
                                TileBuilder::new(WidgetBuilder::new())
                                    .with_content(TileContent::Window(widget_gallery))
                                    .build(ctx),
                            ],
                            splitter: 0.2,
                        })
                        .build(ctx),
                ),
            )
            .build(ctx),
        )
        .build(ctx);

        Interface {
            debug_text,
            yaw,
            scale,
            reset,
            quality_inspector,
            press_me_button,
            message_box: Default::default(),
        }
    }
}
