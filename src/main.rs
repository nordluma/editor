use iced::{widget::text, Sandbox, Settings};

fn main() -> iced::Result {
    Editor::run(Settings::default())
}

#[derive(Debug)]
enum Messages {}

struct Editor;

impl Sandbox for Editor {
    type Message = Messages;

    fn new() -> Self {
        Self
    }

    fn title(&self) -> String {
        String::from("An iced out editor")
    }

    fn update(&mut self, message: Self::Message) {
        match message {}
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        text("Hello, iced").into()
    }
}
