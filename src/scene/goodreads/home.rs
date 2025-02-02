use crate::{
    backend::goodreads::book::BookInfo,
    scene::{
        self,
        goodreads::{
            State,
            book::{self, Book},
        },
    },
};

use color_eyre::Result;
use iced::{
    Task,
    futures::{SinkExt, Stream},
    widget::scrollable,
};

#[derive(Clone, Debug, Default)]
pub struct Home {
    books: Vec<Option<Result<Book, book::Error>>>,
    selected_book: Option<usize>
}

impl From<Home> for State {
    fn from(state: Home) -> Self {
        Self::Home(state)
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    BookFetched((usize, Result<Book, book::Error>)),
    BookSelected(usize),
}

impl From<Message> for scene::goodreads::Message {
    fn from(message: Message) -> Self {
        Self::Home(message)
    }
}

impl TryFrom<scene::goodreads::Message> for Message {
    type Error = crate::backend::Error;

    fn try_from(message: scene::goodreads::Message) -> Result<Self, Self::Error> {
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
    pub fn new(books: Vec<Option<Result<Book, book::Error>>>) -> Self {
        Self { books, ..Default::default() }
    }

    pub fn update(
        mut self,
        message: Result<Message, crate::backend::Error>,
    ) -> (
        State,
        Option<crate::backend::goodreads::Input>,
        Task<scene::goodreads::Message>,
    ) {
        let mut output: Option<crate::backend::goodreads::home::Input> = None;
        let mut state = None;

        match message {
            Ok(message) => match message {
                Message::BookFetched((i, book)) => {
                    if let Err(ref error) = book {
                        todo!("{error}")
                    }

                    self.books[i] = Some(book);
                }
                Message::BookSelected(selection) => self.selected_book = Some(selection),
            },
            Err(error) => todo!(),
        }

        (
            state.unwrap_or(self.into()),
            output.map(|output| output.into()),
            Task::none(),
        )
    }

    pub fn view(&self) -> iced::Element<Message> {
        const COVER_PLACEHOLDER_DATA: &'static [u8] =
            include_bytes!(r"..\..\..\Assets\Icons\cover_placeholder.jpg");
        let cover_placeholder = iced::widget::image::Handle::from_bytes(COVER_PLACEHOLDER_DATA);

        let covers: Vec<_> = self
            .books
            .iter()
            .map(|book| match book {
                Some(book) => match book {
                    Ok(book) => book.cover(),
                    Err(error) => &cover_placeholder,
                },
                None => &cover_placeholder,
            })
            .collect();

        let mut covers: Vec<_> = covers
            .iter()
            .enumerate()
            .map(|(i, cover)| {
                iced::widget::button(iced::widget::image(cover.clone()))
                    .on_press(Message::BookSelected(i))
                    .width(iced::Length::Fixed(100.0))
                    .padding(iced::Padding::new(2.0))
            })
            .collect();

        let grid_height = 3;
        let grid_spacing = 0;
        scrollable(
            iced::widget::row({
                let mut columns = vec![];
                while grid_height <= covers.len() {
                    columns.push(covers.drain(..grid_height).collect::<Vec<_>>());
                }
                columns.push(covers.drain(..).collect::<Vec<_>>());

                columns.into_iter().map(|covers| {
                    iced::widget::column(covers.into_iter().map(|cover| iced::Element::new(cover)))
                        .spacing(grid_spacing)
                        .into()
                })
            })
            .spacing(grid_spacing),
        )
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::new().anchor(scrollable::Anchor::default()),
        ))
        .into()
    }
}

pub fn fetch_books(books: Vec<BookInfo>) -> impl Stream<Item = (usize, Result<Book, book::Error>)> {
    iced::stream::channel(1, move |mut output| async move {
        let number_of_books = books.len();
        let client = reqwest::Client::new();
        for (i, BookInfo { title, url }) in books.into_iter().enumerate() {
            println!("Fetching book {}/{}:", i + 1, number_of_books);
            println!("Url: {url}");
            let book = Book::fetch(url, &client).await;
            output.send((i, book)).await;
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    })
}
