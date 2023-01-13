use std::{collections::HashMap, path::PathBuf};

use iced::{
    theme,
    widget::{button, column, container, horizontal_space, row, scrollable},
    Element, Length, Renderer, Sandbox,
};

use self::render_image::{init_json_obj, AnnotatedStore, Message, Step, StepMessage};

#[path = "render_image.rs"]
mod render_image;

#[derive(Default, Debug, Clone)]
pub struct Steps {
    steps: Vec<render_image::Step>,
    folder_path: String,
    curr_idx: usize,
    all_images: Vec<PathBuf>,
    correct_items: Vec<Option<bool>>,
    json_obj: AnnotatedStore,
    current: usize,
    modified: bool,
    btn_status: bool,
}

#[derive(Default)]
pub struct FolderVisualizer {
    steps: Steps,
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

impl Sandbox for FolderVisualizer {
    type Message = render_image::Message;

    fn new() -> FolderVisualizer {
        let folder_path: String = "".into();
        let all_images = vec![];
        let json_obj: AnnotatedStore = init_json_obj(folder_path.clone(), all_images.clone());
        let mut steps_obj = Steps::new(folder_path, 0, all_images.clone(), vec![], json_obj);
        steps_obj.correct_items = vec![None; all_images.len()];
        // steps_obj.buttons = HashMap::new();
        FolderVisualizer { steps: steps_obj }
    }

    fn title(&self) -> String {
        self.steps.title()
    }

    fn view(self: &FolderVisualizer) -> iced::Element<'_, Self::Message, Renderer> {
        let FolderVisualizer { steps, .. } = self;
        let mut controls = row![];

        if steps.has_previous() {
            controls = controls.push(
                button("Back")
                    .on_press(Message::BackPressed)
                    .style(theme::Button::Secondary),
            );
        }

        controls = controls.push(horizontal_space(Length::Fill));

        // let (new_btn_status, element_view) = steps.view();
        let element_view = steps.view();
        // println!("new status: {}", new_btn_status);
        if steps.can_continue() {
            unsafe {
                if render_image::FOLDER_FOUND {
                    controls = controls.push(
                        button("Next")
                            .on_press(Message::NextPressed)
                            .style(theme::Button::Primary),
                    );
                } else {
                    controls = controls.push(button("Next"));
                }
            }
        }

        let content: Element<_> = column![container(
            column![element_view.map(Message::StepMessage), controls,]
                .spacing(20)
                .padding(20)
                .align_items(iced::Alignment::Fill),
        )]
        .into();

        let scrollable = scrollable(container(content).width(Length::Fill).center_x());
        container(scrollable).height(Length::Fill).center_y().into()
        // scrollable(container(content)).into()
        // container(content).into()
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::BackPressed => {
                self.steps.go_back();
            }
            Message::NextPressed => {
                self.steps.advance();
            }
            Message::StepMessage(step_msg) => {
                self.steps.update(step_msg);
            }
        }
    }
}

impl Steps {
    pub fn new(
        folder_path: String,
        curr_idx: usize,
        all_images: Vec<PathBuf>,
        correct_items: Vec<Option<bool>>,
        json_obj: AnnotatedStore,
    ) -> Steps {
        Steps {
            steps: vec![Step::WelcomeWithFolderChoose, Step::Images, Step::End],
            folder_path,
            curr_idx,
            all_images,
            correct_items,
            json_obj,
            current: 0,
            modified: false,
            btn_status: false,
        }
    }

    pub fn update(&mut self, msg: StepMessage) {
        let (new_idx, new_image_prop_map, new_annotation, new_correct_items, new_steps_obj) =
            self.steps[self.current].update(
                msg,
                &mut self.curr_idx,
                self.folder_path.clone(),
                &mut self.json_obj.image_to_properties_map,
                &mut self.correct_items,
            );
        if new_steps_obj.modified {
            self.curr_idx = new_steps_obj.curr_idx;
            self.correct_items = new_steps_obj.correct_items;
            self.folder_path = new_steps_obj.folder_path;
            self.json_obj.image_to_properties_map = new_image_prop_map;
            self.all_images = new_steps_obj.all_images;
        } else {
            self.curr_idx = new_idx;
            // self.json_obj.image_to_properties_map.get_mut(&self.folder_path).unwrap().get_mut(self.curr_idx).unwrap().annotation = new_annotation;
            // self.json_obj.image_to_properties_map = new_image_prop_map;
            self.correct_items = new_correct_items;
            self.json_obj
                .image_to_properties_map
                .get_mut(&self.folder_path)
                .unwrap()
                .get_mut(self.curr_idx)
                .unwrap()
                .annotation = new_annotation;
        }
        // let prop_for_folder = self
        //     .json_obj
        //     .image_to_properties_map
        //     .get_mut(&self.folder_path);
        println!("json_obj here: {:?}", self.json_obj);

        // if let Some(prop) = prop_for_folder {
        //     let property = prop.get_mut(self.curr_idx);
        //     if let Some(valid_prop) = property {
        //         valid_prop.annotation = new_annotation;
        //     }
        // }
        // println!(
        //     "New ann: {:?}, new map: {:?}",
        //     new_annotation, new_image_prop_map.clone()
        // );
        // self.json_obj.image_to_properties_map = new_image_prop_map;
    }

    pub fn view(&self) -> Element<StepMessage> {
        self.steps[self.current].view(self)
    }

    pub fn advance(&mut self) {
        if self.can_continue() {
            self.current += 1;
        }
    }

    pub fn go_back(&mut self) {
        if self.has_previous() {
            self.current -= 1;
            if self.current == 0 {
                unsafe {
                    render_image::FOLDER_FOUND = false;
                }
            }
        }
    }

    pub fn has_previous(&self) -> bool {
        self.current > 0
    }

    pub fn enable_next_button(&mut self) {
        self.btn_status = true;
    }

    pub fn disable_next_button(&mut self) {
        self.btn_status = false;
    }

    pub fn can_continue(&self) -> bool {
        self.current + 1 < self.steps.len() && self.steps[self.current].can_continue()
    }

    pub fn is_next_image_available(&self) -> bool {
        self.curr_idx + 1 < self.all_images.len()
    }

    pub fn is_previous_image_available(&self) -> bool {
        self.curr_idx != 0
    }

    pub fn title(&self) -> String {
        self.steps[self.current].title()
    }
}
