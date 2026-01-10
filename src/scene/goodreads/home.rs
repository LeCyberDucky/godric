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
    BookDisplay(crate::scene::goodreads::book::Message),
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
                Message::BookDisplay(message) => {
                    dbg!(message);
                    todo!()
                }
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
        let book = self
            .selected_book
            .and_then(|id| self.books[id].1.as_ref())
            .and_then(|book| book.as_ref().ok())
            .unwrap_or(&self.placeholder);

        let comparison = iced::widget::row![
            book.view().map(Message::BookDisplay),
            self.placeholder.view().map(Message::BookDisplay)
        ];

        /*****************
         * Grid of books *
         *****************/
        let covers: Vec<_> = self
            .books
            .iter()
            .map(|(_, book)| {
                book.as_ref()
                    .and_then(|book| book.as_ref().ok())
                    .map(|book| &book.thumbnail)
                    .unwrap_or(&self.placeholder.thumbnail)
            })
            .enumerate()
            .map(|(i, cover)| {
                iced::widget::button(iced::widget::image(cover))
                    .on_press(Message::BookSelected(i))
                    .padding(iced::Padding::new(4.0))
                    .into()
            })
            .collect();

        let grid = grid(covers, 1, 1);

        iced::widget::column![comparison, grid].into()
    }
}

fn grid(
    mut items: Vec<iced::Element<Message>>,
    column_height: usize,
    spacing: u32,
) -> iced::Element<Message> {
    let mut columns = Vec::new();
    while items.len() >= column_height {
        columns.push(items.drain(..column_height).collect());
    }
    if !items.is_empty() {
        columns.push(items);
    }
    let row = iced::widget::row(
        columns
            .into_iter()
            .map(|column| iced::widget::column(column).spacing(spacing).into()),
    )
    .spacing(spacing);
    iced::widget::scrollable(row)
        .horizontal()
        .auto_scroll(true)
        .spacing(spacing)
        .into()
}
