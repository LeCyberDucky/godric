use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};
use scraper::{Html, Selector};

#[tokio::main]
async fn main() -> Result<()> {
    // https://www.goodreads.com/review/list/176878294?shelf=to-read
    let user_id = "176878294";
    let bookshelf_link = url::Url::parse(&format!(
        "https://www.goodreads.com/review/list/{user_id}?shelf=to-read"
    ))
    .context("Unable to create link to reading list")?;

    // let client = reqwest::Client::new();
    // let bookshelf = client.get(bookshelf_link).send().await.context("Unable to load bookshelf")?;
    // let bookshelf = bookshelf.text().await.context("Failed to read bookshelf content")?;
    // std::fs::write("Data/bookshelf_long.html", &bookshelf);
    let bookshelf = std::fs::read_to_string("Data/bookshelf_medium.html")?;

    let bookshelf = Html::parse_document(&bookshelf);
    let bookshelf = bookshelf
        .select(&Selector::parse("#rightCol").expect("Failed to create CSS selector"))
        .next()
        .context("Failed to read bookshelf content")?;

    let page_count = {
        let menu = bookshelf
            .select(&Selector::parse("#reviewPagination").expect("Failed to create CSS selector"))
            .next();

        match menu {
            None => 1, // If we can't find the page navigation menu, we assume that there's only one single page
            Some(menu) => {
                let button_selector = Selector::parse("a").expect("Failed to create CSS selector");
                let count = menu.select(&button_selector).count();
                let pages = menu
                    .select(&button_selector)
                    .nth(count - 2)
                    .expect("Failed to count bookshelf pages")
                    .inner_html()
                    .parse()?;
                pages
            }
        }
    };

    dbg!(bookshelf);
    dbg!(page_count);

    // Parse books on a page
    let books: Vec<_> = bookshelf
        .select(
            &Selector::parse(r#"tr[class="bookalike review"]"#)
                .expect("Failed to create CSS selector"),
        )
        .collect();

    for book in &books {
        dbg!(book);
    }

    let book = books.first().unwrap();

    // let position: usize = book.select(&Selector::parse(r#"td[class="field position"] div"#).unwrap()).next().unwrap().text().next().unwrap().trim().parse()?;
    let position: usize = book
        .select(&Selector::parse(r#"td[class="field position"] div"#).unwrap())
        .next()
        .unwrap()
        .inner_html()
        .trim()
        .parse()?;

    let book = book
        .select(
            &Selector::parse(r#"td[class="field title"] a"#)
                .expect("Failed to create CSS selector"),
        )
        .next()
        .unwrap();
    let title = book.text().next().unwrap().trim();
    let link = book.attr("href").context("Failed to obtain book link")?;
    let link = url::Url::parse("https://goodreads.com")
        .unwrap()
        .join(link)?;

    dbg!(title);
    dbg!(link);
    dbg!(position);

    for i in 1..=page_count {
        let mut link = bookshelf_link.clone();
        link.query_pairs_mut().append_pair("page", &i.to_string());
        println!("{}", link);
    }

    Ok(())
}
