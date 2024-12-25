use color_eyre::Result;
use thirtyfour as tf;

use crate::{
    backend::{self, State},
    common::{browser, helpers::Mode},
};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("failed to connect to browser")]
    BrowserConnection(String),
    #[error("")]
    Other(String),
}

impl From<Error> for backend::Error {
    fn from(error: Error) -> Self {
        Self::Uninitialized(error)
    }
}

#[derive(Clone, Debug)]
pub enum Input {
    Launch {
        browser_driver_config: browser::DriverConfig,
        mode: Mode,
    },
}

impl From<Input> for backend::Input {
    fn from(input: Input) -> Self {
        Self::Uninitialized(input)
    }
}

impl TryFrom<backend::Input> for Input {
    type Error = backend::Error;

    fn try_from(input: backend::Input) -> Result<Self, Self::Error> {
        match input {
            backend::Input::Uninitialized(input) => Ok(input),
            _ => Err(backend::Error::InvalidState {
                state: "Uninitialized".to_string(),
                message: format!("{:?}", input),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Output {
    Initialized(Mode),
}

impl From<Output> for backend::Output {
    fn from(output: Output) -> Self {
        Self::Uninitialized(output)
    }
}

impl From<Uninitialized> for State {
    fn from(state: Uninitialized) -> Self {
        State::Uninitialized(state)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Uninitialized {}

impl Uninitialized {
    pub async fn update(
        self,
        connection: &mut Option<browser::Connection>,
        input: Input,
    ) -> Result<(State, Option<Output>), Error> {
        match input {
            Input::Launch {
                browser_driver_config,
                mode,
            } => {
                if connection.is_none() {
                    match browser::Connection::new(&browser_driver_config).await {
                        Ok(new_connection) => *connection = Some(new_connection),
                        Err(error) => return Err(Error::BrowserConnection(error.to_string())),
                    }
                }

                match mode {
                    Mode::Goodreads => Ok((
                        State::Goodreads(backend::goodreads::welcome::Welcome::default().into()),
                        Some(Output::Initialized(mode.into()).into()),
                    )),
                }
            }
        }
    }

    async fn sign_in_to_goodreads(browser: &mut tf::WebDriver) -> Result<String, Error> {
        //     let url = url::Url::parse("https://www.goodreads.com/user/sign_in")?;
        //     browser.goto(url).await?;

        //     let email_signin_button = browser.find(tf::By::ClassName("gr-button.gr-button--dark.gr-button--auth.authPortalConnectButton.authPortalSignInButton")).await?;
        // email_signin_button.click().await?;

        // let email_field = browser.find(tf::By::Id("ap_email")).await?;
        // let password_field = browser.find(tf::By::Id("ap_password")).await?;
        // let signin_button = browser.find(tf::By::Id("signInSubmit")).await?;

        // email_field.send_keys(email).await?;
        // password_field.send_keys(password).await?;
        // signin_button.click().await?;

        // // Find user ID and construct link to "want to read" list
        // // https://www.goodreads.com/user/show/176878294-testy-mctestface
        // let profile_button = browser.find(tf::By::ClassName("dropdown__trigger.dropdown__trigger--profileMenu.dropdown__trigger--personalNav")).await?;
        // let user = profile_button.attr("href").await?.context("Unable to find user ID.")?.split('/').last().context("Unable to parse user ID.")?.to_owned();
        // let user_id = user.split('-').next().context("Unable to extract user ID number.")?.to_owned();
        // user_id
        todo!()
    }
}
