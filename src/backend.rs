pub mod goodreads;
pub mod uninitialized;

use color_eyre::{eyre::ContextCompat, Result};
use iced::futures::SinkExt;
use tokio::sync::mpsc;

use self::uninitialized::Uninitialized;
use crate::common::browser;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Goodreads(#[from] goodreads::Error),
    #[error("Invalid message ({message}) for state {state}")]
    InvalidState { state: String, message: String },
    #[error("Unhandled message: {0}")]
    UnhandledMessage(String),
    #[error("Not initialized")]
    Uninitialized(uninitialized::Error),
    #[error("Backend unable to reach UI")]
    UiDisconnected(String),
}

#[derive(Debug, Clone, Default)]
pub enum Connection {
    #[default]
    Disconnected,
    Connected(mpsc::Sender<Input>),
}

impl Connection {
    pub fn send(&mut self, input: Input) -> Result<(), Error> {
        match self {
            Connection::Disconnected => todo!(),
            Connection::Connected(connection) => connection
                .try_send(input)
                .map_err(|error| Error::UiDisconnected(error.to_string())),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Input {
    Uninitialized(uninitialized::Input),
    Goodreads(goodreads::Input),
}

#[derive(Debug, Clone)]
pub enum Output {
    Connection(Connection),
    Goodreads(goodreads::Output),
    Uninitialized(uninitialized::Output),
}

#[derive(Clone, Debug)]
enum State {
    Uninitialized(Uninitialized),
    Goodreads(goodreads::State),
}

impl Default for State {
    fn default() -> Self {
        Self::Uninitialized(uninitialized::Uninitialized::default())
    }
}

#[derive(Debug, Default)]
pub struct Backend {
    browser_connection: Option<browser::Connection>,
    state: State,
}

impl Backend {
    async fn update(&mut self, input: Input) -> Result<Option<Output>, Error> {
        dbg!(self.state.clone());
        dbg!(self.browser_connection.is_some());
        dbg!(input.clone());

        if let State::Uninitialized(state) = self.state.clone()
            && let Input::Uninitialized(input) = input
        {
            let (state, output) = state.update(&mut self.browser_connection, input).await?;
            self.state = state;
            return Ok(output.map(|output| output.into()));
        };

        let connection = self
            .browser_connection
            .as_mut()
            .context("Browser disconnected!")
            .map_err(|error| uninitialized::Error::BrowserConnection(error.to_string()))?;

        let (state, output) = match self.state.clone() {
            State::Goodreads(state) => {
                let (state, output) = state
                    .update(&mut connection.browser, input.try_into()?)
                    .await?;
                (state, Ok(output.map(|output| output.into())))
            }
            _ => (
                self.state.clone(),
                Err(Error::InvalidState {
                    state: format!("{:?}", self.state),
                    message: format!("{input:?}"),
                }),
            ),
        };

        self.state = state;
        output
    }

    pub fn launch() -> impl iced::futures::Stream<Item = Result<Output, Error>> {
        iced::stream::channel(0, |mut ui| async move {
            // Executed only once, even on repeated calls of subscription
            let (sender, mut receiver) = mpsc::channel(50);
            let mut backend = Backend::default();

            ui.send(Ok(Output::Connection(Connection::Connected(sender))))
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
    }
}
