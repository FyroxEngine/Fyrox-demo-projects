//! Executor with your game connected to it as a plugin.
use fyrox::core::wasm_bindgen::{self, prelude::*};
use fyrox::engine::executor::Executor;
use fyrox::event_loop::EventLoop;
use lightmap::Game;

#[wasm_bindgen]
pub fn main() {
    let mut executor = Executor::new(EventLoop::new().ok());
    executor.add_plugin(Game::default());
    executor.run()
}
