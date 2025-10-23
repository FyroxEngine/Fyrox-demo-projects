//! Executor with your game connected to it as a plugin.
use fyrox::engine::executor::Executor;
use fyrox::event_loop::EventLoop;
use platformer::{Game};

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut executor = Executor::new(Some(event_loop));
    executor.add_plugin(Game::default());
    executor.run()
}
