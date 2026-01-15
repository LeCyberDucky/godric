use crate::{
    common::sorting::Sorting,
    scene::{
        self,
        goodreads::{
            State,
            book::{self, Book},
        },
    },
};

use color_eyre::Result;
use iced::Task;
use strum::IntoEnumIterator;

#[derive(Clone, Debug, Default)]
pub struct Home {
    books: Sorting<(url::Url, Option<Result<Book, book::Error>>)>,
    placeholder: Book,
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
    Sort {
        direction: crate::common::sorting::Half,
    },
    SearchSpaceSelected(crate::common::sorting::SearchSpace),
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
            books: Sorting::new(books),
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
                Message::BookSelected(selection) => self
                    .books
                    .select(selection)
                    .expect("Selected book outside range."),
                Message::BookDisplay(message) => {
                    dbg!(message);
                    todo!()
                }
                Message::Sort { direction } => {
                    self.books.step(direction);
                }
                Message::SearchSpaceSelected(search_space) => {
                    self.books.set_search_space(search_space)
                }
            },
            Err(error) => todo!(),
        }

        (
            self.into(),
            output.map(|output| output.into()),
            Task::none(),
        )
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        /*******************
         * Book comparison *
         *******************/
        let book = self
            .books
            .selection()
            .and_then(|selection| selection.1.as_ref())
            .and_then(|book| book.as_ref().ok())
            .unwrap_or(&self.placeholder);

        let candidate = self
            .books
            .candidate()
            .and_then(|candidate| candidate.1.as_ref())
            .and_then(|book| book.as_ref().ok())
            .unwrap_or(&self.placeholder);

        let select_button = |direction| {
            iced::widget::container(
                iced::widget::button(iced::widget::text("Select").width(iced::Fill).center())
                    .on_press(Message::Sort { direction }),
            )
        };

        let comparison = iced::widget::row![
            iced::widget::column![
                book.view().map(Message::BookDisplay),
                select_button(crate::common::sorting::Half::Front).padding(iced::padding::right(1))
            ],
            iced::widget::column![
                candidate.view().map(Message::BookDisplay),
                select_button(crate::common::sorting::Half::Back).padding(iced::padding::left(1))
            ]
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

        /************
         * Settings *
         ************/
        let search_space_selection = {
            let text = iced::widget::text("Search space:");

            let list = iced::widget::pick_list(
                crate::common::sorting::SearchSpace::iter()
                    .map(|mode| mode.to_string())
                    .collect::<Vec<_>>(),
                Some(self.books.search_space().to_string()),
                |selection| {
                    Message::SearchSpaceSelected(
                        crate::common::sorting::SearchSpace::try_from(selection.as_str())
                            .expect("Invalid search space selected!"),
                    )
                },
            );

            iced::widget::container(
                iced::widget::row![text, list]
                    .align_y(iced::Alignment::Center)
                    .spacing(2)
                    .padding(2),
            )
            .align_right(iced::Fill)
        };

        iced::widget::column![comparison, grid, search_space_selection]
            .spacing(2)
            .into()
    }
}

fn grid(
    mut items: Vec<iced::Element<'_, Message>>,
    column_height: usize,
    spacing: u32,
) -> iced::Element<'_, Message> {
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
