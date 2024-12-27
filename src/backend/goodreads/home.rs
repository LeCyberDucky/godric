use crate::backend;

#[derive(Clone, Debug)]
pub struct Home {}

#[derive(Clone, Debug)]
pub enum Input {}

impl From<Input> for backend::goodreads::Input {
    fn from(input: Input) -> Self {
        Self::Home(input)
    }
}
