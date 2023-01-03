// use iced::Length;

#[derive(Debug, PartialEq, Clone, Eq, Copy)]
pub enum ThemeType {
    Light,
    Dark,
}

#[derive(Debug, Clone)]
pub enum Message {
    ThemeChanged(ThemeType),
    Previous(),
    Next(),
    MarkAsCorrect(),
    MarkAsIncorrect(),
    Export(),
}

// pub fn get_image_handle<'a>(img_path: String) -> Container<'a, Message> {
//     container(image()).width(Length::Fill).center_x()
// }
