use fyrox::{
    core::{
        color::{Color, Hsv},
        pool::Handle,
        reflect::prelude::*,
        type_traits::prelude::*,
        visitor::prelude::*,
    },
    gui::{
        border::BorderBuilder,
        brush::Brush,
        define_constructor, define_widget_deref,
        message::{MessageDirection, UiMessage},
        text::TextBuilder,
        widget::{Widget, WidgetBuilder, WidgetMessage},
        BuildContext, Control, HorizontalAlignment, Thickness, UiNode, UserInterface,
        VerticalAlignment,
    },
};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq)]
pub enum MyButtonMessage {
    // A message, that will be emitted when our button is clicked.
    Click,
}

impl MyButtonMessage {
    // A constructor for `Click` message.
    define_constructor!(
        MyButtonMessage:Click => fn click(), layout: false
    );
}

#[derive(Clone, Debug, Reflect, Visit, TypeUuidProvider, ComponentProvider)]
#[type_uuid(id = "3392233e-dafb-42f6-a53d-92e3b7e554ad")]
struct MyButton {
    widget: Widget,
    border: Handle<UiNode>,
    text: Handle<UiNode>,
}

define_widget_deref!(MyButton);

impl MyButton {
    fn set_colors(&self, ui: &UserInterface, text_color: Color, border_color: Color) {
        for (handle, color) in [(self.border, border_color), (self.text, text_color)] {
            ui.send_message(WidgetMessage::foreground(
                handle,
                MessageDirection::ToWidget,
                Brush::Solid(color).into(),
            ));
        }

        let mut border_color = Hsv::from(border_color);
        border_color.set_brightness(border_color.brightness() - 20.0);
        ui.send_message(WidgetMessage::background(
            self.border,
            MessageDirection::ToWidget,
            Brush::Solid(border_color.into()).into(),
        ));
    }
}

impl Control for MyButton {
    fn handle_routed_message(&mut self, ui: &mut UserInterface, message: &mut UiMessage) {
        // Pass another message to the base widget first.
        self.widget.handle_routed_message(ui, message);

        // Then process it in our widget.
        if let Some(msg) = message.data::<WidgetMessage>() {
            if message.destination() == self.handle()
                || self.has_descendant(message.destination(), ui)
            {
                match msg {
                    WidgetMessage::MouseUp { .. } => {
                        // Send the message to outside world, saying that the button was clicked.
                        ui.send_message(MyButtonMessage::click(
                            self.handle(),
                            MessageDirection::FromWidget,
                        ));
                        ui.release_mouse_capture();
                    }
                    WidgetMessage::MouseDown { .. } => {
                        ui.capture_mouse(message.destination());
                    }
                    WidgetMessage::MouseEnter => {
                        // Make both the border and text brighter when the mouse enter the bounds of our button.
                        self.set_colors(
                            ui,
                            Color::opaque(220, 220, 220),
                            Color::opaque(140, 140, 140),
                        );
                    }
                    WidgetMessage::MouseLeave => {
                        // Make both the border and text dimmer when the mouse leaves the bounds of our button.
                        self.set_colors(
                            ui,
                            Color::opaque(120, 120, 120),
                            Color::opaque(100, 100, 100),
                        );
                    }
                    _ => (),
                }
            }
        }
    }
}

pub struct MyButtonBuilder {
    widget_builder: WidgetBuilder,
    // Some text of our button.
    text: String,
}

impl MyButtonBuilder {
    pub fn new(widget_builder: WidgetBuilder) -> Self {
        Self {
            widget_builder,
            text: Default::default(),
        }
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.text = text;
        self
    }

    pub fn build(self, ctx: &mut BuildContext) -> Handle<UiNode> {
        let text = TextBuilder::new(
            WidgetBuilder::new()
                .with_vertical_alignment(VerticalAlignment::Center)
                .with_horizontal_alignment(HorizontalAlignment::Center),
        )
        .with_text(self.text)
        .build(ctx);

        let border = BorderBuilder::new(WidgetBuilder::new().with_child(text))
            .with_stroke_thickness(Thickness::uniform(2.0).into())
            .build(ctx);

        let button = MyButton {
            widget: self.widget_builder.with_child(border).build(ctx),
            border,
            text,
        };

        ctx.add_node(UiNode::new(button))
    }
}
