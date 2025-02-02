use std::io::{BufRead, Read};

use color_eyre::eyre::{ContextCompat, Ok, Result, bail, eyre};
use thirtyfour as tf;
use thirtyfour::prelude::ElementQueryable;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;

    let email = std::env::var("godric_email")?;
    let password = std::env::var("godric_password")?;

    // Set up browser
    let driver_address = std::net::SocketAddrV4::new(std::net::Ipv4Addr::new(127, 0, 0, 1), 4444);

    let mut driver = std::process::Command::new("geckodriver")
        .args([
            "--host",
            &driver_address.ip().to_string(),
            "--port",
            &driver_address.port().to_string(),
        ])
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let driver_address =
        url::Url::parse(&("http://".to_string() + driver_address.to_string().as_str()))?;

    // let driver_address = {
    //     let stdout = driver.stdout.context("Unable to read output of spawning driver!")?;
    //     let mut output = String::new();
    //     std::io::BufReader::new(stdout).read_line(&mut output)?;
    //     let socket_address: std::net::SocketAddrV4 = output.split_whitespace().last().context("Unable to extract address of the WebDriver. Ensure that geckodriver is available on the system and not yet running.")?.parse()?;
    //     url::Url::parse(&("http://".to_string() + socket_address.to_string().as_str()))?
    // };

    let mut browser_settings = tf::DesiredCapabilities::firefox();
    // browser_settings.set_headless()?;
    let browser = tf::WebDriver::new(
        driver_address.as_str(),
        /*"http://localhost:4444",*/ browser_settings,
    )
    .await?;

    // Sign in
    let url = url::Url::parse("https://www.goodreads.com/user/sign_in")?;
    browser.goto(url).await?;

    let email_signin_button = browser.find(tf::By::ClassName("gr-button.gr-button--dark.gr-button--auth.authPortalConnectButton.authPortalSignInButton")).await?;
    email_signin_button.click().await?;

    let email_field = browser.find(tf::By::Id("ap_email")).await?;
    let password_field = browser.find(tf::By::Id("ap_password")).await?;
    let signin_button = browser.find(tf::By::Id("signInSubmit")).await?;

    email_field.send_keys(email).await?;
    password_field.send_keys(password).await?;
    signin_button.click().await?;

    // Find user ID and construct link to "want to read" list
    // https://www.goodreads.com/user/show/176878294-testy-mctestface
    let profile_button = browser
        .find(tf::By::ClassName(
            "dropdown__trigger.dropdown__trigger--profileMenu.dropdown__trigger--personalNav",
        ))
        .await?;
    let user = profile_button
        .attr("href")
        .await?
        .context("Unable to find user ID.")?
        .split('/')
        .last()
        .context("Unable to parse user ID.")?
        .to_owned();
    dbg!(&user);
    let user_id = user
        .split('-')
        .next()
        .context("Unable to extract user ID number.")?
        .to_owned();

    let bookshelf_link = "https://www.goodreads.com/review/list/".to_owned() + user.as_str();
    let bookshelf_link = url::Url::parse_with_params(
        &bookshelf_link,
        &[("shelf", "to-read"), ("sort", "position")],
    )?;

    browser.goto(bookshelf_link).await?;

    // Obtain list of books
    let mut books = browser
        .find_all(tf::By::ClassName("bookalike.review"))
        .await?;
    let mut book_count = 0;

    while book_count != books.len() {
        book_count = books.len();
        books
            .last()
            .context("No books found!")?
            .scroll_into_view()
            .await?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        books = browser
            .find_all(tf::By::ClassName("bookalike.review"))
            .await?;
    }

    std::thread::sleep(std::time::Duration::from_secs(5));
    let book = &books[0];
    let book_controls = book.find(tf::By::ClassName("reorderControls")).await?;
    let position_field = book_controls.find(tf::By::Tag("input")).await?;
    dbg!(position_field.value().await?);
    // position_field.value() = 5;
    // dbg!(position_field.value());
    dbg!(book.text().await?);
    // position_field.focus().await?;
    // position_field.scroll_into_view().await?;
    position_field.focus().await?;
    position_field
        .send_keys(tf::Key::Control + "a".to_string())
        .await?;
    position_field
        .send_keys(tf::Key::Delete.to_string())
        .await?;
    position_field.send_keys("5").await?;
    // position_field.click().await?;
    browser
        .execute(&format!("savePositionChanges({});", user_id), vec![])
        .await?;

    // let mut book_texts = vec![];
    // for book in books {
    //     book_texts.push(book.text().await?);
    // }

    // println!("{:#?}", book_texts);

    std::thread::sleep(std::time::Duration::from_secs(10));

    browser.quit().await?;

    // let elem_form = driver.find(tf::By::Id("search-form")).await?;

    // // Find element from element.
    // let elem_text = elem_form.find(tf::By::Id("searchInput")).await?;

    // // Type in the search terms.
    // elem_text.send_keys("selenium").await?;

    // // Click the search button.
    // let elem_button = elem_form.find(tf::By::Css("button[type='submit']")).await?;
    // elem_button.click().await?;

    // // Look for header to implicitly wait for the page to load.
    // driver.find(tf::By::ClassName("firstHeading")).await?;
    // assert_eq!(driver.title().await?, "Selenium - Wikipedia");

    // // Always explicitly close the browser.
    // driver.quit().await?;

    // // let button = &Selector::parse("div.third_party_sign_in").unwrap();

    // let url = url::Url::parse("https://www.goodreads.com/user/edit?ref=nav_profile_settings")?;

    Ok(())
}
