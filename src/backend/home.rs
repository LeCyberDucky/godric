use crate::backend::State;

#[derive(Clone, Debug)]
pub struct Home {}

impl From<Home> for State {
    fn from(state: Home) -> Self {
        Self::Home(state)
    }
}

impl Home {
    pub fn new(user: String) -> Self {
        Self {}
    }
}
