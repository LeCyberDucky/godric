use crate::{
    backend::{self, goodreads::State},
    common::helpers::Credentials,
};
use color_eyre::{eyre::Context, eyre::ContextCompat, Result};
use thirtyfour as tf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid message ({message}) for state {state}")]
    InvalidState { state: String, message: String },
    #[error(transparent)]
    Other(#[from] color_eyre::Report),
}

#[derive(Clone, Debug)]
pub enum Input {
    LoginAttempt { credentials: Credentials },
}

impl From<Input> for backend::goodreads::Input {
    fn from(input: Input) -> Self {
        Self::Welcome(input)
    }
}

impl TryFrom<backend::goodreads::Input> for Input {
    type Error = Error;

    fn try_from(input: backend::goodreads::Input) -> Result<Self, Self::Error> {
        match input {
            super::Input::Welcome(input) => Ok(input),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Output {
    LoginSuccess { user_id: String },
}

impl From<Output> for backend::goodreads::Output {
    fn from(output: Output) -> Self {
        Self::Welcome(output)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Welcome {}

impl From<Welcome> for State {
    fn from(state: Welcome) -> Self {
        Self::Welcome(state)
    }
}

impl Welcome {
    pub async fn update(
        self,
        browser: &mut tf::WebDriver,
        input: Input,
    ) -> Result<(State, Option<Output>), Error> {
        let Input::LoginAttempt { credentials } = input;
        let user_id = sign_in_to_goodreads(browser, &credentials).await?;

        Ok((self.into(), Some(Output::LoginSuccess { user_id })))
    }
}

/// Signs in to goodreads.com, returning the user ID-string
async fn sign_in_to_goodreads(
    browser: &mut tf::WebDriver,
    credentials: &Credentials,
) -> Result<String, Error> {
    let url = url::Url::parse("https://www.goodreads.com/user/sign_in")
        .expect("Failed to parse URL for Goodreads sign in page");
    browser
        .goto(url.as_str())
        .await
        .context("Failed to navigate to sign in page")?;

    let email_signin_button = browser.find(tf::By::ClassName("gr-button.gr-button--dark.gr-button--auth.authPortalConnectButton.authPortalSignInButton")).await.context("Failed to find e-mail sign in button")?;
    email_signin_button
        .click()
        .await
        .context("Failed to click e-mail sign in button")?;

    let email_field = browser
        .find(tf::By::Id("ap_email"))
        .await
        .context("E-mail field not available")?;
    let password_field = browser
        .find(tf::By::Id("ap_password"))
        .await
        .context("Password field not available")?;
    let signin_button = browser
        .find(tf::By::Id("signInSubmit"))
        .await
        .context("Sign in button not available")?;

    email_field
        .send_keys(&credentials.email)
        .await
        .context("Unable to enter e-mail")?;
    password_field
        .send_keys(&credentials.password)
        .await
        .context("Unable to enter password")?;
    signin_button
        .click()
        .await
        .context("Unable to click sign in button")?;

    // Find user ID and construct link to "want to read" list
    // https://www.goodreads.com/user/show/176878294-testy-mctestface
    let profile_button = browser
        .find(tf::By::ClassName(
            "dropdown__trigger.dropdown__trigger--profileMenu.dropdown__trigger--personalNav",
        ))
        .await
        .context("Failed to find user profile button")?;
    let user = profile_button
        .attr("href")
        .await
        .context("Unable to find user ID")?
        .context("Unable to find user ID")?
        .split('/')
        .last()
        .context("Unable to parse user ID")?
        .to_owned();
    let user_id = user
        .split('-')
        .next()
        .context("Unable to extract user ID number")?
        .to_owned();
    Ok(user_id)
}
