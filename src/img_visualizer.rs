use std::path::PathBuf;

use iced::{
    widget::{button, column, row},
    widget::{container, image, text, Button}, Theme, Sandbox, Renderer,
};

use self::render_image::Message;
use serde::{Deserialize, Serialize};
use serde_json::json;
use notify_rust::Notification;

#[path = "render_image.rs"]
mod render_image;

#[derive(Default, Debug)]
pub struct FolderVisualizer {
    theme: Theme,
    folder_path: String,
    curr_idx: usize,
    all_images: Vec<PathBuf>,
    correct_items: Vec<bool>,
    json_obj: AnnotatedStore,
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

fn get_all_images(folder_path: &String) -> Vec<PathBuf> {
    // TODO: Handle dir validation here
    let paths = std::fs::read_dir(folder_path).unwrap();
    let mut output: Vec<PathBuf> = vec![];
    for path in paths {
        output.push(path.unwrap().path().as_path().to_owned());
    }
    output
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
struct AnnotatedStore {
    indices: Vec<i32>,
    values: Vec<bool>,
}

fn init_json_obj(total_len: usize) -> AnnotatedStore {
    let init_vec = vec![0; total_len];
    let bool_vec = vec![false; total_len];
    // init_vec.iter().enumerate().map(|(idx, elem)| hash_map.insert(idx, elem));
    let json_obj = json!({"indices": init_vec, "values": bool_vec});
    println!("json_obj: {:?}", json_obj);
    let obj: AnnotatedStore = serde_json::from_value(json_obj).unwrap();
    obj
}

impl Sandbox for FolderVisualizer {
    type Message = render_image::Message;

    fn new() -> FolderVisualizer {
        let folder_path: String = "sample_folder".into();
        let all_images = get_all_images(&folder_path);
        let json_obj: AnnotatedStore = init_json_obj(all_images.len());
        let mut folder_obj = FolderVisualizer {
            theme: Theme::Dark,
            folder_path,
            curr_idx: 0,
            all_images,
            correct_items: vec![],
            json_obj,
        };
        folder_obj.correct_items = vec![false; folder_obj.all_images.len()];
        folder_obj
    }

    fn title(&self) -> String {
        format!("Image {0}", self.curr_idx)
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Renderer> {
        let export_btn = button(text("Export").size(40)).on_press(Message::Export());
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
                update_json(&mut self.json_obj, self.curr_idx as i32, true);
            }
            render_image::Message::MarkAsIncorrect() => {
                self.correct_items[self.curr_idx] = false;
                update_json(&mut self.json_obj, self.curr_idx as i32, false);
            }
            render_image::Message::Export() => {
                write_json(&self.json_obj);
                let _ = Notification::new().summary("Exported to output.json").body("See this is the detailed body").show();
            }
        }
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
