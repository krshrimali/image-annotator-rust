use std::path::PathBuf;

use iced::{
    widget::{button, column, row, Row},
    widget::{container, image, text, Column},
    Element, Renderer, Sandbox,
};

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

// const BUTTON_TEXTS: Vec<&str> = vec!["Mark as correct", "<"];

// fn create_button<'a>(button_text: &'a str) -> Element<'a, render_image::Message, Renderer> {
//     // Row::with_children(
//     //     button_texts
//     //         .iter()
//     //         .map(|btn_text| container(row![button(*btn_text)]))
//     //         .collect()
//     // ).into()
//     row![button(button_text)].into()
// }

// fn create_buttons<'a>(buttons: Vec::<Element<'a, render_image::Message, Renderer>>) -> Element<'a, render_image::Message, Renderer> {
//     Row::with_children(buttons).into()
// }

// // let rows = vec![
// fn create_btn_row() -> Element<'static, render_image::Message> {
//     // let buttons: Element<render_image::Message> = BUTTON_TEXTS.iter().map(|btn_text| {
//     //     create_button(&btn_text)
//     // }).collect();
//     // buttons.into()
//     // let mut row: Row<'static, _, _> = Row::new().push(
//     //     BUTTON_TEXTS.iter().map(|btn_text| button(*btn_text).into())
//     // );
//     row.into()
// }

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
        let export_btn = row![button(text("Export").size(40))];
        let previous_btn = row![button(text("Previous").size(40))];
        let next_btn = row![button(text("Next").size(40))];
        let img_row = row![image::viewer(
            fetch_image(self.all_images.clone(), &self.curr_idx).unwrap()
        )]
        .align_items(iced::Alignment::Center)
        .width(iced::Length::Fill)
        .height(iced::Length::FillPortion(2));
        // .align_items(iced::Alignment::Center);

        container(
            column![
                img_row,
                row![previous_btn, export_btn, next_btn]
                    .spacing(20)
                    .padding(10)
            ]
            .align_items(iced::Alignment::Center),
        )
        .center_y()
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into()
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
