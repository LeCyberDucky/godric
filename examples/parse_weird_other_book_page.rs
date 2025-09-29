use scraper::{Html, Selector};

fn main() {
    let page = std::fs::read_to_string(r"D:\OneDrive\Dokumenter\Andy hjemme\Programmering\Projekte\godric\Data\weird_other_book_page.html").expect("weird_other_book_page.html should be readable");
    let page = Html::parse_document(&page);

    let page = page
        .select(&Selector::parse(r#"script[id="__NEXT_DATA__"]"#).unwrap())
        .next()
        .unwrap()
        .inner_html()
        .trim()
        .to_string();

    let page: serde_json::Value = serde_json::from_str(&page).unwrap();

    let page = &page
        .get("props")
        .and_then(|json| json.get("pageProps"))
        .and_then(|json| json.get("apolloState"))
        .unwrap()
        .as_object()
        .unwrap();

    let author = page
        .keys()
        .find(|dings| dings.contains("Contributor:"))
        .unwrap();
    let author = page
        .get(author)
        .and_then(|inner| inner.get("name"))
        .unwrap();

    let book_info = page.keys().find(|dings| dings.contains("Book:")).unwrap();
    let book_info = page.get(book_info).unwrap();

    let title = book_info.get("titleComplete").unwrap();
    let blurb = book_info.get("description").unwrap();
    let blurb = Html::parse_fragment(blurb.as_str().unwrap()).html();
    let image_url = book_info.get("imageUrl").unwrap();

    dbg!(author);
    dbg!(title);
    dbg!(image_url);
    dbg!(blurb);

    /*
     [examples\parse_weird_other_book_page.rs:24:5] wat = [
    "ROOT_QUERY",
    "User:8042491",
    "Contributor:kca://author/amzn1.gr.author.v1.Q1af3WsexrzELSAIT5BG9Q",
    "Book:kca://book/amzn1.gr.book.v3.kFPCxSjeDGXeVQMI",
    "Work:kca://work/amzn1.gr.work.v3.mVWYpQlloMzbsjJi",]
    */

    // dbg!(page);
}
