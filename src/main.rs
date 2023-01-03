use iced::{Sandbox, Settings};

mod img_visualizer;

fn main() -> iced::Result {
    // let mut settings = Settings::default();
    // TODO: Need to make this size dynamic
    // settings.window.size = (1600, 800);
    img_visualizer::FolderVisualizer::run(Settings::default())
}
