use color_eyre::eyre::{Context, ContextCompat, Result};
use scraper::{Html, Selector};

pub const COVER_PLACEHOLDER_DATA: &'static [u8] =
    include_bytes!(r"..\..\..\Assets\Icons\cover_placeholder.jpg");

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Other(String),
}

impl From<color_eyre::eyre::ErrReport> for Error {
    fn from(error: color_eyre::eyre::ErrReport) -> Self {
        Self::Other(error.to_string())
    }
}

#[derive(Clone, Debug)]
pub struct Book {
    pub url: url::Url,
    pub title: String,
    pub author: String,
    pub blurb: String,
    pub cover: iced::widget::image::Handle,
}

impl Default for Book {
    fn default() -> Self {
        let url = url::Url::parse("https://127.0.0.1").expect("Failed to parse loopback ip");
        let title = "Placeholders for dummies, First Edition".to_string();
        let author = "Max Mustermann".to_string();
        let blurb = "Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquid ex ea commodi consequat. Quis aute iure reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint obcaecat cupiditat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string();
        let cover = iced::widget::image::Handle::from_bytes(COVER_PLACEHOLDER_DATA);
        Self {
            url,
            title,
            author,
            blurb,
            cover,
        }
    }
}

impl Book {
    pub async fn fetch(url: url::Url, client: &reqwest::Client) -> Result<Self, Error> {
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

            let image_source = page
                .select(&Selector::parse(r#"img[class="ResponsiveImage"]"#).unwrap())
                .next()
                .context("Failed to select cover image")?
                .attr("src")
                .context("Failed to obtain cover image source")?
                .to_string();

            (title, author, blurb, image_source)
        };

        let cover = client
            .get(image_source)
            .send()
            .await
            .context("Failed to request cover image")?
            .bytes()
            .await
            .context("Failed to download cover image")?;
        let cover = iced::widget::image::Handle::from_bytes(cover);

        Ok(Self {
            url,
            title,
            author,
            blurb,
            cover,
        })
    }
}
