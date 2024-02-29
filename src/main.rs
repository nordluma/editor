use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use iced::{
    executor,
    highlighter::{self, Highlighter},
    theme,
    widget::{
        button, column, container, horizontal_space, pick_list, row, text, text_editor, tooltip,
    },
    Application, Command, Element, Font, Length, Settings, Theme,
};

fn main() -> iced::Result {
    Editor::run(Settings {
        default_font: Font::MONOSPACE,
        fonts: vec![include_bytes!("../fonts/editor-icons.ttf")
            .as_slice()
            .into()],
        ..Settings::default()
    })
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IOFailed(io::ErrorKind),
}

#[derive(Debug, Clone)]
enum Messages {
    New,
    Open,
    Save,
    Edit(text_editor::Action),
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    FileSaved(Result<PathBuf, Error>),
    ThemeSelected(highlighter::Theme),
}

struct Editor {
    theme: highlighter::Theme,
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
                theme: highlighter::Theme::SolarizedDark,
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
            Messages::Open => Command::perform(pick_file(), Messages::FileOpened),
            Messages::New => {
                self.path = None;
                self.content = text_editor::Content::new();

                Command::none()
            }
            Messages::Edit(action) => {
                self.content.edit(action);
                self.error = None;

                Command::none()
            }
            Messages::Save => {
                let text = self.content.text();

                Command::perform(save_file(self.path.clone(), text), Messages::FileSaved)
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
            Messages::FileSaved(Ok(path)) => {
                self.path = Some(path);

                Command::none()
            }
            Messages::FileSaved(Err(err)) => {
                self.error = Some(err);

                Command::none()
            }
            Messages::ThemeSelected(theme) => {
                self.theme = theme;

                Command::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let controls = row![
            action(new_icon(), "Create a new file", Messages::New),
            action(open_icon(), "Open file", Messages::Open),
            action(save_icon(), "Save file", Messages::Save),
            horizontal_space(Length::Fill),
            pick_list(
                highlighter::Theme::ALL,
                Some(self.theme),
                Messages::ThemeSelected
            )
        ]
        .spacing(10);

        let input = text_editor(&self.content)
            .on_edit(Messages::Edit)
            .highlight::<Highlighter>(
                highlighter::Settings {
                    theme: self.theme,
                    extension: self
                        .path
                        .as_ref()
                        .and_then(|path| path.extension()?.to_str())
                        .unwrap_or("rs")
                        .to_string(),
                },
                |highlight, _theme| highlight.to_format(),
            );
        let status_bar = {
            let status = if let Some(Error::IOFailed(err)) = self.error.as_ref() {
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
        if self.theme.is_dark() {
            iced::Theme::Dark
        } else {
            iced::Theme::Light
        }
    }
}

fn action<'a>(
    content: Element<'a, Messages>,
    label: &str,
    on_press: Messages,
) -> Element<'a, Messages> {
    tooltip(
        button(container(content).width(30).center_x())
            .on_press(on_press)
            .padding([5, 10]),
        label,
        tooltip::Position::FollowCursor,
    )
    .style(theme::Container::Box)
    .into()
}

fn new_icon<'a>() -> Element<'a, Messages> {
    icon('\u{E800}')
}

fn open_icon<'a>() -> Element<'a, Messages> {
    icon('\u{E802}')
}

fn save_icon<'a>() -> Element<'a, Messages> {
    icon('\u{E801}')
}

fn icon<'a>(codepoint: char) -> Element<'a, Messages> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint).font(ICON_FONT).into()
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
        .map_err(Error::IOFailed)?;

    Ok((path, content))
}

async fn save_file(path: Option<PathBuf>, text: String) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .set_title("Choose a file name.")
            .save_file()
            .await
            .ok_or(Error::DialogClosed)
            .map(|handle| handle.path().to_path_buf())?
    };

    tokio::fs::write(&path, text)
        .await
        .map_err(|err| Error::IOFailed(err.kind()))?;

    Ok(path)
}
