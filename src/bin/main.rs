use godric::{
    backend::{self, Connection},
    scene::{self, Scene},
};

use color_eyre::Result;
use iced::{futures::SinkExt, Element, Subscription, Task};
use tokio::sync::mpsc;

pub fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv()?;
    Ok(iced::application("Godric", Godric::update, Godric::view)
        .theme(Godric::theme)
        .subscription(Godric::backend_subscription)
        .window(iced::window::Settings {
            icon: iced::window::icon::from_file("Assets/Logo/Icon - zoomed.jpg").ok(),
            ..Default::default()
        })
        .run_with(|| (Godric::default(), Task::none()))?)
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

impl Godric {
    fn update(&mut self, message: Message) -> Task<Message> {
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

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let content = self.scene.view().map(Message::Scene);

        iced::widget::container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }

    fn backend_subscription(&self) -> Subscription<Message> {
        Subscription::run(|| {
            iced::stream::channel(0, |mut ui| async move {
                // Executed only once, even on repeated calls of subscription
                let (sender, mut receiver) = mpsc::channel(50);
                let mut backend = crate::backend::Backend::default();

                ui.send(Ok(crate::backend::Output::Connection(
                    Connection::Connected(sender),
                )))
                .await
                .expect("Unable to connect to GUI!");

                // Executed continuously, kept alive across calls
                loop {
                    let message = receiver
                        .recv()
                        .await
                        .expect("Input connection from GUI closed!");

                    match backend.update(message).await {
                        Ok(message) => {
                            if let Some(message) = message {
                                ui.send(Ok(message)).await;
                            }
                        }
                        Err(error) => {
                            ui.send(Err(error)).await;
                        }
                    }
                }
            })
        })
        .map(Message::Backend)
    }
}
