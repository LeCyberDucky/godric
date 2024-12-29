use godric::{
    backend::{self, Connection},
    scene::{self, Scene},
};

use color_eyre::Result;
use iced::{Element, Subscription, Task};

pub fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv()?;
    let godric = Godric::default();
    Ok(iced::application("Godric", update, view)
        .theme(theme)
        .subscription(backend_subscription)
        .window(iced::window::Settings {
            icon: iced::window::icon::from_file("Assets/Logo/Icon - zoomed.jpg").ok(),
            ..Default::default()
        })
        .run()?)
}

#[derive(Debug)]
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

fn update(state: &mut Godric, message: Message) -> Task<Message> {
    // Special treatment for establishing initial backend connection
    if let Message::Backend(ref message) = message {
        if let Ok(message) = message {
            if let backend::Output::Connection(connection) = message {
                state.backend = connection.clone();
            }
        }
    }

    let message = match message {
        Message::Scene(message) => Ok(message),
        Message::Backend(output) => output.map(|output| output.into()),
    };

    if let Some(input) = state.scene.update(message) {
        state.backend.send(input);
    }

    Task::none()
}

fn view(state: &Godric) -> Element<Message> {
    let content = state.scene.view().map(Message::Scene);

    iced::widget::container(content)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}

fn backend_subscription(state: &Godric) -> Subscription<Message> {
    Subscription::run(backend::Backend::launch).map(Message::Backend)
}

fn theme(state: &Godric) -> iced::Theme {
    state.theme.clone()
}
