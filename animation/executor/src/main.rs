//! Executor with your game connected to it as a plugin.
use animation::Game;
use fyrox::{
    dpi::LogicalSize, engine::executor::Executor, engine::GraphicsContextParams,
    event_loop::EventLoop, window::WindowAttributes,
};

fn main() {
    let mut window_attributes = WindowAttributes::default();
    window_attributes.inner_size = Some(LogicalSize::new(1280.0, 720.0).into());
    window_attributes.title = "Animation".to_string();
    let mut executor = Executor::from_params(
        EventLoop::new().unwrap(),
        GraphicsContextParams {
            window_attributes,
            vsync: false,
            msaa_sample_count: None,
        },
    );
    executor.add_plugin(Game::default());
    executor.run()
}
