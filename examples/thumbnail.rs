fn main() {
    let image = image::open("Assets/Icons/cover_placeholder.jpg").unwrap();
    let thumbnail = godric::scene::goodreads::book::create_thumbnail(&image, 84, 126).unwrap();
    thumbnail.save("Assets/Icons/cover_placeholder_thumbnail.png");
}
