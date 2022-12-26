use std::path::PathBuf;

use iced::{
    widget::{button, row, Row},
    widget::{container, image},
    Element, Renderer, Sandbox,
};
use iced_native::Renderer;

#[path = "render_image.rs"]
mod render_image;

#[derive(Default, Debug)]
pub struct FolderVisualizer {
    folder_path: String,
    curr_idx: usize,
    all_images: Vec<PathBuf>,
}

fn fetch_image(
    all_images: Vec<PathBuf>,
    curr_idx: &usize,
) -> Result<image::Handle, reqwest::Error> {
    // TODO: Set a default image to show that we are waiting for an image...// folder is empty
    // TODO: Handle cases when the curr_idx is out of bound/negative
    let path: PathBuf = all_images.get(*curr_idx).unwrap().to_owned();
    Ok(image::Handle::from_path(path))
}

fn get_all_images(folder_path: String) -> Vec<PathBuf> {
    // TODO: Handle dir validation here
    let paths = std::fs::read_dir(folder_path).unwrap();
    let mut output: Vec<PathBuf> = vec![];
    for path in paths {
        output.push(path.unwrap().path().as_path().to_owned());
    }
    output
}

impl Sandbox for FolderVisualizer {
    type Message = render_image::Message;

    fn new() -> FolderVisualizer {
        let folder_path: String = "sample_folder".into();
        FolderVisualizer {
            folder_path: folder_path.clone(),
            curr_idx: 0,
            all_images: get_all_images(folder_path),
        }
    }

    fn title(&self) -> String {
        String::from("Image Annotator")
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Renderer> {
        fn create_buttons<'a>(_: Vec<&str>) -> Element<'a, render_image::Message, Renderer> {
            // Row::with_children(
            //     button_texts
            //         .iter()
            //         .map(|btn_text| button("sample".to_string()))
            //         .collect(),
            // )
            row![button("sample")].into()
        }

        let rows = vec![
            row![image::viewer(
                fetch_image(self.all_images.clone(), &self.curr_idx).unwrap()
            )],
            row![create_buttons(vec!["Mark as correct", "<"])],
        ];

        // container()
            // row![create_buttons(vec![
            //     "Mark as Correct",
            //     "<",
            //     "Reset",
            //     ">",
            //     "Mark as Wrong"
            // ])],
    }

    fn update(&mut self, _: Self::Message) {
        // match message {
        //     render_image::Message::ThemeChanged(theme) => {
        //         self.theme = match theme {
        //             render_image::ThemeType::Dark => Theme::Dark,
        //             render_image::ThemeType::Light => Theme::Light,
        //         }
        //     }
        // }
    }
}
