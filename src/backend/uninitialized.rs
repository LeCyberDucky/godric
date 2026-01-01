use color_eyre::Result;

use crate::{
    backend::{self, State},
    common::{browser, cache, helpers::Mode},
};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("failed to connect to browser")]
    BrowserConnection(String),
    #[error("")]
    Other(String),
        #[error("{0}")]
    Cache(#[from] cache::Error),
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
        cache_config: cache::Config,
    },
    Tick,
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
            backend::Input::Tick => Ok(Self::Tick),
            _ => Err(backend::Error::InvalidState {
                state: "Uninitialized".to_string(),
                message: format!("{input:?}"),
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

impl Uninitialized
{
    pub async fn update(
        self,
        connection: &mut Option<browser::Connection>,
        input: Input,
    ) -> Result<(State, Option<backend::Output>), Error> {
        match input {
            Input::Launch {
                browser_driver_config,
                mode,
                cache_config,
            } => {
                if connection.is_none() {
                    match browser::Connection::new(&browser_driver_config).await {
                        Ok(new_connection) => *connection = Some(new_connection),
        Err(error) => return Err(Error::BrowserConnection(error.to_string())),
                    }
                }

                match mode {
                    Mode::Goodreads => {
                        let cache = std::sync::Arc::new(std::sync::RwLock::new(cache_config.try_into()?));
                        Ok((
                        State::Goodreads{cache , state: backend::goodreads::welcome::Welcome::default().into()},
                        Some(Output::Initialized(mode).into()),
                    ))},
                    Mode::Steam => todo!(),
                }
            }
            Input::Tick => Ok((self.into(), None)),
        }
    }
}
