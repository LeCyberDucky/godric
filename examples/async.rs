// use color_eyre::Result;
// use futures::stream::{FuturesOrdered, FuturesUnordered, StreamExt};
// use godric::backend::goodreads::book::BookInfo;
// use tokio;
// use tokio_stream::wrappers::ReceiverStream;
// use tokio_stream::wrappers::UnboundedReceiverStream;

use color_eyre::Result;
use futures::stream::StreamExt;
use tokio;

async fn move_test(mut booklist: godric::backend::goodreads::book::BookList) {
    let c = booklist.queue.next().await;
    dbg!(c);
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut booklist = godric::backend::goodreads::book::BookList::new(vec![1, 2, 3, 4, 5, 6, 7]);

    let a = booklist.queue.next().await;
    dbg!(a);
    let b = booklist.queue.next().await;
    dbg!(b);

    move_test(booklist).await;

    Ok(())
}

// struct Book;

// struct BookList {
//     queue // Some kind of queue of book requests?
// }

// impl BookList {
//     pub async fn fetch(book_link: url::Url) -> impl futures::Stream<Item = Book> {
//         // Download books
//         // We may be tasked to download a whole bunch of books very quickly
//         // Therefore, there should be some kind of queue that can be processed concurrently
//         // Since we don't want to make too many web requests, we want to limit this to 5 concurrent fetch requests
//     }
// }

// struct Backend {
//     book_list: BookList
// }

// impl Backend {
//     async fn update(&mut self, book_link: Option<url::Url>) -> Option<Book> {
//         let mut book = None;
//         if let Some(link) = book_link {
//             // How do i send this new request to book_list.fetch()?
//             // I think book_list.fetch() should always be called, even if there's no new request, since old requests may still be in the works
//             book = self.book_list.fetch(book_link).next().await;
//         }

//         return book;
//     }
// }

// #[tokio::main]
// async fn main() -> Result<()> {
//     let mut booklist = godric::backend::goodreads::book::BookList::new();

//     booklist.fetch(Some(BookInfo {
//         title: "Buch".into(),
//         url: url::Url::parse("https://www.google.com").unwrap(),
//     }));

//     while let Some(book) = booklist.fetch(None).next().await {}

//     Ok(())
// }

// #[tokio::main]
// async fn main() -> Result<()> {
//     let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
//     let mut stream = UnboundedReceiverStream::new(receiver);

//     //     let (sender, mut receiver) = tokio::sync::mpsc::channel(5);
//     // let mut stream = ReceiverStream::new(receiver);

//     for i in 0..100 {
//         sender.send(async move {
//             // tokio::time::sleep(std::time::Duration::from_millis(200*i)).await;
//             download_book(format!("Ayy! {i}").into()).await;
//             println!("{i} done!");
//         });
//     }

//     let mut wat = stream.buffered(5);

//     while let dings = wat.next() {
//         let blub = dings.await;
//         dbg!(blub);
//     }

//     // while let Some(wat) = stream.next().await {
//     //     wat.await;
//     // }

//     Ok(())
// }

// async fn download_book(message: String) {
//     println!("{message}");
//     tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
// }

// pub fn dummy_book_fetcher(
//     mut queue: UnboundedReceiver<impl Future<Output = Result<Book, Error>>>,
// ) -> impl Stream<Item = Result<Book, Error>> {
//     let mut stream = UnboundedReceiverStream::new(queue);

//     stream.buffered(5)
//     // futures::stream::iter(vec![dummy_task(), dummy_task(), dummy_task()]).buffered(5)
// }

// async fn dummy_task() -> usize {
//     return 1;
// }
