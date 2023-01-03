// use iced::Length;

use std::path::PathBuf;

use iced::{
    widget::{button, container, image, row, text},
    Element, Renderer,
};
use iced_native::{
    column,
    widget::{column, Button, Column},
};
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{FolderVisualizer, Steps};

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
}

#[derive(Debug, Clone)]
pub enum Step {
    WelcomeWithFolderChoose,
    Images,
    End,
}

impl<'a> Step {
    pub fn update(&'a mut self, msg: StepMessage, curr_idx: &mut usize, json_indices: Vec<i32>, json_values: Vec<bool>, correct_items: &mut Vec<bool>) -> (usize, Vec<i32>, Vec<bool>, Vec<bool>) {
        let mut json_obj = AnnotatedStore::default();
        json_obj.indices = json_indices;
        json_obj.values = json_values;
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
                // NOTE: Suppressing sound by default
                let _ = Notification::new()
                    .summary("Exported to output.json")
                    .body("See this is the detailed body")
                    .hint(notify_rust::Hint::SuppressSound(true))
                    .show();
            }
        };

        (*curr_idx, json_obj.indices.clone(), json_obj.values.clone(), correct_items.to_vec())
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

    pub fn container(title: &str) -> Column<'a, StepMessage, Renderer> {
        column![text(title).size(50)].spacing(20)
    }

    pub fn welcome() -> Column<'a, StepMessage, Renderer> {
        Self::container("Welcome!").push("Hi")
    }

    pub fn images(obj: &Steps) -> Column<'a, StepMessage, Renderer> {
        let export_btn = button(text("Export").size(40)).on_press(StepMessage::Export());
        let correct_btn =
            button(text("Mark as Correct").size(40)).on_press(StepMessage::MarkAsCorrect());
        let incorrect_btn =
            button(text("Mark as Incorrect").size(40)).on_press(StepMessage::MarkAsIncorrect());
        let previous_btn: Button<StepMessage, Renderer> =
            button(text("Previous").size(40)).on_press(StepMessage::Previous());
        let next_btn: Button<StepMessage, Renderer> =
            button(text("Next").size(40)).on_press(StepMessage::Next());
        let img_row = row![image::viewer(
            fetch_image(obj.all_images.clone(), &obj.curr_idx).unwrap()
        )]
        .align_items(iced::Alignment::Center)
        .width(iced::Length::Fill)
        .height(iced::Length::FillPortion(2));
        // .align_items(iced::Alignment::Center);

        // container(
            column![
                img_row,
                row![correct_btn, incorrect_btn].spacing(20).padding(10),
                row![previous_btn, export_btn, next_btn]
                    .spacing(20)
                    .padding(10)
            ]
            .align_items(iced::Alignment::Center)
        // )
        // .center_y()
        // .align_x(iced::alignment::Horizontal::Center)
        // .align_y(iced::alignment::Vertical::Center)
        // .into()
    }

    pub fn end() -> Column<'a, StepMessage, Renderer> {
        Self::container("End!").push("Hi")
    }

    pub fn title(&self) -> &str {
        match self {
            Step::WelcomeWithFolderChoose => "Welcome",
            Step::Images => "Images",
            Step::End => "End",
        }
    }
}

pub fn fetch_image(
    all_images: Vec<PathBuf>,
    curr_idx: &usize,
) -> Result<image::Handle, reqwest::Error> {
    // TODO: Set a default image to show that we are waiting for an image...// folder is empty
    // TODO: Handle cases when the curr_idx is out of bound/negative
    let path: PathBuf = all_images.get(*curr_idx).unwrap().to_owned();
    Ok(image::Handle::from_path(path))
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
    println!("json_obj: {:?}", json_obj);
    let obj: AnnotatedStore = serde_json::from_value(json_obj).unwrap();
    obj
}
