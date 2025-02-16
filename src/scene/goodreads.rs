pub mod book;
pub mod home;
pub mod welcome;

use color_eyre::Result;
use iced::Task;

#[derive(Clone, Debug)]
pub enum Message {
    Welcome(welcome::Message),
    Home(home::Message),
}

impl From<Message> for crate::scene::Message {
    fn from(message: Message) -> Self {
        Self::Goodreads(message)
    }
}

impl From<crate::backend::goodreads::Output> for Message {
    fn from(output: crate::backend::goodreads::Output) -> Self {
        match output {
            crate::backend::goodreads::Output::Welcome(output) => Self::Welcome(output.into()),
        }
    }
}

impl TryFrom<crate::scene::Message> for Message {
    type Error = crate::backend::Error;

    fn try_from(message: crate::scene::Message) -> Result<Self, Self::Error> {
        match message {
            crate::scene::Message::Goodreads(message) => Ok(message),
            _ => Err(crate::backend::Error::UnhandledMessage(format!(
                "{message:?}"
            ))),
        }
    }
}

#[derive(Clone, Debug)]
pub enum State {
    Welcome(welcome::Welcome),
    Home(home::Home),
}

impl From<State> for crate::scene::State {
    fn from(state: State) -> Self {
        Self::Goodreads(state)
    }
}

impl State {
    pub fn update(
        mut self,
        message: Result<Message, crate::backend::Error>,
    ) -> (
        crate::scene::State,
        Option<crate::backend::Input>,
        Task<crate::scene::Message>,
    ) {
        let (state, output, task) = match self {
            State::Welcome(state) => state.update(message.and_then(|message| message.try_into())),
            State::Home(state) => state.update(message.and_then(|message| message.try_into())),
        };

        (
            state.into(),
            output.map(|output| output.into()),
            task.map(|message| message.into()),
        )
    }

    pub fn view(&self) -> iced::Element<Message> {
        match self {
            State::Welcome(state) => state.view().map(Message::Welcome),
            State::Home(state) => state.view().map(Message::Home),
        }
    }
}
