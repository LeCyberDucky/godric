use crate::{
    backend::{self, goodreads::State},
    common::helpers::Credentials,
};
use color_eyre::Result;
use thirtyfour as tf;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Invalid message ({message}) for state {state}")]
    InvalidState { state: String, message: String },
}

#[derive(Clone, Debug)]
pub enum Input {
    LoginAttempt { credentials: Credentials },
}

impl From<Input> for backend::goodreads::Input {
    fn from(input: Input) -> Self {
        Self::Welcome(input)
    }
}

impl TryFrom<backend::goodreads::Input> for Input {
    type Error = Error;

    fn try_from(input: backend::goodreads::Input) -> Result<Self, Self::Error> {
        match input {
            super::Input::Welcome(input) => Ok(input),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Output {}

impl From<Output> for backend::goodreads::Output {
    fn from(output: Output) -> Self {
        Self::Welcome(output)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Welcome {}

impl From<Welcome> for State {
    fn from(state: Welcome) -> Self {
        Self::Welcome(state)
    }
}

impl Welcome {
    pub async fn update(
        self,
        browser: &mut tf::WebDriver,
        input: Input,
    ) -> Result<(State, Option<Output>), Error> {
        todo!()
    }
}
