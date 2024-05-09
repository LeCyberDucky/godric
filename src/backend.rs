pub mod home;
pub mod logged_out;

use color_eyre::eyre::ContextCompat;
use iced::futures::SinkExt;
use tokio::sync::mpsc;

use crate::common::browser;

use self::{home::Home, logged_out::LoggedOut};

#[derive(Debug, Clone, Default)]
pub enum Connection {
    #[default]
    Disconnected,
    Connected(mpsc::Sender<Input>),
}

impl Connection {
    pub fn send(&mut self, input: Input) {
        match self {
            Connection::Disconnected => todo!(),
            Connection::Connected(connection) => {
                connection.try_send(input);
            }
        }
    }
}

pub enum Input {
    LoggedOut(logged_out::Input),
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum Error {
    #[error("logged out")]
    LoggedOut(logged_out::Error),
}

#[derive(Debug, Clone)]
pub enum Output {
    Connection(Connection),
    LoggedOut(logged_out::Output),
    Error(Error),
}

#[derive(Clone)]
enum State {
    LoggedOut(LoggedOut),
    Home(Home),
}

impl Default for State {
    fn default() -> Self {
        Self::LoggedOut(logged_out::LoggedOut::default())
    }
}

#[derive(Default)]
pub struct Backend {
    browser: Option<browser::Connection>,
    state: State,
}

impl Backend {
    async fn update(&mut self, input: Input) -> Option<Output> {
        let (state, output) = match (self.state.clone(), input) {
            (State::LoggedOut(state), Input::LoggedOut(input)) => {
                state.update(&mut self.browser, input).await
            }
            (State::Home(_), Input::LoggedOut(_)) => todo!(),
        };

        self.state = state;
        output
    }
}

impl Backend {
    pub fn launch() -> iced::subscription::Subscription<Output> {
        iced::subscription::channel(
            std::any::TypeId::of::<Backend>(),
            0,
            |mut output| async move {
                // Executed only once, even on repeated calls of subscription
                let (sender, mut receiver) = mpsc::channel(50);
                let mut backend = Backend::default();

                output
                    .send(Output::Connection(Connection::Connected(sender)))
                    .await
                    .expect("Unable to connect to GUI!");

                // Executed continuously, kept alive across calls
                loop {
                    let message = receiver
                        .recv()
                        .await
                        .context("Input connection from GUI closed!")
                        .unwrap();
                    // let message = receiver.try_next();
                    if let Some(message) = backend.update(message).await {
                        output.send(message);
                    }
                }
            },
        )
    }
}
