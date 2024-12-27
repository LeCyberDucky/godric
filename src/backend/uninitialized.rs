use color_eyre::Result;

use crate::{
    backend::{self, State},
    common::{browser, helpers::Mode},
};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("failed to connect to browser")]
    BrowserConnection(String),
    #[error("")]
    Other(String),
}

impl From<Error> for backend::Error {
    fn from(error: Error) -> Self {
        Self::Uninitialized(error)
    }
}

#[derive(Clone, Debug)]
pub enum Input {
    Launch {
        browser_driver_config: browser::DriverConfig,
        mode: Mode,
    },
}

impl From<Input> for backend::Input {
    fn from(input: Input) -> Self {
        Self::Uninitialized(input)
    }
}

impl TryFrom<backend::Input> for Input {
    type Error = backend::Error;

    fn try_from(input: backend::Input) -> Result<Self, Self::Error> {
        match input {
            backend::Input::Uninitialized(input) => Ok(input),
            _ => Err(backend::Error::InvalidState {
                state: "Uninitialized".to_string(),
                message: format!("{:?}", input),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Output {
    Initialized(Mode),
}

impl From<Output> for backend::Output {
    fn from(output: Output) -> Self {
        Self::Uninitialized(output)
    }
}

impl From<Uninitialized> for State {
    fn from(state: Uninitialized) -> Self {
        State::Uninitialized(state)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Uninitialized {}

impl Uninitialized {
    pub async fn update(
        self,
        connection: &mut Option<browser::Connection>,
        input: Input,
    ) -> Result<(State, Option<Output>), Error> {
        match input {
            Input::Launch {
                browser_driver_config,
                mode,
            } => {
                if connection.is_none() {
                    match browser::Connection::new(&browser_driver_config).await {
                        Ok(new_connection) => *connection = Some(new_connection),
                        Err(error) => return Err(Error::BrowserConnection(error.to_string())),
                    }
                }

                match mode {
                    Mode::Goodreads => Ok((
                        State::Goodreads(backend::goodreads::welcome::Welcome::default().into()),
                        Some(Output::Initialized(mode.into()).into()),
                    )),
                }
            }
        }
    }
}
