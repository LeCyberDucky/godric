use color_eyre::Result;
use godric::backend::{self, Connection};
use iced::{executor, Application, Command, Element, Theme};

use godric::scene::{self, Scene};

pub fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv()?;
    let mut settings = iced::Settings::default();
    settings.window.icon = iced::window::icon::from_file("Assets/Logo/Icon - zoomed.jpg").ok();
    Ok(Godric::run(settings)?)
}

#[derive(Debug, Clone)]
enum Message {
    Backend(Result<backend::Output, backend::Error>),
    Scene(scene::Message),
}

struct Godric {
    backend: Connection,
    scene: Scene,
    theme: iced::Theme,
}

impl Default for Godric {
    fn default() -> Self {
        Self {
            backend: Default::default(),
            scene: Default::default(),
            theme: iced::Theme::SolarizedLight,
        }
    }
}

impl Application for Godric {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Godric::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Godric")
    }

    fn theme(&self) -> Self::Theme {
        self.theme.clone()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        dbg!(message.clone());

        // Special treatment for establishing initial backend connection
        if let Message::Backend(ref message) = message {
            if let Ok(message) = message {
                if let backend::Output::Connection(connection) = message {
                    self.backend = connection.clone();
                }
            }
        }

        let message = match message {
            Message::Scene(message) => Ok(message),
            Message::Backend(output) => output.map(|output| output.into()),
        };

        if let Some(input) = self.scene.update(message) {
            self.backend.send(input);
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let content = self.scene.view().map(Message::Scene);

        iced::widget::container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        backend::Backend::launch().map(Message::Backend)
    }
}
