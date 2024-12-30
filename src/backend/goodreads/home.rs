use crate::{
    backend::{
        self,
        goodreads::{self, State},
    },
    common::book,
};
use thirtyfour as tf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid message ({message}) for state {state}")]
    InvalidState { state: String, message: String },
}

#[derive(Clone, Debug)]
pub enum Input {}

impl From<Input> for goodreads::Input {
    fn from(input: Input) -> Self {
        Self::Home(input)
    }
}

impl TryFrom<goodreads::Input> for Input {
    type Error = Error;

    fn try_from(input: goodreads::Input) -> Result<Self, Self::Error> {
        match input {
            super::Input::Home(input) => Ok(input),
            _ => Err(Self::Error::InvalidState {
                state: "Home".into(),
                message: format!("{:?}", input),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Output {}

#[derive(Clone, Debug)]
pub struct Home {
    user_id: String,
    books: Vec<book::BookID>,
}

impl From<Home> for State {
    fn from(state: Home) -> Self {
        Self::Home(state)
    }
}

impl Home {
    pub async fn new(user_id: String) -> Self {
        let books = fetch_books(&user_id).await;
        Self { user_id, books }
    }

    pub async fn update(
        self,
        _browser: &mut tf::WebDriver,
        input: Input,
    ) -> Result<(State, Option<goodreads::Output>), Error> {
        Ok((self.into(), None))
    }
}

async fn fetch_books(user_id: &str) -> Vec<book::BookID> {
    todo!()
}
