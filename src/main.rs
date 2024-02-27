use iced::{
    widget::{container, text_editor},
    Sandbox, Settings,
};

fn main() -> iced::Result {
    Editor::run(Settings::default())
}

#[derive(Debug, Clone)]
enum Messages {
    Edit(text_editor::Action),
}

struct Editor {
    content: text_editor::Content,
}

impl Sandbox for Editor {
    type Message = Messages;

    fn new() -> Self {
        Self {
            content: text_editor::Content::new(),
        }
    }

    fn title(&self) -> String {
        String::from("An iced out editor")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Messages::Edit(action) => {
                self.content.edit(action);
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let input = text_editor(&self.content).on_edit(Messages::Edit);

        container(input).padding(10).into()
    }
}
