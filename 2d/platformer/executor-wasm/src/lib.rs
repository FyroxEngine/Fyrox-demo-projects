//! Executor with your game connected to it as a plugin.
use fyrox::core::wasm_bindgen::{self, prelude::*};
use fyrox::dpi::LogicalSize;
use fyrox::engine::executor::Executor;
use fyrox::engine::GraphicsContextParams;
use fyrox::event_loop::EventLoop;
use fyrox::window::WindowAttributes;
use platformer::GameConstructor;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(msg: String);

    type Error;

    #[wasm_bindgen(constructor)]
    fn new() -> Error;

    #[wasm_bindgen(structural, method, getter)]
    fn stack(error: &Error) -> String;
}

fn custom_panic_hook(info: &std::panic::PanicInfo) {
    let mut msg = info.to_string();
    msg.push_str("\n\nStack:\n\n");
    let e = Error::new();
    let stack = e.stack();
    msg.push_str(&stack);
    msg.push_str("\n\n");
    error(msg);
}

#[inline]
pub fn set_panic_hook() {
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        std::panic::set_hook(Box::new(custom_panic_hook));
    });
}

#[wasm_bindgen]
pub fn main() {
    set_panic_hook();
    let mut executor = Executor::from_params(
        EventLoop::new(),
        GraphicsContextParams {
            window_attributes: WindowAttributes {
                inner_size: Some(LogicalSize::new(800, 600).into()),
                resizable: true,
                title: "Platformer".to_string(),
                ..Default::default()
            },
            vsync: true,
        },
    );
    executor.add_plugin_constructor(GameConstructor);
    executor.run()
}
