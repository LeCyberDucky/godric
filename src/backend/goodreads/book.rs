use color_eyre::eyre::{Context, ContextCompat, Result};
use futures::StreamExt;
use image::GenericImage;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Clone, Debug)]
pub struct BookInfo {
    pub title: String,
    pub url: url::Url,
}

const COVER_PLACEHOLDER_PATH: &str = r"..\..\..\Assets\Icons\cover_placeholder.jpg";

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Image(String),
    #[error("{0}")]
    Other(String),
}

impl From<color_eyre::eyre::ErrReport> for Error {
    fn from(error: color_eyre::eyre::ErrReport) -> Self {
        let mut description = String::new();
        for cause in error.chain() {
            description += &(cause.to_string() + "\n");
        }
        Self::Other(description)
    }
}

#[derive(Clone, Debug)]
pub struct Book {
    pub url: url::Url,
    pub title: String,
    pub author: String,
    pub blurb: String,
    pub cover_cache: std::path::PathBuf,
}

impl Default for Book {
    fn default() -> Self {
        let url = url::Url::parse("https://127.0.0.1").expect("Failed to parse loopback ip");
        let title = "Placeholders for dummies, First Edition".to_string();
        let author = "Max Mustermann".to_string();
        let blurb = "Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquid ex ea commodi consequat. Quis aute iure reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint obcaecat cupiditat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string();
        let cover_cache = COVER_PLACEHOLDER_PATH.into();
        Self {
            url,
            title,
            author,
            blurb,
            cover_cache,
        }
    }
}

impl Book {
    pub async fn fetch(
        url: url::Url,
        client: &reqwest::Client,
        image_dir: std::path::PathBuf,
    ) -> Result<Self, Error> {
        use scraper::Html;
        use scraper::Selector;

        let page = client
            .get(url.clone())
            .send()
            .await
            .context("Unable to load book page")?
            .text()
            .await
            .context("Unable to read book page")?;

        let (title, author, blurb, image_source) = {
            let page = Html::parse_document(&page);

            let title = page
                .select(&Selector::parse(r#"h1[class="Text Text__title1"]"#).unwrap())
                .next()
                .context("Failed to select title")?
                .inner_html()
                .trim()
                .to_string();

            let author = page
                .select(&Selector::parse(r#"span[class="ContributorLink__name"]"#).unwrap())
                .next()
                .context("Failed to select author")?
                .inner_html()
                .trim()
                .to_string();

            let blurb = page
                .select(&Selector::parse(r#"span[class="Formatted"]"#).unwrap())
                .next()
                .context("Failed to select blurb")?
                .inner_html()
                .trim()
                .to_string();

            let image_source: url::Url = page
                .select(&Selector::parse(r#"img[class="ResponsiveImage"]"#).unwrap())
                .next()
                .context("Failed to select cover image")?
                .attr("src")
                .context("Failed to obtain cover image source")?
                .parse()
                .context("Invalid url for cover image")?;

            (title, author, blurb, image_source)
        };

        let cover_cache = cache_book_cover(image_source, image_dir, client).await?;

        Ok(Self {
            url,
            title,
            author,
            blurb,
            cover_cache,
        })
    }
}

/// Downloads a book cover image from a given source and stores it to the given cache directory.
///
/// # Errors
///
/// This function will return an error if it fails to download a valid book cover.
async fn cache_book_cover(
    source: url::Url,
    cache_directory: std::path::PathBuf,
    client: &reqwest::Client,
) -> Result<std::path::PathBuf, Error> {
    let cover_data = client
        .get(source.clone())
        .send()
        .await
        .context("Failed to request cover image")?
        .bytes()
        .await
        .context("Failed to download cover image")?;

    tokio::task::spawn_blocking(move || {
        let cover = image::load_from_memory(&cover_data)
            .context("Failed to interpret downloaded bytes as cover image")?;

        let filepath = cache_directory
            .join(uuid::Uuid::new_v4().to_string())
            .with_extension("png");

        dbg!(&filepath);
        // We drop the alpha channel to make things easier for ourselves - To cite Kasper: Why should a book cover have an alpha channel?
        cover
            .to_rgb8()
            .save(&filepath)
            .context("Failed to cache cover image")?;

        Ok(filepath)
    })
    .await
    .context("Failed to process cover image")?
}

pub struct BookList {
    pub queue:
        std::pin::Pin<Box<dyn futures::Stream<Item = (url::Url, Result<Book, Error>)> + Send>>,
    books: Vec<Book>,
    cache: crate::common::cache::Config
}

impl std::fmt::Debug for BookList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BookList")
            .field("books", &self.books)
            .finish()
    }
}

impl BookList {
    pub fn new(
        urls: Vec<url::Url>,
        http_client: reqwest::Client,
        cache: crate::common::cache::Config,
    ) -> Self {
        let number_of_urls = urls.len();
        let mut work = vec![];
        for (i, url) in urls.into_iter().enumerate() {
            let client = http_client.clone();
            let image_dir = cache.active_path().to_path_buf();
            work.push(async move {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                println!("Fetching book {}/{}:", i + 1, number_of_urls);
                println!("Url: {url}");
                (
                    url.clone(),
                    Book::fetch(url, &client, image_dir.as_path().into()).await,
                )
            });
        }

        let queue = futures::stream::iter(work).buffered(5).boxed();

        Self {
            queue,
            books: vec![],
            cache,
        }
    }
}
