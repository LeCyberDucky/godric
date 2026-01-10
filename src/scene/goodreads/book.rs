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
pub enum Message {
    Markdown(iced::widget::markdown::Uri),
}

#[derive(Clone, Debug)]
pub struct Book {
    pub info: crate::backend::goodreads::book::Book,
    pub blurb: Vec<iced::widget::markdown::Item>,
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
        let blurb = iced::widget::markdown::parse(&info.blurb).collect();
        Ok(Self {
            info,
            blurb,
            thumbnail: thumbnail,
        })
    }
}

impl Default for Book {
    fn default() -> Self {
        let info = crate::backend::goodreads::book::Book::default();
        let blurb = iced::widget::markdown::parse(&info.blurb).collect();
        let thumbnail = iced::widget::image::Handle::from_bytes(COVER_PLACEHOLDER_THUMBNAIL);
        Self {
            info,
            blurb,
            thumbnail,
        }
    }
}

impl Book {
    // 6x9 is a common aspect ratio for fiction books. See: https://blog.reedsy.com/guide/book-design/book-cover-dimensions/
    pub const THUMBNAIL_WIDTH: u32 = 84;
    pub const THUMBNAIL_HEIGHT: u32 = 126;

    pub fn view(&self) -> iced::Element<Message> {
        use iced::widget;

        let header = widget::row![
            widget::image(self.thumbnail.clone()).height(iced::Fill),
            widget::column![
                widget::container(
                    widget::column![
                        widget::text(&self.info.title).font(iced::font::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        }),
                        widget::text(&self.info.author).font(iced::font::Font {
                            family: iced::font::Family::SansSerif,
                            ..Default::default()
                        })
                    ]
                    .spacing(2)
                )
                .padding(iced::padding::left(2)),
                widget::rule::horizontal(2)
            ]
        ]
        .align_y(iced::Bottom);

        let header = widget::container(header).align_bottom(Self::THUMBNAIL_HEIGHT);

        let blurb = widget::scrollable(
            widget::container(
                widget::markdown::view(&self.blurb, widget::Theme::TokyoNight)
                    .map(Message::Markdown),
            )
            .padding(5),
        )
        .height(iced::Fill)
        .spacing(0);

        let display = widget::column![header, blurb];

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
