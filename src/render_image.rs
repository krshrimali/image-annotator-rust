// use iced::Length;

use std::{collections::HashMap, path::PathBuf};

use rfd::FileDialog;

use iced::{
    theme,
    widget::{button, container, horizontal_space, row, text, text_input, Container},
    Element, Length, Renderer,
};
use iced_native::{
    column,
    image::Handle,
    widget::{image, Button, Column},
};
use serde::{Deserialize, Serialize};

use super::{get_all_images, Steps};

#[derive(PartialEq, Clone, Eq, Copy)]
pub enum ThemeType {
    Light,
    Dark,
}

#[derive(Clone, Debug)]
pub enum Message {
    BackPressed,
    NextPressed,
    StepMessage(StepMessage),
}

pub static mut FOLDER_FOUND: bool = false;

#[derive(Clone, Debug)]
pub enum StepMessage {
    Previous(),
    Next(),
    MarkAsCorrect(),
    MarkAsIncorrect(),
    ResetSelection(),
    Export(),
    ChooseFolderPath(),
    CommentAdded(String),
    CommentType(String),
}

#[derive(Clone, Debug)]
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
        folder_path: String,
        image_properties_map_vec: &mut HashMap<String, Vec<Properties>>,
        old_msg: String,
        old_incorrect_btn_clicked: bool,
        correct_items: &mut [Option<bool>],
    ) -> (
        usize,                            // new idx
        HashMap<String, Vec<Properties>>, // new prop map
        Option<bool>,                     // new annotation value
        Vec<Option<bool>>,                // new list of annotation values
        Option<String>,
        Steps, // revised Steps
    ) {
        let mut new_annotation: Option<bool> = match image_properties_map_vec.get(&folder_path) {
            Some(vec_prop_map) => {
                if let Some(prop_map) = vec_prop_map.get(*curr_idx) {
                    prop_map.annotation
                } else {
                    None
                }
            }
            None => None,
        };
        let mut json_obj = AnnotatedStore {
            image_to_properties_map: image_properties_map_vec.clone(),
        };
        let mut new_steps_obj = Steps {
            incorrect_btn_clicked: old_incorrect_btn_clicked,
            new_message: old_msg,
            ..Default::default()
        };

        let mut new_comment = match image_properties_map_vec.get(&folder_path) {
            Some(vec_prop_map) => {
                if let Some(prop_map) = vec_prop_map.get(*curr_idx) {
                    if let Some(comment) = &prop_map.comments {
                        Some(comment.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            None => None,
        };

        match msg {
            StepMessage::Next() => {
                *curr_idx += 1;
                new_steps_obj.incorrect_btn_clicked = false;
            }
            StepMessage::Previous() => {
                *curr_idx -= 1;
                new_steps_obj.incorrect_btn_clicked = false;
            }
            StepMessage::MarkAsCorrect() => {
                if curr_idx < &mut correct_items.len() {
                    correct_items[*curr_idx] = Some(true);
                    new_annotation = Some(true);
                    new_steps_obj.incorrect_btn_clicked = false;
                }
            }
            StepMessage::MarkAsIncorrect() => {
                if curr_idx < &mut correct_items.len() {
                    correct_items[*curr_idx] = Some(false);
                    new_annotation = Some(false);
                    new_steps_obj.incorrect_btn_clicked = true;
                }
            }
            StepMessage::ResetSelection() => {
                if curr_idx < &mut correct_items.len() {
                    correct_items[*curr_idx] = None;
                    new_annotation = None;
                    new_steps_obj.incorrect_btn_clicked = false;
                }
            }
            StepMessage::Export() => {
                write_json(&json_obj);
                new_steps_obj.incorrect_btn_clicked = false;
            }
            StepMessage::CommentAdded(entered_comment) => {
                new_steps_obj.new_message.clear();
                new_steps_obj.new_message = entered_comment;
                new_steps_obj.incorrect_btn_clicked = false;
                // NOTE: Enable this if you want to disable "Send" button after clicking it (make msg required)
                // new_comment = None;
            }
            StepMessage::CommentType(entered_comment) => {
                new_steps_obj.new_message = entered_comment.clone();
                new_comment = Some(entered_comment);
            }
            StepMessage::ChooseFolderPath() => {
                let new_folder_path = FileDialog::new().set_directory(".").pick_folder();

                if let Some(valid_path) = new_folder_path {
                    let new_folder_path_as_str = valid_path.into_os_string().into_string().unwrap();
                    let new_all_images_paths = get_all_images(&new_folder_path_as_str);

                    let new_json_obj: AnnotatedStore =
                        init_json_obj(new_folder_path_as_str.clone(), new_all_images_paths.clone());

                    let mut steps_obj = Steps::new(
                        new_folder_path_as_str,
                        0,
                        new_all_images_paths.clone(),
                        vec![],
                        new_json_obj.clone(),
                    );

                    steps_obj.correct_items = vec![None; new_all_images_paths.len()];
                    steps_obj.modified = true;
                    steps_obj.btn_status = true;
                    new_steps_obj = steps_obj;

                    json_obj.image_to_properties_map = new_json_obj.image_to_properties_map;

                    unsafe {
                        FOLDER_FOUND = true;
                    }
                } else {
                    new_steps_obj.btn_status = false;
                    unsafe {
                        FOLDER_FOUND = false;
                    }
                }
            }
        };

        (
            *curr_idx,
            json_obj.image_to_properties_map,
            new_annotation,
            correct_items.to_vec(),
            new_comment,
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
            Step::WelcomeWithFolderChoose => Self::welcome().into(),
            Step::Images => Self::images(obj).into(),
            Step::End => Self::end().into(),
        }
    }

    pub fn container_(title: &str) -> Column<'a, StepMessage, Renderer> {
        column![text(title).size(50)].spacing(20)
    }

    pub fn welcome() -> Column<'a, StepMessage, Renderer> {
        unsafe {
            if FOLDER_FOUND {
                let file_choose_button = button(text("Select folder"))
                    .on_press(StepMessage::ChooseFolderPath())
                    .style(theme::Button::Secondary);
                column![container(row![file_choose_button])]
            } else {
                let file_choose_button = button(text("Select folder"))
                    .on_press(StepMessage::ChooseFolderPath())
                    .style(theme::Button::Primary);
                column![container(row![file_choose_button])]
            }
        }
    }

    pub fn create_info(
        curr_idx: &usize,
        len_images: &usize,
        folder_path: &str,
        correct_items: &[Option<bool>],
    ) -> Container<'a, StepMessage, Renderer> {
        let curr_idx_text = text(format!("curr_idx: {}", curr_idx)).size(20);
        let len_images_text = text(format!("Total Images: {}", len_images)).size(20);
        let folder_path_text = text(format!("Folder Path: {}", folder_path)).size(20);
        let mut val: &str = "No Image";
        if *curr_idx < correct_items.len() {
            val = match correct_items[*curr_idx] {
                Some(true) => "Correct",
                Some(false) => "Incorrect",
                None => "Not selected yet",
            };
        }
        let correct_item_text = text(format!("Current selection: {}", val)).size(20);

        container(column![
            row![
                curr_idx_text,
                horizontal_space(Length::Fill),
                len_images_text,
                horizontal_space(Length::Fill),
                correct_item_text
            ]
            .padding(10),
            row![
                horizontal_space(Length::Fill),
                folder_path_text,
                horizontal_space(Length::Fill),
            ]
            .padding(5),
        ])
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
        let reset_btn =
            button(text("Reset Selection").size(20)).on_press(StepMessage::ResetSelection());
        let mut previous_btn: Option<Button<StepMessage, Renderer>> =
            Some(button(text("Previous Image").size(20)).on_press(StepMessage::Previous()));
        let mut next_btn: Option<Button<StepMessage, Renderer>> =
            Some(button(text("Next Image").size(20)).on_press(StepMessage::Next()));

        match obj.is_next_image_available() {
            true => {
                next_btn = Some(next_btn.unwrap().style(iced::theme::Button::Primary));
            }
            false => {
                next_btn = None;
            }
        }

        match obj.is_previous_image_available() {
            true => {
                previous_btn = Some(previous_btn.unwrap().style(iced::theme::Button::Primary));
            }
            false => {
                previous_btn = None;
            }
        };

        let new_message_input = {
            let mut input = text_input(
                "(Optional) Type your reason here...",
                &obj.new_message,
                StepMessage::CommentType,
            )
            .padding(10);

            let mut button = button(
                text("Done")
                    .height(Length::Fill)
                    .vertical_alignment(iced::alignment::Vertical::Center),
            )
            .padding([0, 20]);

            if let Some(valid_msg) = msg_check(obj.new_message.clone()) {
                input = input.on_submit(StepMessage::CommentAdded(valid_msg.clone()));
                button = button.on_press(StepMessage::CommentAdded(valid_msg));
            }

            row![input, button]
                .spacing(10)
                .align_items(iced::Alignment::Fill)
        };
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
        let next_prev_buttons_row = match next_btn {
            Some(next_btn_valid) => match previous_btn {
                Some(prev_btn_valid) => row![
                    prev_btn_valid,
                    horizontal_space(Length::Fill),
                    export_btn,
                    horizontal_space(Length::Fill),
                    next_btn_valid,
                    horizontal_space(Length::Fill),
                ],
                None => row![
                    horizontal_space(Length::Fill),
                    export_btn,
                    horizontal_space(Length::Fill),
                    next_btn_valid,
                    horizontal_space(Length::Fill),
                ],
            },
            None => match previous_btn {
                Some(prev_btn_valid) => row![
                    prev_btn_valid,
                    horizontal_space(Length::Fill),
                    export_btn,
                    horizontal_space(Length::Fill),
                ],
                None => row![
                    horizontal_space(Length::Fill),
                    export_btn,
                    horizontal_space(Length::Fill),
                ],
            },
        };
        let image_option_buttons = match obj.incorrect_btn_clicked {
            false => container(
                row![
                    correct_btn,
                    horizontal_space(Length::Fill),
                    reset_btn,
                    horizontal_space(Length::Fill),
                    incorrect_btn,
                ]
                .spacing(20)
                .padding(10),
            ),
            true => container(
                row![
                    correct_btn,
                    horizontal_space(Length::Fill),
                    reset_btn,
                    horizontal_space(Length::Fill),
                    new_message_input,
                ]
                .spacing(20)
                .padding(10),
            ),
        };

        column![container(column![
            // container(img_row).width(Length::FillPortion(2)).height(Length::FillPortion(2)),
            container(img_row),
            // .width(Length::Fill)
            // .height(Length::FillPortion(4)),
            // .height(Length::FillPortion(1))
            // .width(Length::FillPortion(1)),
            image_option_buttons,
            info_row,
            // .height(Length::FillPortion(1))
            // .width(Length::FillPortion(1))
            next_prev_buttons_row.spacing(20).padding(10)
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
        column![container(Self::container_("End!")).center_x().center_y()]
    }

    pub fn title(&self) -> String {
        match self {
            Step::WelcomeWithFolderChoose => "Welcome".to_string(),
            Step::Images => "Images".to_string(),
            Step::End => "End".to_string(),
        }
    }
}

pub fn fetch_image(all_images: Vec<PathBuf>, curr_idx: &usize) -> Result<Handle, reqwest::Error> {
    // TODO: Set a default image to show that we are waiting for an image...// folder is empty
    // TODO: Handle cases when the curr_idx is out of bound/negative
    let path: PathBuf = all_images
        .get(*curr_idx)
        .unwrap_or(&PathBuf::default())
        .to_owned();
    Ok(Handle::from_path(path))
}

fn write_json(json_obj: &AnnotatedStore) {
    let res = std::fs::write(
        "output.json",
        serde_json::to_string_pretty(json_obj).unwrap_or_default(),
    );
    match res {
        Ok(_) => println!("Done"),
        Err(e) => println!("Error: {}", e),
    };
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Properties {
    pub index: usize,
    pub image_path: String,
    pub annotation: Option<bool>,
    pub comments: Option<String>,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct AnnotatedStore {
    pub image_to_properties_map: HashMap<String, Vec<Properties>>,
}

pub fn init_json_obj(folder_path: String, all_paths: Vec<PathBuf>) -> AnnotatedStore {
    let mut image_to_properties_map = HashMap::new();
    let mut vec_maps = vec![];
    for (idx, path) in all_paths.iter().enumerate() {
        let path_str = path.to_str().unwrap().to_string();
        let selected_option = None;
        let properties = Properties {
            index: idx,
            image_path: path_str,
            annotation: selected_option,
            comments: None, // TODO
        };
        vec_maps.push(properties);
    }
    image_to_properties_map.insert(folder_path, vec_maps);

    AnnotatedStore {
        image_to_properties_map,
    }
}

pub fn msg_check(msg: String) -> Option<String> {
    // if msg.is_() {
    //     None
    // } else {
    Some(msg)
    // }
}
