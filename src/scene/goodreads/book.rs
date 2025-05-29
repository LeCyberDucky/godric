use std::path::PathBuf;

use color_eyre::eyre::{Context, ContextCompat, Result};
use image::EncodableLayout;
use scraper::{Html, Selector};

const COVER_PLACEHOLDER_PATH: &str = r"..\..\..\Assets\Icons\cover_placeholder.jpg";
pub const COVER_PLACEHOLDER_THUMBNAIL: &[u8] =
    include_bytes!(r"..\..\..\Assets\Icons\cover_placeholder_thumbnail.jpg");

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
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
    pub cover_cache: PathBuf,
    pub thumbnail: iced::widget::image::Handle,
}

impl Default for Book {
    fn default() -> Self {
        let url = url::Url::parse("https://127.0.0.1").expect("Failed to parse loopback ip");
        let title = "Placeholders for dummies, First Edition".to_string();
        let author = "Max Mustermann".to_string();
        let blurb = "Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquid ex ea commodi consequat. Quis aute iure reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint obcaecat cupiditat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string();
        let cover = iced::widget::image::Handle::from_bytes(COVER_PLACEHOLDER_THUMBNAIL);
        let cover_cache = COVER_PLACEHOLDER_PATH.into();
        Self {
            url,
            title,
            author,
            blurb,
            cover_cache,
            thumbnail: cover,
        }
    }
}

impl Book {
    pub async fn fetch(
        url: url::Url,
        client: &reqwest::Client,
        image_dir: PathBuf,
    ) -> Result<Self, Error> {
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

        let (thumbnail, cover_cache) = cache_book_cover(image_source, image_dir, client).await?;
        let thumbnail = iced::widget::image::Handle::from_rgba(
            thumbnail.width(),
            thumbnail.height(),
            thumbnail.as_bytes().to_owned(),
        );

        Ok(Self {
            url,
            title,
            author,
            blurb,
            thumbnail,
            cover_cache,
        })
    }
}

/// Downloads a book cover image from a given source, stores it to the given cache directory, and creates a thumbnail version of the image.
///
/// # Errors
///
/// This function will return an error if it fails to download a valid book cover.
async fn cache_book_cover(
    source: url::Url,
    cache_directory: PathBuf,
    client: &reqwest::Client,
) -> Result<(image::RgbaImage, PathBuf), Error> {
    // Store full cover image to file
    // Hash URL to create file name

    // Create thumbnail to keep in memory

    let cover_data = client
        .get(source.clone())
        .send()
        .await
        .context("Failed to request cover image")?
        .bytes()
        .await
        .context("Failed to download cover image")?;

    // let cache_directory = cache_directory.as_ref();
    tokio::task::spawn_blocking(move || {
        let cover = image::load_from_memory(&cover_data)
            .context("Failed to interpret downloaded bytes as cover image")?;

        let image_format_extension = image::guess_format(&cover_data)
            .context("Failed to detect format of downloaded cover image")?
            .extensions_str()
            .first()
            .context("No known file extension for detected image format")?;

        let filepath = cache_directory
            .join(uuid::Uuid::new_v4().to_string())
            .with_extension(image_format_extension);

        cover
            .save(&filepath)
            .context("Failed to cache cover image")?;

        let thumbnail = cover.thumbnail(84, 126).into_rgba8(); // 6x9 is a common aspect ratio for fiction books. See: https://blog.reedsy.com/guide/book-design/book-cover-dimensions/
        Ok((thumbnail, filepath))
    })
    .await
    .context("Failed to process cover image")?
}
