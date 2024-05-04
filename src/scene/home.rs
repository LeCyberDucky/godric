use crate::{backend, scene::State};

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Clone)]
pub struct Home {}

impl From<Home> for State {
    fn from(state: Home) -> Self {
        State::Home(state)
    }
}

impl Home {
    pub fn update(mut self, message: Message) -> (State, Option<backend::Input>) {
        match message {}
        (self.into(), None)
    }

    pub fn view(&self) -> iced::Element<Message> {
        "Home!".into()
    }
}
