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
    futures::{Stream, StreamExt},
    widget::scrollable,
};

#[derive(Clone, Debug, Default)]
pub struct Home {
    books: Vec<Option<Result<Book, book::Error>>>,
    selected_book: Option<usize>,
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
                message: format!("{message:?}"),
            }),
        }
    }
}

impl Home {
    pub fn new(books: Vec<Option<Result<Book, book::Error>>>) -> Self {
        Self {
            books,
            ..Default::default()
        }
    }

    pub fn update(
        mut self,
        message: Result<Message, crate::backend::Error>,
    ) -> (
        State,
        Option<crate::backend::goodreads::Input>,
        Task<scene::goodreads::Message>,
    ) {
        let output: Option<crate::backend::goodreads::home::Input> = None;
        let state = None;

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

    pub fn view(&self) -> iced::Element<'_, Message> {
        /*******************
         * Book comparison *
         *******************/
        // Display cover
        // Display title
        // Display author
        // Display blurb
        // Display page count

        let book = if let Some(id) = self.selected_book
            && let Some(book) = &self.books[id]
        {
            book
        } else {
            &Ok(book::Book::default())
        };

        let book = book
            .as_ref()
            .expect("Handling of books that failed to download not yet implemented");

        let comparisons = iced::widget::row![
            self.book_comparison(book.clone()),
            self.book_comparison(book::Book::default())
        ];

        // let book = match self.selected_book {
        //     Some(id) => self.books[id],
        //     None => todo!(),
        // };

        /*****************
         * Grid of books *
         *****************/
        let cover_placeholder =
            iced::widget::image::Handle::from_bytes(book::COVER_PLACEHOLDER_DATA);
        let covers: Vec<_> = self
            .books
            .iter()
            .map(|book| match book {
                Some(book) => match book {
                    Ok(book) => &book.cover,
                    Err(error) => &cover_placeholder,
                },
                None => &cover_placeholder,
            })
            .collect();

        let mut covers: Vec<_> = covers
            .into_iter()
            .enumerate()
            .map(|(i, cover)| {
                iced::widget::button(iced::widget::image(cover))
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
                columns.push(std::mem::take(&mut covers));

                columns.into_iter().map(|covers| {
                    iced::widget::column(covers.into_iter().map(iced::Element::new))
                        .spacing(grid_spacing)
                        .into()
                })
            })
            .spacing(grid_spacing),
        )
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::new(),
        ))
        .into()
    }

    fn book_comparison(&self, book: book::Book) -> iced::Element<'_, Message> {
        // let book = book::Book::default();
        let comparison = iced::widget::row![
            iced::widget::image(book.cover).height(iced::Fill),
            iced::widget::column![
                iced::widget::container(iced::widget::text(book.title)).padding(5),
                iced::widget::container(iced::widget::text(book.author)).padding(5),
                iced::widget::horizontal_rule(2),
                iced::widget::scrollable(
                    iced::widget::container(iced::widget::text(book.blurb)).padding(5)
                )
                .direction(scrollable::Direction::Vertical(scrollable::Scrollbar::new())) // .spacing(5)
            ]
        ];

        comparison.into()
    }
}

pub fn fetch_books(books: Vec<BookInfo>) -> impl Stream<Item = (usize, Result<Book, book::Error>)> {
    let number_of_books = books.len();
    let client = reqwest::Client::new();
    let mut book_requests = vec![];
    for (i, BookInfo { title, url }) in books.into_iter().enumerate() {
        // Cloning the client *should* be okay, because it uses an Arc internally. So, new clones should refer to the same client after all
        let client = client.clone();
        book_requests.push(async move {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            println!("Fetching book {}/{}:", i + 1, number_of_books);
            println!("Url: {url}");
            (i, Book::fetch(url, &client).await)
        })
    }

    iced::futures::stream::iter(book_requests).buffered(5)
}
