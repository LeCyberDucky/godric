use std::path::PathBuf;

use color_eyre::eyre::{Context, ContextCompat, Result};
use image::GenericImage;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

const COVER_PLACEHOLDER_PATH: &str = r"..\..\..\Assets\Icons\cover_placeholder.jpg";
pub const COVER_PLACEHOLDER_THUMBNAIL: &[u8] =
    include_bytes!(r"..\..\..\Assets\Icons\cover_placeholder_thumbnail.png");

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
    // 6x9 is a common aspect ratio for fiction books. See: https://blog.reedsy.com/guide/book-design/book-cover-dimensions/
    pub const THUMBNAIL_WIDTH: u32 = 84;
    pub const THUMBNAIL_HEIGHT: u32 = 126;

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

    /// Turns the book into a StorableBook suitable for serialization. This involves moving the cached cover from the temporary directory to a permanent directory
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    fn to_storeable_book(
        self,
        cover_directory: impl AsRef<std::path::Path>,
    ) -> Result<StoreableBook> {
        // Move cover image from cache to permanent storage
        let cover_path = self
            .cover_cache
            .file_name()
            .context(format!("Invalid cover path: {:?}", self.cover_cache))?;
        let cover_path = cover_directory.as_ref().join(cover_path);
        if self.cover_cache.canonicalize()? != cover_path.canonicalize()? {
            std::fs::copy(self.cover_cache, &cover_path)?;
        }

        Ok(StoreableBook {
            url: self.url,
            title: self.title,
            author: self.author,
            blurb: self.blurb,
            cover: cover_path,
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
) -> Result<(image::DynamicImage, PathBuf), Error> {
    // Store full cover image to file
    // Create thumbnail to keep in memory

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
            .with_extension(".png");

        // We drop the alpha channel to make things easier for ourselves - To cite Kasper: Why should a book cover have an alpha channel?
        cover
            .to_rgb8()
            .save(&filepath)
            .context("Failed to cache cover image")?;

        let thumbnail = create_thumbnail(&cover, Book::THUMBNAIL_WIDTH, Book::THUMBNAIL_HEIGHT)?;
        Ok((thumbnail, filepath))
    })
    .await
    .context("Failed to process cover image")?
}

pub fn create_thumbnail(
    image: &image::DynamicImage,
    width: u32,
    height: u32,
) -> Result<image::DynamicImage> {
    let pad_colour = okolors::Okolors::try_from(&image.to_rgb8())?
        .parallel(true)
        .sort_by_frequency(true)
        .srgb_palette()
        .into_iter()
        .rev()
        .next()
        .context("Failed to compute palette for image")?
        .into_format();

    let thumbnail = image.thumbnail(width, height);
    let x_offset = (width - thumbnail.width()) / 2;
    let y_offset = (height - thumbnail.height()) / 2;
    let mut padded_thumbnail = image::RgbaImage::from_pixel(
        width,
        height,
        image::Rgba::from([pad_colour.red, pad_colour.green, pad_colour.blue, 255]),
    );
    padded_thumbnail.copy_from(&thumbnail, x_offset, y_offset)?;
    Ok(padded_thumbnail.into())
}

// (De-)serializing a book is a hassle, because the cover image should be stored to and loaded from a separate location
// Instead, we create a helper type that can be obtained from a Book. We take care of storing/loading images during the conversion between the two types.
// Hence, we can just go like this: Book --> StoreableBook --> serialize --> json --> deserialize --> StoreableBook --> Book
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StoreableBook {
    pub url: url::Url,
    pub title: String,
    pub author: String,
    pub blurb: String,
    pub cover: PathBuf,
}

impl TryFrom<StoreableBook> for Book {
    type Error = Error;

    fn try_from(value: StoreableBook) -> std::result::Result<Self, Self::Error> {
        let cover = &image::open(&value.cover).map_err(|error| Error::Image(error.to_string()))?;
        let thumbnail = create_thumbnail(cover, Book::THUMBNAIL_WIDTH, Book::THUMBNAIL_HEIGHT)?;
        let thumbnail = iced::widget::image::Handle::from_rgba(
            thumbnail.width(),
            thumbnail.height(),
            thumbnail.as_bytes().to_owned(),
        );
        Ok(Self {
            url: value.url,
            title: value.title,
            author: value.author,
            blurb: value.blurb,
            cover_cache: value.cover,
            thumbnail,
        })
    }
}
