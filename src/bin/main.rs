use godric::backend;
use iced::executor;
use iced::{Application, Command, Element, Settings, Theme};

use godric::scene::{self, Scene};

pub fn main() -> iced::Result {
    Godric::run(Settings::default())
}

#[derive(Debug, Clone)]
enum Message {
    Backend(backend::Output),
    Scene(scene::Message),
}

struct Godric {
    backend: backend::Endpoint,
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
        match message {
            Message::Backend(output) => match output {
                backend::Output::Connection(connection) => self.backend.connection = connection,
            },
            Message::Scene(message) => {
                let backend_message = self.scene.update(message);
                if let Some(message) = backend_message {
                    self.backend.connection.send(message);
                }
            }
        };

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let content = self.scene.view().map(Message::Scene);

        iced::widget::container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}