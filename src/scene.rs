pub mod goodreads;
pub mod launch;

use crate::backend;
use color_eyre::Result;
use iced::Element;

type Error = backend::Error;

#[derive(Clone)]
pub enum State {
    Launch(launch::Launch),
    Goodreads(goodreads::State),
}

impl Default for State {
    fn default() -> Self {
        Self::Launch(launch::Launch::default())
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Connected,
    Launch(launch::Message),
    Goodreads(goodreads::Message),
}

impl From<crate::backend::Output> for Message {
    fn from(output: crate::backend::Output) -> Self {
        match output {
            backend::Output::Connection(connection) => Self::Connected,
            backend::Output::Goodreads(output) => Self::Goodreads(output.into()),
            backend::Output::Uninitialized(output) => Self::Launch(output.into()),
        }
    }
}

#[derive(Default)]
pub struct Scene {
    state: State,
}

impl Scene {
    pub fn update(&mut self, message: Result<Message, Error>) -> Option<backend::Input> {
        let (state, output) = match self.state.clone() {
            State::Launch(state) => {
                state.update(message.and_then(|message| launch::Message::try_from(message)))
            }
            State::Goodreads(state) => {
                state.update(message.and_then(|message| goodreads::Message::try_from(message)))
            }
        };

        self.state = state;
        output
    }

    pub fn view(&self) -> Element<Message> {
        match &self.state {
            State::Launch(state) => state.view().map(Message::Launch),
            State::Goodreads(state) => state.view().map(Message::Goodreads),
        }
    }
}
