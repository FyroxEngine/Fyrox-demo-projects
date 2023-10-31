//! Executor with your game connected to it as a plugin.
use fyrox::{
    dpi::LogicalSize,
    engine::{executor::Executor, GraphicsContextParams},
    event_loop::EventLoop,
    window::WindowAttributes,
};
use ui::GameConstructor;

fn main() {
    let mut window_attributes = WindowAttributes::default();
    window_attributes.inner_size = Some(LogicalSize::new(1280.0, 720.0).into());
    window_attributes.title = "User Interface".to_string();
    window_attributes.resizable = true;
    let mut executor = Executor::from_params(
        EventLoop::new().unwrap(),
        GraphicsContextParams {
            window_attributes,
            vsync: true,
        },
    );
    executor.add_plugin_constructor(GameConstructor);
    executor.run()
}
