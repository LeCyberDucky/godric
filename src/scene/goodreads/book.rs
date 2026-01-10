use std::ops::{Deref, DerefMut};

use color_eyre::eyre::{ContextCompat, Result};
use image::GenericImage;

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
    pub info: crate::backend::goodreads::book::Book,
    pub thumbnail: iced::widget::image::Handle,
}

impl TryFrom<crate::backend::goodreads::book::Book> for Book {
    type Error = Error;

    fn try_from(
        info: crate::backend::goodreads::book::Book,
    ) -> std::result::Result<Self, Self::Error> {
        let thumbnail = image::ImageReader::open(&info.cover_cache)
            .map_err(|error| Error::Image(error.to_string()))?
            .decode()
            .map_err(|error| Error::Image(error.to_string()))?;
        let thumbnail =
            create_thumbnail(&thumbnail, Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT)?;
        let thumbnail = iced::widget::image::Handle::from_rgba(
            thumbnail.width(),
            thumbnail.height(),
            thumbnail.as_bytes().to_owned(),
        );
        Ok(Self {
            info,
            thumbnail: thumbnail,
        })
    }
}

impl Default for Book {
    fn default() -> Self {
        let thumbnail = iced::widget::image::Handle::from_bytes(COVER_PLACEHOLDER_THUMBNAIL);
        Self {
            info: Default::default(),
            thumbnail,
        }
    }
}

impl Book {
    // 6x9 is a common aspect ratio for fiction books. See: https://blog.reedsy.com/guide/book-design/book-cover-dimensions/
    pub const THUMBNAIL_WIDTH: u32 = 84;
    pub const THUMBNAIL_HEIGHT: u32 = 126;

    pub fn view<'a, T: 'a>(&'a self) -> iced::Element<'a, T> {
        use iced::widget;
        let display = widget::row![
            widget::image(self.thumbnail.clone()).height(iced::Fill),
            widget::column![
                widget::container(widget::text(&self.info.title)).padding(5),
                widget::container(widget::text(&self.info.author)).padding(5),
                widget::rule::horizontal(2),
                widget::scrollable(widget::container(widget::text(&self.info.blurb)).padding(5))
                    .direction(widget::scrollable::Direction::Vertical(
                        widget::scrollable::Scrollbar::new()
                    ))
                    .spacing(0)
            ]
        ];

        display.into()
    }
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
