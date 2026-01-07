pub mod goodreads;
pub mod uninitialized;

use color_eyre::{Result, eyre::ContextCompat};
use tokio::sync::mpsc;

use self::uninitialized::Uninitialized;
use crate::common::{browser, cache};

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
    pub const CAPACITY: usize = 50;

    pub fn send(&mut self, input: Input) -> Result<(), Error> {
        match self {
            Connection::Disconnected => todo!(),
            Connection::Connected(connection) => {
                if let Input::Tick = input
                    && connection.capacity() < connection.max_capacity()
                {
                    // There are other messages queued up already, so we don't need to send a tick
                    return Ok(());
                }

                connection
                    .try_send(input)
                    .map_err(|error| Error::UiDisconnected(error.to_string()))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Input {
    Uninitialized(uninitialized::Input),
    Goodreads(goodreads::Input),
    Tick,
}

#[derive(Debug, Clone)]
pub enum Output {
    Connection(Connection),
    Goodreads(goodreads::Output),
    Uninitialized(uninitialized::Output),
}

#[derive(Debug)]
pub enum State {
    Uninitialized(Uninitialized),
    Goodreads {
        cache: std::sync::Arc<
            tokio::sync::RwLock<
                crate::common::cache::Cache<url::Url, crate::backend::goodreads::book::Book>,
            >,
        >,
        state: goodreads::State,
    },
    Error,
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
    pub async fn update(mut self, input: Input) -> (Self, Result<Option<Output>, Error>) {
        let state_description = format!("{:?}", self.state);
        let input_description = format!("{input:?}");
        let outcome: Result<(State, Option<Output>), Error> = match self.state {
            State::Uninitialized(state) if let Ok(input) = input.clone().try_into() => state
                .update(&mut self.browser_connection, input)
                .await
                .map_err(|error| error.into()),

            State::Goodreads { cache, state } if let Ok(input) = input.clone().try_into() => {
                let connection = self
                    .browser_connection
                    .as_mut()
                    .context("Browser disconnected!");

                match connection {
                    Ok(connection) => state
                        .update(&mut connection.browser, cache.clone(), input)
                        .await
                        .map_err(|error| error.into()),
                    Err(error) => {
                        Err(uninitialized::Error::BrowserConnection(error.to_string()).into())
                    }
                }
            }
            _ => Err(Error::InvalidState {
                state: state_description,
                message: input_description,
            }),
        };

        // If a state update fails, we cannot just pretend that nothing happened and return to the initial state
        // Rust enforces this, because the update functions consume the state, so there is nothing to return to. Therefore, we enter an error state instead
        // Now, the question is: Should all problems in the update functions lead directly to the error state? Probably not. Perhaps the update functions should return
        // Result<(State, Result<Option<backend::Output>, Error>), Error>
        // instead of
        // Result<(backend::State, Option<backend::Output>), Error>
        if let Ok((state, output)) = outcome {
            self.state = state;
            (self, Ok(output))
        } else {
            self.state = State::Error;
            (self, outcome.map(|(state, output)| output))
        }
    }
}
