use std::{path::PathBuf, sync::Arc};

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
use tempfile::TempDir;

#[derive(Clone, Debug)]
pub struct Home {
    books: Vec<(url::Url, Option<Result<Book, book::Error>>)>,
    selected_book: Option<usize>,
    placeholder: Book,
}

impl Default for Home {
    fn default() -> Self {
        Self {
            books: Default::default(),
            selected_book: Default::default(),
            placeholder: Default::default(),
        }
    }
}

impl From<Home> for State {
    fn from(state: Home) -> Self {
        Self::Home(state)
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    BookFetched {
        url: url::Url,
        book: Result<crate::backend::goodreads::book::Book, crate::backend::goodreads::book::Error>,
    },
    BookSelected(usize),
}

impl From<Message> for scene::goodreads::Message {
    fn from(message: Message) -> Self {
        Self::Home(message)
    }
}

impl From<crate::backend::goodreads::home::Output> for Message {
    fn from(output: crate::backend::goodreads::home::Output) -> Self {
        match output {
            crate::backend::goodreads::home::Output::Book { url, book } => {
                Self::BookFetched { url, book }
            }
        }
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
    pub fn new(books: Vec<(url::Url, Option<Result<Book, book::Error>>)>) -> Self {
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
                Message::BookFetched { url, book } => {
                    if let Err(ref error) = book {
                        todo!("{error}")
                    }

                    match self.books.iter_mut().find(|element| element.0 == url) {
                        Some(entry) => {
                            let book = book.map(|content| {
                                content.try_into().expect("Book should exist in GUI list")
                            });
                            entry.1 =
                                Some(book.map_err(|e| {
                                    scene::goodreads::book::Error::Other(e.to_string())
                                }))
                        }
                        None => todo!(),
                    }
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

        let book = self
            .selected_book
            .and_then(|id| self.books[id].1.as_ref())
            .and_then(|book| book.as_ref().ok())
            .unwrap_or(&self.placeholder);

        let comparisons = iced::widget::row![
            self.book_comparison(book),
            self.book_comparison(&self.placeholder)
        ];

        /*****************
         * Grid of books *
         *****************/
        let covers: Vec<_> = self
            .books
            .iter()
            .map(|(url, book)| match book {
                Some(book) => match book {
                    Ok(book) => &book.thumbnail,
                    Err(error) => &self.placeholder.thumbnail,
                },
                None => &self.placeholder.thumbnail,
            })
            .collect();

        let mut covers: Vec<_> = covers
            .into_iter()
            .enumerate()
            .map(|(i, cover)| {
                iced::widget::button(iced::widget::image(cover))
                    .on_press(Message::BookSelected(i))
                    .padding(iced::Padding::new(4.0))
            })
            .collect();

        let grid_height = 1;
        let grid_spacing = 1;

        let grid = scrollable(
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
        .horizontal()
        .auto_scroll(true)
        .spacing(0);
        // .into();

        // iced::widget::container([comparisons]).into()
        iced::widget::column![comparisons, grid].into()
    }

    fn book_comparison<'a>(&self, book: &'a book::Book) -> iced::Element<'a, Message> {
        let comparison = iced::widget::row![
            iced::widget::image(book.thumbnail.clone()).height(iced::Fill),
            iced::widget::column![
                iced::widget::container(iced::widget::text(&book.info.title)).padding(5),
                iced::widget::container(iced::widget::text(&book.info.author)).padding(5),
                iced::widget::rule::horizontal(2),
                iced::widget::scrollable(
                    iced::widget::container(iced::widget::text(&book.info.blurb)).padding(5)
                )
                .direction(scrollable::Direction::Vertical(scrollable::Scrollbar::new()))
                .spacing(0)
            ]
        ];

        comparison.into()
    }
}
