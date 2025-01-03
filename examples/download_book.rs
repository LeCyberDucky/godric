use color_eyre::eyre::{Error, Result};
use godric::scene::goodreads::book::Book;

#[tokio::main]
async fn main() -> Result<()> {
    let link = url::Url::parse(
        "https://www.goodreads.com/book/show/72193.Harry_Potter_and_the_Philosopher_s_Stone",
    )?;
    let client = reqwest::Client::new();
    Book::fetch(link, &client).await;
    Ok(())
}
