use chrono::Local;
use std::{collections::HashMap, panic, path::PathBuf};

use once_cell::sync::Lazy;
use rfd::FileDialog;

use iced::{
    theme,
    widget::{
        button, container, horizontal_space, image, radio, row, text, text_input, Button, Column,
        Container,
    },
    Element, Length, Renderer,
};
use iced_native::{column, image::Handle};
use serde::{Deserialize, Serialize};

use super::{get_all_images, Steps};

#[derive(PartialEq, Clone, Eq, Copy, Debug)]
pub enum ThemeType {
    Light,
    Dark,
    Custom,
}

#[derive(Clone, Debug)]
pub enum Message {
    BackPressed,
    NextPressed,
    ImageStepMessage(ImageStepMessage),
    ThemeChanged(ThemeType),
}

pub static mut FOLDER_FOUND: bool = false;
// TODO: This is kinda unsafe, curious to know when home_dir() will return None
pub static mut OUTPUT_PATH: Lazy<String> = Lazy::new(|| {
    home::home_dir()
        .unwrap()
        .as_path()
        .join("output.json")
        .as_path()
        .to_str()
        .unwrap()
        .to_string()
});

#[derive(Clone, Debug)]
pub enum ImageStepMessage {
    Previous(),
    Next(),
    MarkAsCorrect(),
    MarkAsIncorrect(),
    ResetSelection(),
    Export(),
    ChooseFolderPath(),
    CommentAdded(String),
    CommentType(String),
    ThemeChanged(ThemeType),
}

#[derive(Clone, Debug)]
pub enum Step {
    WelcomeWithFolderChoose,
    Images,
    End,
}

struct ContainerCustomStyle {
    curr_theme: theme::Theme,
    bg_color: iced::Background,
}

impl container::StyleSheet for ContainerCustomStyle {
    type Style = theme::Theme;
    fn appearance(&self, _: &iced::Theme) -> container::Appearance {
        let (text_color, bg) = match &self.curr_theme {
            iced::Theme::Light => (
                iced::Color::BLACK,
                Some(iced_core::Background::Color(iced::Color::TRANSPARENT)),
            ),
            iced::Theme::Dark => (
                iced::Color::WHITE,
                Some(iced_core::Background::Color(iced::Color::TRANSPARENT)),
            ),
            iced::Theme::Custom(_) => (
                iced::Color::BLACK,
                Some(iced_core::Background::Color(iced::Color::TRANSPARENT)),
            ),
        };
        container::Appearance {
            text_color: Some(text_color),
            background: bg,
            border_radius: 2.0,
            border_width: 2.0,
            border_color: iced::Color::BLACK,
        }
    }
}

impl<'a> Step {
    pub fn update(
        &'a mut self,
        msg: ImageStepMessage,
        curr_idx: &mut usize,
        folder_path: String,
        image_properties_map_vec: &mut HashMap<String, Vec<Properties>>,
        old_msg: String,
        old_incorrect_btn_clicked: bool,
        correct_items: &mut [Option<bool>],
        theme: &theme::Theme,
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
            theme: theme.clone(),
            ..Default::default()
        };

        let mut new_comment = match image_properties_map_vec.get(&folder_path) {
            Some(vec_prop_map) => {
                if let Some(prop_map) = vec_prop_map.get(*curr_idx) {
                    prop_map
                        .comments
                        .as_ref()
                        .map(|comment| comment.to_string())
                } else {
                    None
                }
            }
            None => None,
        };

        match msg {
            ImageStepMessage::Next() => {
                *curr_idx += 1;
                new_steps_obj.incorrect_btn_clicked = false;
            }
            ImageStepMessage::Previous() => {
                *curr_idx -= 1;
                new_steps_obj.incorrect_btn_clicked = false;
            }
            ImageStepMessage::MarkAsCorrect() => {
                if curr_idx < &mut correct_items.len() {
                    correct_items[*curr_idx] = Some(true);
                    new_annotation = Some(true);
                    new_steps_obj.incorrect_btn_clicked = false;
                    new_comment = None;
                }
            }
            ImageStepMessage::MarkAsIncorrect() => {
                if curr_idx < &mut correct_items.len() {
                    correct_items[*curr_idx] = Some(false);
                    new_annotation = Some(false);
                    new_steps_obj.incorrect_btn_clicked = true;
                }
            }
            ImageStepMessage::ResetSelection() => {
                if curr_idx < &mut correct_items.len() {
                    correct_items[*curr_idx] = None;
                    new_annotation = None;
                    new_steps_obj.incorrect_btn_clicked = false;
                }
            }
            ImageStepMessage::Export() => {
                write_json(&json_obj);
                new_steps_obj.incorrect_btn_clicked = false;
            }
            ImageStepMessage::CommentAdded(entered_comment) => {
                new_steps_obj.new_message.clear();
                new_steps_obj.new_message = entered_comment;
                new_steps_obj.incorrect_btn_clicked = false;
                // NOTE: Enable this if you want to disable "Send" button after clicking it (make msg required)
                // new_comment = None;
            }
            ImageStepMessage::CommentType(entered_comment) => {
                new_steps_obj.new_message = entered_comment.clone();
                new_comment = Some(entered_comment);
            }
            ImageStepMessage::ChooseFolderPath() => {
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
            ImageStepMessage::ThemeChanged(theme) => {
                let new_theme = match theme {
                    ThemeType::Dark => iced::Theme::Dark,
                    ThemeType::Light => iced::Theme::Light,
                    ThemeType::Custom => iced::Theme::custom(theme::Palette {
                        background: iced::Color::from_rgb(1.0, 0.9, 1.0),
                        text: iced::Color::BLACK,
                        primary: iced::Color::from_rgb(0.5, 0.5, 0.0),
                        success: iced::Color::from_rgb(0.0, 1.0, 0.0),
                        danger: iced::Color::from_rgb(1.0, 0.0, 0.0),
                    }),
                };
                new_steps_obj.theme = new_theme;
                new_steps_obj.theme_changed = true;
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

    pub fn view(&self, obj: &Steps) -> Element<ImageStepMessage> {
        match self {
            Step::WelcomeWithFolderChoose => Self::welcome(obj).into(),
            Step::Images => Self::images(obj, &obj.theme).into(),
            Step::End => Self::end().into(),
        }
    }

    pub fn container_(title: &str) -> Column<'a, ImageStepMessage, Renderer> {
        column![text(title).size(50)].spacing(20)
    }

    pub fn welcome(obj: &Steps) -> Column<'a, ImageStepMessage, Renderer> {
        let choose_theme = [ThemeType::Dark, ThemeType::Light, ThemeType::Custom]
            .iter()
            .fold(
                row![text("Choose a theme:")].spacing(10),
                |column: iced_native::widget::row::Row<'_, ImageStepMessage, Renderer>, theme| {
                    column.push(radio(
                        format!("{:?}", theme),
                        *theme,
                        Some(match obj.theme {
                            iced::Theme::Dark => ThemeType::Dark,
                            iced::Theme::Light => ThemeType::Light,
                            iced::Theme::Custom { .. } => ThemeType::Custom,
                        }),
                        ImageStepMessage::ThemeChanged,
                    ))
                },
            );

        let choose_theme_content = column![choose_theme]
            .spacing(20)
            .padding(20)
            .max_width(600)
            .width(Length::Fill);

        unsafe {
            if FOLDER_FOUND {
                let file_choose_button = button(text("Select folder"))
                    .on_press(ImageStepMessage::ChooseFolderPath())
                    .style(theme::Button::Secondary);
                column![
                    container(row![choose_theme_content
                        .width(Length::Fill)
                        .align_items(iced::Alignment::Start)]),
                    container(row![file_choose_button])
                ]
            } else {
                let file_choose_button = button(text("Select folder"))
                    .on_press(ImageStepMessage::ChooseFolderPath())
                    .style(theme::Button::Primary);
                column![
                    container(row![choose_theme_content
                        .width(Length::Fill)
                        .align_items(iced::Alignment::Start)]),
                    container(row![file_choose_button])
                ]
            }
        }
    }

    pub fn create_info(
        curr_idx: &usize,
        len_images: &usize,
        folder_path: &str,
        correct_items: &[Option<bool>],
        image_file_name: String,
        theme: &theme::Theme,
    ) -> Container<'a, ImageStepMessage, Renderer> {
        let curr_idx_text = text(format!("Current Item: {}", curr_idx + 1)).size(20);
        let len_images_text = text(format!("Total Images: {}", len_images)).size(20);
        let folder_path_text = text(format!("Folder Path: {}", folder_path)).size(20);
        let image_file_path_text = text(format!("Image file name: {}", image_file_name)).size(20);
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
                correct_item_text,
                horizontal_space(Length::Fill),
                image_file_path_text,
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
                curr_theme: theme.clone(),
                bg_color: iced::Background::Color(iced::Color::WHITE),
            },
        )))
        .width(Length::Fill)
    }

    pub fn images(obj: &Steps, theme: &theme::Theme) -> Column<'a, ImageStepMessage, Renderer> {
        let choose_theme = [ThemeType::Dark, ThemeType::Light, ThemeType::Custom]
            .iter()
            .fold(
                row![text("Choose a theme:")].spacing(10),
                |column: iced_native::widget::row::Row<'_, ImageStepMessage, Renderer>, theme| {
                    column.push(radio(
                        format!("{:?}", theme),
                        *theme,
                        Some(match obj.theme {
                            iced::Theme::Dark => ThemeType::Dark,
                            iced::Theme::Light => ThemeType::Light,
                            iced::Theme::Custom { .. } => ThemeType::Custom,
                        }),
                        ImageStepMessage::ThemeChanged,
                    ))
                },
            );

        let choose_theme_content = column![choose_theme]
            .spacing(20)
            .padding(20)
            .max_width(600)
            .width(Length::Fill);
        let export_btn = button(text("Export").size(20)).on_press(ImageStepMessage::Export());
        let correct_btn =
            button(text("Mark as Correct").size(20)).on_press(ImageStepMessage::MarkAsCorrect());
        let incorrect_btn = button(text("Mark as Incorrect").size(20))
            .on_press(ImageStepMessage::MarkAsIncorrect());
        let reset_btn =
            button(text("Reset Selection").size(20)).on_press(ImageStepMessage::ResetSelection());
        let mut previous_btn: Option<Button<ImageStepMessage, Renderer>> =
            Some(button(text("Previous Image").size(20)).on_press(ImageStepMessage::Previous()));
        let mut next_btn: Option<Button<ImageStepMessage, Renderer>> =
            Some(button(text("Next Image").size(20)).on_press(ImageStepMessage::Next()));

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
                ImageStepMessage::CommentType,
            )
            .padding(10);

            let mut button = button(
                text("Done")
                    .height(Length::Fill)
                    .vertical_alignment(iced::alignment::Vertical::Center),
            )
            .padding([0, 20]);

            if let Some(valid_msg) = msg_check(obj.new_message.clone()) {
                input = input.on_submit(ImageStepMessage::CommentAdded(valid_msg.clone()));
                button = button.on_press(ImageStepMessage::CommentAdded(valid_msg));
            }

            row![input, button]
                .spacing(10)
                .align_items(iced::Alignment::Fill)
        };
        let img_handle = fetch_image(obj.all_images.clone(), &obj.curr_idx);

        let mut error_msg: Option<String> = None;
        let img_viewer = match img_handle {
            Ok(valid_img_handle) => Some(image::viewer(valid_img_handle)),
            Err(e) => {
                error_msg = Some(e.to_string());
                None
            }
        };

        let file_name = obj.all_images[obj.curr_idx]
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let info_row = Self::create_info(
            &obj.curr_idx,
            &obj.all_images.len(),
            &obj.folder_path,
            &obj.correct_items,
            file_name.to_string(),
            theme,
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

        match img_viewer {
            Some(valid_img_viewer) => {
                column![
                    container(row![choose_theme_content
                        .width(Length::Fill)
                        .align_items(iced::Alignment::Start)]),
                    container(row![
                        horizontal_space(Length::Fill),
                        valid_img_viewer,
                        horizontal_space(Length::Fill)
                    ]),
                    image_option_buttons,
                    info_row,
                    next_prev_buttons_row.spacing(20).padding(10)
                ]
            }
            None => column![
                container(row![choose_theme_content
                    .width(Length::Fill)
                    .align_items(iced::Alignment::Start)]),
                container(row![
                    horizontal_space(Length::Fill),
                    text(error_msg.unwrap_or_default()),
                    horizontal_space(Length::Fill)
                ])
                .style(iced::theme::Container::Custom(Box::new(
                    ContainerCustomStyle {
                        curr_theme: theme.clone(),
                        bg_color: iced::Background::Color(iced::Color::WHITE),
                    },
                )))
                .height(Length::Units(400)) // TOOD: Instead of hard-coding this year, find current windows' height - and make this 40% of that height.
                .center_y(),
                image_option_buttons,
                info_row,
                next_prev_buttons_row.spacing(20).padding(10)
            ],
        }
    }

    pub fn end() -> Column<'a, ImageStepMessage, Renderer> {
        column![container(Self::container_("End!")).center_x().center_y()]
    }

    pub fn title(&self) -> String {
        match self {
            Step::WelcomeWithFolderChoose => "Welcome".to_string(),
            Step::Images => "Images".to_string(),
            Step::End => "End".to_string(),
        }
    }

    // fn theme(&self) -> iced::Theme {
    //     self.theme.clone()
    // }
}

pub fn fetch_image(all_images: Vec<PathBuf>, curr_idx: &usize) -> Result<Handle, std::io::Error> {
    // TODO: Set a default image to show that we are waiting for an image...// folder is empty
    let formatted_string = format!("Invalid index: {}", curr_idx);
    let path: PathBuf = all_images
        .get(*curr_idx)
        .unwrap_or_else(|| panic!("{}", formatted_string.to_string()))
        .to_owned();
    let img_validation_result = imghdr::from_file(path.clone());
    match img_validation_result {
        Ok(if_none) => match if_none {
            Some(_) => Ok(Handle::from_path(path)),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid file, please check if it is a valid image file!",
            )),
        },
        Err(e) => Err(e),
    }
}

pub fn load_json_and_update(path_str: &String, json_obj: &AnnotatedStore) {
    if std::path::Path::new(path_str).exists() {
        let content = std::fs::read_to_string(path_str).ok().unwrap();
        let mut v: AnnotatedStore = serde_json::from_str(&content).ok().unwrap();
        // println!("Prop: {:?}", json_obj.image_to_properties_map);
        for (folder_path, val) in json_obj.image_to_properties_map.iter() {
            v.image_to_properties_map
                .insert(folder_path.to_string(), val.to_vec());
        }
        let res = std::fs::write(
            path_str,
            serde_json::to_string_pretty(&v).unwrap_or_default(),
        );
        match res {
            Ok(_) => println!("Done"),
            Err(_) => println!("Error"),
        }
    } else {
        let res = std::fs::write(
            path_str,
            serde_json::to_string_pretty(json_obj).unwrap_or_default(),
        );
        match res {
            Ok(_) => println!("Done"),
            Err(_) => println!("Error"),
        }
    }
}

fn write_json(json_obj: &AnnotatedStore) {
    unsafe {
        load_json_and_update(&OUTPUT_PATH, json_obj);
    }
}

#[derive(Deserialize, Serialize, Default, Debug, Clone, Eq, PartialEq)]
pub struct Properties {
    pub index: usize,
    pub image_path: String,
    pub annotation: Option<bool>,
    pub comments: Option<String>,
    pub last_updated: Option<String>,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone, Eq, PartialEq)]
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
            comments: None,
            last_updated: Some(Local::now().to_string()),
        };
        vec_maps.push(properties);
    }
    image_to_properties_map.insert(folder_path, vec_maps);

    AnnotatedStore {
        image_to_properties_map,
    }
}

pub fn msg_check(msg: String) -> Option<String> {
    // NOTE: Change this if you want to disable button by default
    // if msg.is_() {
    //     None
    // } else {
    Some(msg)
    // }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::sync::Once;

    extern crate image;
    use image::{ImageBuffer, Rgb};

    static INIT: Once = Once::new();

    pub fn initialize() {
        INIT.call_once(|| {
            // create sample test image
            let _ = std::fs::create_dir("test");
            // Just creating a sample image
            let image = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(10, 10);
            image.save("test/sample.jpg").unwrap();
        });
    }

    #[test]
    #[should_panic(expected = "Invalid index")]
    fn test_fetch_image_invalid_curr_idx() {
        initialize();
        let curr_idx: i32 = -1;
        let path_buf = PathBuf::from_str("test/sample.jpg").unwrap();
        let all_images = vec![path_buf];
        let _ = fetch_image(all_images, &(curr_idx as usize));
    }

    #[test]
    fn test_fetch_image_valid_curr_idx_invalid_image() {
        initialize();
        let curr_idx: usize = 0;
        let path_buf = PathBuf::from_str("test/invalid_sample.jpg").unwrap();
        let all_images = vec![path_buf];
        let result = fetch_image(all_images, &curr_idx);
        assert!(result.is_err());
        // TODO: This assertion fails on Windows, check what could be the error on Windows
        // assert!(result
        //     .err()
        //     .unwrap()
        //     .to_string()
        //     .contains("No such file or directory"));
    }

    #[test]
    fn test_fetch_image_valid_curr_idx_valid_image() {
        initialize();
        let curr_idx: usize = 0;
        let path_buf = PathBuf::from_str("test/sample.jpg").unwrap();
        let all_images = vec![path_buf];
        let result = fetch_image(all_images, &curr_idx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_json_obj_valid() {
        initialize();
        let folder_path = "test".to_string();
        let all_paths: Vec<PathBuf> = vec![PathBuf::from_str("test/sample.jpg").unwrap()];
        let json_obj = init_json_obj(folder_path.clone(), all_paths);
        // Getting rid of timestamp for now, hard to compare
        let last_updated_time = json_obj
            .image_to_properties_map
            .get(&folder_path)
            .unwrap()
            .get(0)
            .unwrap()
            .last_updated
            .as_ref();
        let expected_json_obj = AnnotatedStore {
            image_to_properties_map: HashMap::from([(
                folder_path,
                vec![Properties {
                    index: 0,
                    image_path: "test/sample.jpg".to_string(),
                    annotation: None,
                    comments: None,
                    last_updated: last_updated_time.cloned(),
                }],
            )]),
        };
        assert_eq!(json_obj, expected_json_obj);
    }

    #[test]
    fn test_init_json_obj_empty() {
        initialize();
        let folder_path = "test".to_string();
        let all_paths: Vec<PathBuf> = vec![];
        let json_obj = init_json_obj(folder_path.clone(), all_paths);
        let expected_json_obj = AnnotatedStore {
            image_to_properties_map: HashMap::from([(folder_path, vec![])]),
        };
        assert_eq!(json_obj, expected_json_obj);
    }

    #[test]
    fn test_deserialize_annotated_store_empty() {
        let store = AnnotatedStore {
            image_to_properties_map: HashMap::from([(String::from("test"), vec![])]),
        };
        let expected_string = r###"{"image_to_properties_map":{"test":[]}}"###;
        if let Ok(res_string) = serde_json::to_string(&store) {
            assert_eq!(res_string, expected_string);
        }
    }

    #[test]
    fn test_deserialize_annotated_store_non_empty() {
        let store = AnnotatedStore {
            image_to_properties_map: HashMap::from([(
                String::from("test"),
                vec![Properties {
                    index: 0,
                    image_path: String::from("test/sample.jpg"),
                    annotation: None,
                    comments: None,
                    last_updated: None,
                }],
            )]),
        };
        let expected_string = r###"{"image_to_properties_map":{"test":[{"index":0,"image_path":"test/sample.jpg","annotation":null,"comments":null,"last_updated":null}]}}"###;
        if let Ok(res_string) = serde_json::to_string(&store) {
            assert_eq!(res_string, expected_string);
        }
    }

    #[test]
    fn test_serialize_annotated_store_empty() {
        let raw_string = r###"{"image_to_properties_map":{"test":[]}}"###;
        let serialized_obj: AnnotatedStore = serde_json::from_str(raw_string)
            .unwrap_or_else(|_| panic!("{}", format!("Couldn't serialize {}", raw_string)));
        let store = AnnotatedStore {
            image_to_properties_map: HashMap::from([(String::from("test"), vec![])]),
        };
        assert_eq!(serialized_obj, store);
    }

    #[test]
    fn test_serialize_annotated_store_non_empty() {
        let raw_string = r###"{"image_to_properties_map":{"test":[{"index":0,"image_path":"test/sample.jpg","annotation":null,"comments":null,"last_updated":null}]}}"###;
        let serialized_obj: AnnotatedStore = serde_json::from_str(raw_string)
            .unwrap_or_else(|_| panic!("{}", format!("Couldn't serialize {}", raw_string)));
        let store = AnnotatedStore {
            image_to_properties_map: HashMap::from([(
                String::from("test"),
                vec![Properties {
                    index: 0,
                    image_path: String::from("test/sample.jpg"),
                    annotation: None,
                    comments: None,
                    last_updated: None,
                }],
            )]),
        };
        assert_eq!(serialized_obj, store);
    }
}
