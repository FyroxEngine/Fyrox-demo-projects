//! Executor with your game connected to it as a plugin.
use animation::GameConstructor;
use fyrox::{
    dpi::LogicalSize, engine::executor::Executor, engine::GraphicsContextParams,
    event_loop::EventLoop, window::WindowAttributes,
};

fn main() {
    let mut executor = Executor::from_params(
        EventLoop::new().unwrap(),
        GraphicsContextParams {
            window_attributes: WindowAttributes {
                inner_size: Some(LogicalSize::new(1280.0, 720.0).into()),
                title: "Animation".to_string(),
                ..Default::default()
            },
            vsync: false,
        },
    );
    executor.add_plugin_constructor(GameConstructor);
    executor.run()
}
