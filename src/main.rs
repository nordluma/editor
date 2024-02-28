use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use iced::{
    executor,
    widget::{button, column, container, horizontal_space, row, text, text_editor},
    Application, Command, Length, Settings, Theme,
};

fn main() -> iced::Result {
    Editor::run(Settings::default())
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IO(io::ErrorKind),
}

#[derive(Debug, Clone)]
enum Messages {
    Edit(text_editor::Action),
    New,
    Open,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
}

struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
}

impl Application for Editor {
    type Message = Messages;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Messages>) {
        (
            Self {
                path: None,
                content: text_editor::Content::new(),
                error: None,
            },
            Command::perform(load_file(default_file()), Messages::FileOpened),
        )
    }

    fn title(&self) -> String {
        String::from("An iced out editor")
    }

    fn update(&mut self, message: Self::Message) -> Command<Messages> {
        match message {
            Messages::New => {
                self.path = None;
                self.content = text_editor::Content::new();

                Command::none()
            }
            Messages::Open => Command::perform(pick_file(), Messages::FileOpened),
            Messages::Edit(action) => {
                self.content.edit(action);

                Command::none()
            }
            Messages::FileOpened(Ok((path, content))) => {
                self.path = Some(path);
                self.content = text_editor::Content::with(&content);

                Command::none()
            }
            Messages::FileOpened(Err(err)) => {
                self.error = Some(err);

                Command::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let controls = row![
            button("New").on_press(Messages::New),
            button("Open").on_press(Messages::Open)
        ];

        let input = text_editor(&self.content).on_edit(Messages::Edit);
        let status_bar = {
            let status = if let Some(Error::IO(err)) = self.error.as_ref() {
                text(err.to_string())
            } else {
                match self.path.as_deref().and_then(Path::to_str) {
                    Some(path) => text(path).size(14),
                    None => text("New file"),
                }
            };

            let position = {
                let (line, column) = self.content.cursor_position();
                text(format!("{}:{}", line + 1, column + 1))
            };

            row![status, horizontal_space(Length::Fill), position]
        };

        container(column![controls, input, status_bar].spacing(10))
            .padding(10)
            .into()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}

fn default_file() -> PathBuf {
    format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")).into()
}

async fn pick_file() -> Result<(PathBuf, Arc<String>), Error> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Choose a text file")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;

    load_file(handle.path().to_path_buf()).await
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let content = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|err| err.kind())
        .map_err(Error::IO)?;

    Ok((path, content))
}
