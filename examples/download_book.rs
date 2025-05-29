use color_eyre::eyre::{Error, Result};
use godric::scene::goodreads::book::Book;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let link = url::Url::parse(
        "https://www.goodreads.com/book/show/203578812-i-m-starting-to-worry-about-this-black-box-of-doom",
    )?;
    let client = reqwest::Client::new();

    let scene = godric::scene::goodreads::home::Home::default();

    let book = Book::fetch(link, &client, scene.cache_path()).await?;
    dbg!(book);
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    Ok(())
}
