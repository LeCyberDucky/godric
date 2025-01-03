use crate::{
    backend::goodreads::book::BookInfo,
    scene::goodreads::{book::{self, Book}, State},
};

use color_eyre::Result;
use iced::{
    Task,
    futures::{SinkExt, Stream},
};

#[derive(Clone, Debug)]
pub struct Home {
    user_id: String,
    books: Vec<Option<Result<Book, book::Error>>>
}

impl From<Home> for State {
    fn from(state: Home) -> Self {
        Self::Home(state)
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    BookFetched(Result<Book, book::Error>),
}

impl TryFrom<crate::scene::goodreads::Message> for Message {
    type Error = crate::backend::Error;

    fn try_from(message: crate::scene::goodreads::Message) -> Result<Self, Self::Error> {
        match message {
            super::Message::Home(message) => Ok(message),
            _ => Err(Self::Error::InvalidState {
                state: "Home".into(),
                message: format!("{:?}", message),
            }),
        }
    }
}

impl Home {
    pub fn new(user_id: String) -> Self {
        Self { user_id, books: vec![] }
    }

    pub fn update(
        mut self,
        message: Result<Message, crate::backend::Error>,
    ) -> (State, Option<crate::backend::goodreads::Input>) {
        let mut output: Option<crate::backend::goodreads::home::Input> = None;
        let mut state = None;

        match message {
            Ok(message) => match message {
                Message::BookFetched(book) => todo!(),
            },
            Err(error) => todo!(),
        }


        (
            state.unwrap_or(self.into()),
            output.map(|output| output.into()),
        )
    }

    pub fn view(&self) -> iced::Element<Message> {
        todo!()
    }
}

fn fetch_books(books: &Vec<BookInfo>) -> impl Stream<Item = Result<Book, book::Error>> {
    iced::stream::try_channel(1, move |mut output| async move {
        let client = reqwest::Client::new();
        for BookInfo { title, url } in books {
            let book = Book::fetch(url.clone(), &client).await?;
            output.send(book).await;
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        Ok(())
    })
}
