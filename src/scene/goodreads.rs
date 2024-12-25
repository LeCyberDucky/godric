pub mod welcome;

use color_eyre::Result;

#[derive(Clone, Debug)]
pub enum Message {
    Welcome(welcome::Message),
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
    ) -> (crate::scene::State, Option<crate::backend::Input>) {

        let (state, output) = match self {
            State::Welcome(state) => state.update(message.and_then(|message| message.try_into())),
        };

        (state.into(), output.map(|output| output.into()))
    }

    pub fn view(&self) -> iced::Element<Message> {
        match self {
            State::Welcome(state) => state.view().map(Message::Welcome),
        }
    }
}
