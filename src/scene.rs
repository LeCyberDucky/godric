pub mod home;
pub mod login;

use iced::executor;
use iced::widget::pane_grid::state;
use iced::{application::Application, Command, Element, Settings, Theme};

use crate::backend;

#[derive(Clone)]
pub enum State {
    Login(login::Login),
    Home(home::Home),
}

impl Default for State {
    fn default() -> Self {
        Self::Login(login::Login::default())
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Login(login::Message),
    Home(home::Message),
}

#[derive(Default)]
pub struct Scene {
    state: State,
}

impl Scene {
    pub fn update(&mut self, message: Message) -> Option<backend::Input> {
        let (state, backend_input) = match (self.state.clone(), message) {
            (State::Login(state), Message::Login(message)) => state.update(message),
            (State::Home(state), Message::Home(message)) => state.update(message),
            _ => todo!(),
        };

        self.state = state;
        backend_input
    }

    pub fn view(&self) -> Element<Message> {
        match &self.state {
            State::Login(state) => state.view().map(Message::Login),
            State::Home(state) => state.view().map(Message::Home),
        }
    }
}
