pub mod initial;

use futures::channel::mpsc;
use iced::futures::{self, SinkExt};

use crate::common::helpers::Credentials;

#[derive(Debug, Clone, Default)]
pub enum Connection {
    #[default]
    Disconnected,
    Connected(mpsc::Sender<Input>),
}

impl Connection {
    pub fn send(&mut self, input: Input) {
        match self {
            Connection::Disconnected => todo!(),
            Connection::Connected(connection) => {
                connection.try_send(input);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Output {
    Connection(Connection),
}

pub enum Input {}

#[derive(Default)]
pub struct Endpoint {
    pub connection: Connection,
}

pub struct Backend<S> {
    receiver: mpsc::Receiver<Input>,
    credentials: Option<Credentials>,
    state: S,
}

impl Backend<initial::Initial> {
    pub fn new(receiver: mpsc::Receiver<Input>) -> Self {
        Self {
            receiver,
            credentials: None,
            state: initial::Initial {},
        }
    }

    pub fn launch() -> iced::subscription::Subscription<Output> {
        iced::subscription::channel(
            std::any::TypeId::of::<Backend<initial::Initial>>(),
            0,
            |mut output| async move {
                // Executed only once, even on repeated calls of subscription
                let (sender, receiver) = mpsc::channel(50);
                let backend = Backend::new(receiver);

                output
                    .send(Output::Connection(Connection::Connected(sender)))
                    .await
                    .expect("Unable to connect to GUI!");

                // Executed continuously, kept alive across calls
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            },
        )
    }
}
