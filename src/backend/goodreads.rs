pub mod home;
pub mod welcome;

use crate::backend;
use color_eyre::Result;
use thirtyfour as tf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Welcome(#[from] welcome::Error),
    #[error("Invalid message ({message}) for state {state}")]
    InvalidState { state: String, message: String },
}

#[derive(Clone, Debug)]
pub enum Input {
    Welcome(welcome::Input),
    Home(home::Input),
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
            _ => Err(Self::Error::InvalidState {
                state: "Goodreads".into(),
                message: format!("{:?}", input),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Output {
    Welcome(welcome::Output),
}

impl From<Output> for backend::Output {
    fn from(output: Output) -> Self {
        Self::Goodreads(output)
    }
}

#[derive(Clone, Debug)]
pub enum State {
    Welcome(welcome::Welcome),
}

impl Default for State {
    fn default() -> Self {
        Self::Welcome(welcome::Welcome::default())
    }
}

impl From<State> for crate::backend::State {
    fn from(state: State) -> Self {
        Self::Goodreads(state)
    }
}

impl State {
    pub async fn update(
        self,
        browser: &mut tf::WebDriver,
        input: Input,
    ) -> Result<(backend::State, Option<Output>), Error> {
        let (state, output) = match self {
            State::Welcome(state) => state.update(browser, input.try_into()?).await?,
        };

        Ok((state.into(), output.map(|output| output.into())))
    }
}
