use iced::{Sandbox, Settings};
use winit::monitor::MonitorHandle;

mod img_visualizer;

fn main() -> iced::Result {
    img_visualizer::FolderVisualizer::run(Settings::default())
}
