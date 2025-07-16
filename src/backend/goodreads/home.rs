use crate::backend::goodreads::{
    self, State,
    book::{BookInfo, BookList},
};
use color_eyre::{
    Result,
    eyre::{Context, ContextCompat},
};
use iced::futures::stream::StreamExt;
use scraper::{Html, Selector};
use tempfile::TempDir;
use thirtyfour as tf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid message ({message}) for state {state}")]
    InvalidState { state: String, message: String },
    #[error(transparent)]
    Other(#[from] color_eyre::Report),
}

#[derive(Clone, Debug)]
pub enum Input {
    Tick,
}

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
            super::Input::Tick => Ok(Self::Tick),
            _ => Err(Self::Error::InvalidState {
                state: "Home".into(),
                message: format!("{input:?}"),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Output {
    Book(Result<goodreads::book::Book, goodreads::book::Error>),
}

impl From<Output> for goodreads::Output {
    fn from(output: Output) -> Self {
        Self::Home(output)
    }
}

#[derive(Debug)]
pub struct Home {
    user_id: String,
    books: BookList,
}

impl From<Home> for State {
    fn from(state: Home) -> Self {
        Self::Home(state)
    }
}

impl Home {
    pub fn new(user_id: String, books: Vec<BookInfo>) -> Self {
        let books: Vec<_> = books.into_iter().map(|info| info.url).collect();
        let books = BookList::new(
            books,
            reqwest::Client::new(),
            TempDir::new()
                .expect("Failed to create temporary directory to store book covers")
                .into(),
        );
        Self { user_id, books }
    }

    pub async fn update(
        mut self,
        _browser: &mut tf::WebDriver,
        input: Input,
    ) -> Result<(State, Option<goodreads::Output>), Error> {
        // Book downloading struct that implements stream
        // This means that we can call next on the thing to get a single new book
        // Then we can transition --> send the book as output, and take the struct with us into the next state

        // let output = None;
        // if let Some(book) = self.books.queue.next().await {
        //     output =
        //     println!("Downloaded book: {}!", book.unwrap().title);
        // }

        let output = self.books.queue.next().await.map(Output::Book);

        Ok((self.into(), output.map(|output| output.into())))
    }
}

pub async fn fetch_booklist(user_id: &str) -> Result<Vec<BookInfo>, Error> {
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

    // https://stackoverflow.com/questions/51044467/how-can-i-perform-parallel-asynchronous-http-get-requests-with-reqwest
    let page_count = parse_bookshelf_page_count(&bookshelf)?;
    let mut bookshelf_requests = vec![];
    for page in 1..=page_count {
        let mut link = bookshelf_link.clone();
        link.query_pairs_mut()
            .append_pair("page", &page.to_string());
        let client = &client;
        bookshelf_requests.push(async move {
            // Don't want to DOS Amazon with our handful of requests
            let sleep_time = 100 + (100.0 * rand::random::<f64>()) as u64;
            tokio::time::sleep(std::time::Duration::from_millis(sleep_time)).await;

            println!("Fetching bookshelf page {page}/{page_count}");

            let bookshelf = client
                .get(link)
                .send()
                .await
                .context("Unable to load bookshelf")?
                .text()
                .await
                .context("Failed to read bookshelf content")?;
            parse_bookshelf_page_books(&bookshelf)
        });
    }
    let bookshelf_pages: Vec<_> = iced::futures::stream::iter(bookshelf_requests)
        .buffer_unordered(5)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    let mut books: Vec<_> = bookshelf_pages
        .into_iter()
        .flatten()
        .collect::<Result<_, _>>()?;

    // Sort collection of books according to user sorting
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

                menu.select(&button_selector)
                    .nth(count - 2)
                    .context("Failed to count bookshelf pages")?
                    .inner_html()
                    .trim()
                    .parse()
                    .context("Failed to parse bookshelf page count")?
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
            let link = url::Url::parse("https://www.goodreads.com")
                .unwrap()
                .join(link)
                .context("Failed to create book link")?;

            Ok((position, BookInfo { title, url: link }))
        })
        .collect();

    Ok(books)
}
