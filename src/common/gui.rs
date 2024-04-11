use crate::backend;

#[derive(Debug, Clone)]
pub enum Message {
    // Backend(backend::Output)
    Backend(backend::Output),
    LoginScene(crate::scene::login::Message),
    Home(crate::scene::home::Message),
}

pub enum State {
    Login,
    Main,
}
