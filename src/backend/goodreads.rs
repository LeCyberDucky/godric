pub mod book;
pub mod home;
pub mod welcome;

use crate::backend;
use color_eyre::Result;
use thirtyfour as tf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Welcome(#[from] welcome::Error),
    #[error(transparent)]
    Home(#[from] home::Error),
    #[error("Invalid message ({message}) for state {state}")]
    InvalidState { state: String, message: String },
}

#[derive(Clone, Debug)]
pub enum Input {
    Welcome(welcome::Input),
    Home(home::Input),
    Tick,
}

impl From<Input> for backend::Input {
    fn from(input: Input) -> Self {
        Self::Goodreads(input)
    }
}

impl TryFrom<backend::Input> for Input {
    type Error = backend::Error;

    fn try_from(input: backend::Input) -> Result<Self, Self::Error> {
        match input {
            backend::Input::Goodreads(input) => Ok(input),
            backend::Input::Tick => Ok(Self::Tick),
            _ => Err(Self::Error::InvalidState {
                state: "Goodreads".into(),
                message: format!("{input:?}"),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Output {
    Welcome(welcome::Output),
    Home(home::Output),
}

impl From<Output> for backend::Output {
    fn from(output: Output) -> Self {
        Self::Goodreads(output)
    }
}

#[derive(Debug)]
pub enum State {
    Welcome(welcome::Welcome),
    Home(home::Home),
}

impl Default for State {
    fn default() -> Self {
        Self::Welcome(welcome::Welcome::default())
    }
}

impl State {
    pub async fn update(
        self,
        browser: &mut tf::WebDriver,
        cache: std::sync::Arc<
            tokio::sync::RwLock<
                crate::common::cache::Cache<url::Url, crate::backend::goodreads::book::Book>,
            >,
        >,
        input: Input,
    ) -> Result<(backend::State, Option<backend::Output>), Error> {
        let (state, output) = match self {
            State::Welcome(state) => {
                state
                    .update(browser, cache.clone(), input.try_into()?)
                    .await?
            }
            State::Home(state) => state.update(browser, input.try_into()?).await?,
        };

        let state = backend::State::Goodreads { cache, state };
        Ok((state, output.map(|output| output.into())))
    }
}
