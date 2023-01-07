use iced::{Sandbox, Settings};

mod img_visualizer;

fn main() -> iced::Result {
    img_visualizer::FolderVisualizer::run(Settings::default())
}
