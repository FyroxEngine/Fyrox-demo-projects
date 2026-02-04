//! Editor with your game connected to it as a plugin.
use fyrox::event_loop::EventLoop;
use fyroxed_base::{Editor, StartupData};
use lightmap::Game;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut editor = Editor::new(Some(StartupData {
        working_directory: Default::default(),
        scenes: vec!["data/Sponza/sponza.rgs".into()],
        named_objects: false,
    }));
    editor.add_game_plugin(Game);
    editor.run(event_loop)
}
