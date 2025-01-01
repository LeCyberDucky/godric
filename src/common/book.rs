#[derive(Clone, Debug)]
pub struct BookInfo {
    pub title: String,
    pub url: url::Url,
}

struct Book {
    title: String,
    url: url::Url,
}
