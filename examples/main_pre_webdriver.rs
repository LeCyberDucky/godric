use std::fs::read_to_string;

use color_eyre::eyre::{bail, eyre, ContextCompat, Ok, Result};
use scraper::{Html, Selector};

fn main() -> Result<()> {
    let url = url::Url::parse("https://www.goodreads.com/user/sign_in")?;
    let client = reqwest::blocking::ClientBuilder::new()
        .cookie_store(true)
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:124.0) Gecko/20100101 Firefox/124.0",
        )
        .build()?;
    let response = client.get(url).send()?;
    let page = response.text()?;
    // std::fs::write("Data/signin_start.html", response.text()?);

    // let page = std::fs::read_to_string("Data/signin_start.html")?;
    let page = Html::parse_document(&page);

    let button = &Selector::parse("div.third_party_sign_in").unwrap();
    // let result: Vec<_> = page.select(button).flat_map(|element| element.text()).collect();
    let result = page.select(button).next().unwrap();
    // let results: Vec<_> = result.children().map(|child| child).collect();
    // for dings in results {
    //     if let Some(element) = dings.value().as_element() {
    //         println!("{:#?}", element.attr("href"));
    //     }
    //     println!();
    //     println!();
    //     println!();
    // }

    let result = result.children().nth(7).unwrap();
    let url = url::Url::parse(result.value().as_element().unwrap().attr("href").unwrap())?;

    println!("Sign in URL:");
    println!("{}", url);
    println!();
    println!();
    println!();

    client.get(url.clone()).send()?;
    // let response = client.get(url).send()?;
    // println!("{:#?}", response);

    println!("Ayy! Logging in!");
    dotenv::dotenv()?;

    let email = std::env::var("email")?;
    let password = std::env::var("password")?;
    // let form_data = [("user[email]", email), ("user[password]", password)];
    let form_data = [("ap_email", email), ("ap_password", password)];
    let login_request = client.post(url.clone()).form(&form_data).build()?;
    println!("Sign in request:");
    println!("{:#?}", login_request);
    println!();
    println!();
    println!();
    let response = client.post(url).form(&form_data).send()?;
    // let response = client.post(url).basic_auth(email, Some(password)).send()?;
    println!();
    println!();
    println!();
    println!("{:#?}", response);
    std::fs::write("Data/login_result.html", response.text()?);
    println!();
    println!();
    println!();

    println!("Navigating to settings!");
    let url = url::Url::parse("https://www.goodreads.com/user/edit?ref=nav_profile_settings")?;
    let response = client.get(url).send()?.text()?;
    println!();
    println!();
    println!();
    println!("{}", response);

    // println!("Getting books!");
    // let url = url::Url::parse(r"https://www.goodreads.com/review/list/131573995-andy?ref=nav_mybooks&shelf=to-read")?;
    // let response = client.get(url).basic_auth(email, Some(password)).send()?.text()?;
    // println!();
    // println!();
    // println!();
    // println!("{}", response);

    // let result = page.select(button).flat_map(|element| element.child_elements().attr("href"));//.flat_map(|element| element.text()).next();
    // println!("{:#?}", result);

    // /html/body/div[1]/div[1]/div[2]/div/div/div/div[1]/div/a[4]

    // .third_party_sign_in > a:nth-child(4)

    // html.picture.es5array.es5date.es5function.es5object.strictmode.es5string.json.es5syntax.es5undefined.es5.no-touchevents.cssanimations.flexbox.flexwrap.csstransforms.localstorage body.textured div.wrapper div.content.distractionless div#topLanding.mainContentContainer div.mainContent div.contentBox.clearfix div.column_right div#choices div.third_party_sign_in a

    // println!("{:#?}", response);

    // Sign in to GoodReads
    // https://stackoverflow.com/questions/68697683/struggling-to-log-in-to-goodreads-using-requests-session-what-is-missing-in-my
    // let url = url::Url::parse("https://www.goodreads.com/user/sign_in?source=home")?;
    // let client = reqwest::blocking::ClientBuilder::new().cookie_store(true).build()?;
    // let response = client.post(url).basic_auth(email, Some(password)).send()?;

    // println!("{:#?}", response);
    // println!("{:#?}", client);

    // let url = url::Url::parse(r"https://www.goodreads.com/review/list/131573995?ref=nav_mybooks&shelf=to-read")?;
    // let response = reqwest::get(r"https://www.goodreads.com/review/list/131573995?ref=nav_mybooks&shelf=to-read").await?;

    // let client = reqwest::Client::new();
    // let response = client.get(url).basic_auth(email, Some(password)).header("key", value).send().await?;

    // reqwest::ClientBuilder::new().

    // std::fs::write("Data/books.html", response.text().await?);

    // println!("{}", response.text().await?);

    Ok(())
}
