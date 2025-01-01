use crate::{
    backend::{
        self,
        goodreads::{self, State},
    },
    common::book::{self, BookInfo},
};
use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};
use rand::Rng;
use scraper::{Html, Selector};
use thirtyfour as tf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid message ({message}) for state {state}")]
    InvalidState { state: String, message: String },
    #[error(transparent)]
    Other(#[from] color_eyre::Report),
}

#[derive(Clone, Debug)]
pub enum Input {}

impl From<Input> for goodreads::Input {
    fn from(input: Input) -> Self {
        Self::Home(input)
    }
}

impl TryFrom<goodreads::Input> for Input {
    type Error = Error;

    fn try_from(input: goodreads::Input) -> Result<Self, Self::Error> {
        match input {
            super::Input::Home(input) => Ok(input),
            _ => Err(Self::Error::InvalidState {
                state: "Home".into(),
                message: format!("{:?}", input),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Output {}

#[derive(Clone, Debug)]
pub struct Home {
    user_id: String,
    books: Vec<book::BookInfo>,
}

impl From<Home> for State {
    fn from(state: Home) -> Self {
        Self::Home(state)
    }
}

impl Home {
    pub async fn new(user_id: String) -> Result<Self, Error> {
        let books = fetch_books(&user_id).await?;
        Ok(Self { user_id, books })
    }

    pub async fn update(
        self,
        _browser: &mut tf::WebDriver,
        input: Input,
    ) -> Result<(State, Option<goodreads::Output>), Error> {
        Ok((self.into(), None))
    }
}

async fn fetch_books(user_id: &str) -> Result<Vec<book::BookInfo>, Error> {
    let bookshelf_link = url::Url::parse(&format!(
        "https://www.goodreads.com/review/list/{user_id}?shelf=to-read"
    ))
    .context("Unable to create link to reading list")?;

    let client = reqwest::Client::new();
    let bookshelf = client
        .get(bookshelf_link.clone())
        .send()
        .await
        .context("Unable to load bookshelf")?
        .text()
        .await
        .context("Failed to read bookshelf content")?;

    let mut books = vec![];
    for i in 1..=parse_bookshelf_page_count(&bookshelf)? {
        let mut link = bookshelf_link.clone();
        link.query_pairs_mut().append_pair("page", &i.to_string());

        // Don't want to DOS Amazon with our handful of requests
        let sleep_time = 20 + (10.0 * rand::random::<f64>()) as u64;
        tokio::time::sleep(std::time::Duration::from_millis(sleep_time));

        let bookshelf = client
            .get(link)
            .send()
            .await
            .context("Unable to load bookshelf")?
            .text()
            .await
            .context("Failed to read bookshelf content")?;

        books.append(&mut parse_bookshelf_page_books(&bookshelf)?)
    }

    // Flatten to one big result, and sort collection of books according to user sorting
    let mut books: Vec<(usize, BookInfo)> = books.into_iter().collect::<Result<_, _>>()?;
    books.sort_by(|a, b| a.0.cmp(&b.0));
    let books: Vec<_> = books.into_iter().map(|entry| entry.1).collect();

    Ok(books)
}

fn parse_bookshelf_page_count(page: &str) -> Result<usize, Error> {
    let html = Html::parse_document(page);
    let bookshelf = html
        .select(&Selector::parse("#rightCol").unwrap())
        .next()
        .context("Failed to read bookshelf content")?;

    let page_count = {
        let menu = bookshelf
            .select(&Selector::parse("#reviewPagination").unwrap())
            .next();

        match menu {
            None => 1, // If we can't find the page navigation menu, we assume that there's only one single page
            Some(menu) => {
                let button_selector = Selector::parse("a").unwrap();
                let count = menu.select(&button_selector).count();
                let pages = menu
                    .select(&button_selector)
                    .nth(count - 2)
                    .context("Failed to count bookshelf pages")?
                    .inner_html()
                    .trim()
                    .parse()
                    .context("Failed to parse bookshelf page count")?;
                pages
            }
        }
    };
    Ok(page_count)
}

fn parse_bookshelf_page_books(page: &str) -> Result<Vec<Result<(usize, BookInfo), Error>>, Error> {
    let html = Html::parse_document(page);
    let bookshelf = html
        .select(&Selector::parse("#rightCol").unwrap())
        .next()
        .context("Failed to read bookshelf content")?;

    let books: Vec<_> = bookshelf
        .select(&Selector::parse(r#"tr[class="bookalike review"]"#).unwrap())
        .collect();

    let books = books
        .iter()
        .map(|book| {
            let position: usize = book
                .select(&Selector::parse(r#"td[class="field position"] div"#).unwrap())
                .next()
                .context("Unable to obtain book position")?
                .inner_html()
                .trim()
                .parse()
                .context("Failed to parse book position")?;

            let book = book
                .select(&Selector::parse(r#"td[class="field title"] a"#).unwrap())
                .next()
                .context("Unable to obtain book info")?;
            let title = book
                .text()
                .next()
                .context("Unable to obtain book title")?
                .trim()
                .to_string();
            let link = book.attr("href").context("Failed to obtain book link")?;
            let link = url::Url::parse("https://goodreads.com")
                .unwrap()
                .join(link)
                .context("Failed to create book link")?;

            Ok((position, BookInfo { title, url: link }))
        })
        .collect();

    Ok(books)
}
