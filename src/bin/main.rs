use godric::backend::{self, Connection};
use iced::executor;
use iced::{Application, Command, Element, Settings, Theme};

use godric::common::gui::{self, Message};
use godric::scene::{self, Scene};

pub fn main() -> iced::Result {
    Godric::run(Settings::default())
}

struct Godric {
    backend: backend::Endpoint,
    login_scene: godric::scene::login::State,
}

impl Application for Godric {
    type Executor = executor::Default;
    type Message = gui::Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Godric {
                backend: backend::Endpoint::default(),
                login_scene: godric::scene::login::State::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Godric")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Backend(output) => match output {
                backend::Output::Connection(connection) => 
                    self.backend.connection = connection,
            },
            Message::LoginScene(message) => return self.login_scene.update(message),
            Message::Home(_) => todo!(),
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        self.login_scene.view().into()
    }
}
