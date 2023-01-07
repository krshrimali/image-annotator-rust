// use iced::Length;

use std::path::PathBuf;

use rfd::FileDialog;

use iced::{
    alignment,
    widget::{button, container, container::Appearance, horizontal_space, row, text, Container},
    Color, Element, Length, Renderer,
};
use iced_native::{
    column,
    image::Handle,
    widget::{column, image, Button, Column},
};
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{get_all_images, FolderVisualizer, Steps};

#[derive(Debug, PartialEq, Clone, Eq, Copy)]
pub enum ThemeType {
    Light,
    Dark,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackPressed,
    NextPressed,
    StepMessage(StepMessage),
}

#[derive(Debug, Clone)]
pub enum StepMessage {
    Previous(),
    Next(),
    MarkAsCorrect(),
    MarkAsIncorrect(),
    Export(),
    ChooseFolderPath(),
}

#[derive(Debug, Clone)]
pub enum Step {
    WelcomeWithFolderChoose,
    Images,
    End,
}

struct ContainerCustomStyle {
    bg_color: iced::Background,
}

impl container::StyleSheet for ContainerCustomStyle {
    type Style = iced::theme::Theme;

    fn appearance(&self, _: &iced::Theme) -> container::Appearance {
        // TODO: Consider adding an option for theme here...
        // Also might consider bg as transparent instead...
        container::Appearance {
            border_radius: 2.0,
            border_width: 2.0,
            border_color: iced::Color::BLACK,
            background: Some(self.bg_color),
            ..Default::default()
        }
    }
}

impl<'a> Step {
    pub fn update(
        &'a mut self,
        msg: StepMessage,
        curr_idx: &mut usize,
        json_indices: Vec<i32>,
        json_values: Vec<bool>,
        correct_items: &mut [bool],
    ) -> (usize, Vec<i32>, Vec<bool>, Vec<bool>, Steps) {
        let mut json_obj = AnnotatedStore {
            indices: json_indices,
            values: json_values,
        };
        let mut new_steps_obj = Steps::default();

        match msg {
            StepMessage::Next() => {
                *curr_idx += 1;
            }
            StepMessage::Previous() => {
                *curr_idx -= 1;
            }
            StepMessage::MarkAsCorrect() => {
                correct_items[*curr_idx] = true;
                update_json(&mut json_obj, *curr_idx as i32, true);
            }
            StepMessage::MarkAsIncorrect() => {
                correct_items[*curr_idx] = false;
                update_json(&mut json_obj, *curr_idx as i32, false);
            }
            StepMessage::Export() => {
                write_json(&json_obj);
            }
            StepMessage::ChooseFolderPath() => {
                let new_folder_path = FileDialog::new()
                    .set_directory(".")
                    .pick_folder()
                    .unwrap_or_default();

                let new_folder_path_as_str =
                    new_folder_path.into_os_string().into_string().unwrap();
                let new_all_images = get_all_images(&new_folder_path_as_str);
                let new_json_obj: AnnotatedStore = init_json_obj(new_all_images.len());
                let mut steps_obj = Steps::new(
                    new_folder_path_as_str,
                    0,
                    new_all_images.clone(),
                    vec![],
                    new_json_obj,
                );
                steps_obj.correct_items = vec![false; new_all_images.len()];
                steps_obj.modified = true;
                json_obj.indices = vec![];
                json_obj.values = vec![];
                for idx in 0..new_all_images.len() {
                    json_obj.indices.push(idx as i32);
                    json_obj.values.push(false);
                }
                new_steps_obj = steps_obj;
            }
        };

        (
            *curr_idx,
            json_obj.indices.clone(),
            json_obj.values.clone(),
            correct_items.to_vec(),
            new_steps_obj,
        )
    }

    pub fn can_continue(&self) -> bool {
        match self {
            Step::WelcomeWithFolderChoose => true,
            Step::Images => true,
            Step::End => false,
        }
    }

    pub fn view(&self, obj: &Steps) -> Element<StepMessage> {
        match self {
            Step::WelcomeWithFolderChoose => Self::welcome(),
            Step::Images => Self::images(obj),
            Step::End => Self::end(),
        }
        .into()
    }

    pub fn container_(title: &str) -> Column<'a, StepMessage, Renderer> {
        column![text(title).size(50)].spacing(20)
    }

    pub fn welcome() -> Column<'a, StepMessage, Renderer> {
        let file_choose_button =
            button(text("Select folder")).on_press(StepMessage::ChooseFolderPath());

        column![container(row![file_choose_button])]
    }

    pub fn create_info(
        curr_idx: &usize,
        len_images: &usize,
        folder_path: &str,
        correct_items: &Vec<bool>,
    ) -> Container<'a, StepMessage, Renderer> {
        let curr_idx_text = text(format!("curr_idx: {}", curr_idx)).size(20);
        let len_images_text = text(format!("Total Images: {}", len_images)).size(20);
        let folder_path_text = text(format!("Folder Path: {}", folder_path)).size(20);
        let val = match correct_items[*curr_idx] {
            true => "Correct",
            false => "Incorrect",
        };
        let correct_item_text = text(format!("Current selection: {}", val)).size(20);

        container(
            row![
                curr_idx_text,
                horizontal_space(Length::Fill),
                len_images_text,
                horizontal_space(Length::Fill),
                folder_path_text,
                horizontal_space(Length::Fill),
                correct_item_text
            ]
            .padding(20),
        )
        .style(iced::theme::Container::Custom(Box::new(
            ContainerCustomStyle {
                bg_color: iced::Background::Color(iced::Color::WHITE),
            },
        )))
        .width(Length::Fill)
    }

    pub fn images(obj: &Steps) -> Column<'a, StepMessage, Renderer> {
        let export_btn = button(text("Export").size(20)).on_press(StepMessage::Export());
        let correct_btn =
            button(text("Mark as Correct").size(20)).on_press(StepMessage::MarkAsCorrect());
        let incorrect_btn =
            button(text("Mark as Incorrect").size(20)).on_press(StepMessage::MarkAsIncorrect());
        let previous_btn: Button<StepMessage, Renderer> =
            button(text("Previous Image").size(20)).on_press(StepMessage::Previous());
        let next_btn: Button<StepMessage, Renderer> =
            button(text("Next Image").size(20)).on_press(StepMessage::Next());
        // let img_row = container(row![image::viewer(
        //     fetch_image(obj.all_images.clone(), &obj.curr_idx).unwrap()
        // ).width(Length::Units(600)).height(Length::Units(800))])
        let img_row = container(row![image::viewer(
            fetch_image(obj.all_images.clone(), &obj.curr_idx).unwrap()
        )])
        .width(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(
            ContainerCustomStyle {
                bg_color: iced::Background::Color(iced::Color::WHITE),
            },
        )));

        let info_row = Self::create_info(
            &obj.curr_idx,
            &obj.all_images.len(),
            &obj.folder_path,
            &obj.correct_items,
        );

        // container(
        // border
        // TODO: Optional resize option for all the images
        column![container(column![
            // container(img_row).width(Length::FillPortion(2)).height(Length::FillPortion(2)),
            container(img_row),
            // .width(Length::Fill)
            // .height(Length::FillPortion(4)),
            container(
                row![correct_btn, horizontal_space(Length::Fill), incorrect_btn]
                    .spacing(20)
                    .padding(10)
            ),
            // .height(Length::FillPortion(1))
            // .width(Length::FillPortion(1)),
            info_row,
            row![
                previous_btn,
                horizontal_space(Length::Fill),
                export_btn,
                horizontal_space(Length::Fill),
                next_btn
            ]
            // .height(Length::FillPortion(1))
            // .width(Length::FillPortion(1))
            .spacing(20)
            .padding(10)
        ])
        .center_y()]
        // .align_items(iced::Alignment::Center)
        // )
        // .center_y()
        // .align_x(iced::alignment::Horizontal::Center)
        // .align_y(iced::alignment::Vertical::Center)
        // .into()
    }

    pub fn end() -> Column<'a, StepMessage, Renderer> {
        Self::container_("End!").push("Hi")
    }

    pub fn title(&self) -> &str {
        match self {
            Step::WelcomeWithFolderChoose => "Welcome",
            Step::Images => "Images",
            Step::End => "End",
        }
    }
}

pub fn fetch_image(all_images: Vec<PathBuf>, curr_idx: &usize) -> Result<Handle, reqwest::Error> {
    // TODO: Set a default image to show that we are waiting for an image...// folder is empty
    // TODO: Handle cases when the curr_idx is out of bound/negative
    let path: PathBuf = all_images.get(*curr_idx).unwrap().to_owned();
    Ok(Handle::from_path(path))
}

fn update_json(json_obj: &mut AnnotatedStore, idx_to_update: i32, new_value: bool) {
    json_obj.indices[idx_to_update as usize] = idx_to_update;
    json_obj.values[idx_to_update as usize] = new_value;
}

fn write_json(json_obj: &AnnotatedStore) {
    std::fs::write(
        "output.json",
        serde_json::to_string_pretty(json_obj).unwrap(),
    );
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct AnnotatedStore {
    pub indices: Vec<i32>,
    pub values: Vec<bool>,
}

pub fn init_json_obj(total_len: usize) -> AnnotatedStore {
    let init_vec = vec![0; total_len];
    let bool_vec = vec![false; total_len];
    // init_vec.iter().enumerate().map(|(idx, elem)| hash_map.insert(idx, elem));
    let json_obj = json!({"indices": init_vec, "values": bool_vec});
    let obj: AnnotatedStore = serde_json::from_value(json_obj).unwrap();
    obj
}
