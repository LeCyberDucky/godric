use crate::scene::goodreads::State;

use color_eyre::Result;

#[derive(Clone, Debug)]
pub struct Home {
    user_id: String,
}

impl From<Home> for State {
    fn from(state: Home) -> Self {
        Self::Home(state)
    }
}

#[derive(Clone, Debug)]
pub enum Message {}

impl TryFrom<crate::scene::goodreads::Message> for Message {
    type Error = crate::backend::Error;

    fn try_from(message: crate::scene::goodreads::Message) -> Result<Self, Self::Error> {
        match message {
            _ => Err(Self::Error::InvalidState {
                state: "Home".into(),
                message: format!("{:?}", message),
            }),
        }
    }
}

impl Home {
    pub fn new(user_id: String) -> Self {
        Self { user_id }
    }

    pub fn update(
        mut self,
        message: Result<Message, crate::backend::Error>,
    ) -> (State, Option<crate::backend::goodreads::Input>) {
        let mut output: Option<crate::backend::goodreads::home::Input> = None;
        let mut state = None;

        (
            state.unwrap_or(self.into()),
            output.map(|output| output.into()),
        )
    }

    pub fn view(&self) -> iced::Element<Message> {
        todo!()
    }
}
