use color_eyre::Result;
use iced::futures::stream::{FuturesOrdered, FuturesUnordered, StreamExt};
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    // let mut futures = FuturesUnordered::new();
    let mut futures = vec![];
    for i in 0..15 {
        futures.push(do_stuff(i));
    }

    let nows: Vec<_> = iced::futures::stream::iter(futures)
        .buffer_unordered(5)
        .collect()
        .await;

    dbg!(nows);

    Ok(())
}

async fn do_stuff(i: usize) -> String {
    println!("{i}");
    tokio::time::sleep(std::time::Duration::from_millis(5000 + (i * 500) as u64)).await;
    return i.to_string();
}
