use std::path::PathBuf;

use iced::{
    widget::{button, column, row, Row},
    widget::{container, image, text, Button, Column},
    Element, Renderer, Sandbox, Theme,
};

use self::render_image::Message;

#[path = "render_image.rs"]
mod render_image;

#[derive(Default, Debug)]
pub struct FolderVisualizer {
    theme: Theme,
    folder_path: String,
    curr_idx: usize,
    all_images: Vec<PathBuf>,
    correct_items: Vec<bool>,
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
        let mut folder_obj = FolderVisualizer {
            theme: Theme::Dark,
            folder_path: folder_path.clone(),
            curr_idx: 0,
            all_images: get_all_images(folder_path),
            correct_items: vec![],
        };
        folder_obj.correct_items = vec![false; folder_obj.all_images.len()];
        folder_obj
    }

    fn title(&self) -> String {
        format!("Image {0}", self.curr_idx)
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Renderer> {
        let export_btn = button(text("Export").size(40));
        let correct_btn =
            button(text("Mark as Correct").size(40)).on_press(Message::MarkAsCorrect());
        let incorrect_btn =
            button(text("Mark as Incorrect").size(40)).on_press(Message::MarkAsIncorrect());
        let previous_btn: Button<Self::Message, Renderer> =
            button(text("Previous").size(40)).on_press(Message::Previous());
        let next_btn: Button<Self::Message, Renderer> =
            button(text("Next").size(40)).on_press(Message::Next());
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
                row![correct_btn, incorrect_btn].spacing(20).padding(10),
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

    fn update(&mut self, message: Self::Message) {
        match message {
            render_image::Message::ThemeChanged(theme) => {
                self.theme = match theme {
                    render_image::ThemeType::Dark => Theme::Dark,
                    render_image::ThemeType::Light => Theme::Light,
                }
            }
            render_image::Message::Next() => {
                self.curr_idx += 1;
            }
            render_image::Message::Previous() => {
                self.curr_idx -= 1;
            }
            render_image::Message::MarkAsCorrect() => {
                self.correct_items[self.curr_idx] = true;
            }
            render_image::Message::MarkAsIncorrect() => {
                self.correct_items[self.curr_idx] = false;
            }
        }
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
