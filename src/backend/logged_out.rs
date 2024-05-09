
use color_eyre::Result;
use thirtyfour as tf;

use crate::{
    backend::{self, State},
    common::{browser, helpers::Credentials}
};

#[derive(Debug)]
pub enum Input {
    Login {
        credentials: Credentials,
        browser_driver_config: browser::DriverConfig,
    },
}

impl From<Input> for backend::Input {
    fn from(input: Input) -> Self {
        Self::LoggedOut(input)
    }
}

#[derive(Clone, Debug)]
pub enum Output {
    LoggedIn(String),
    Error(Error),
}

impl From<Error> for Output {
    fn from(error: Error) -> Self {
        Self::Error(error)
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("failed to connect to browser")]
    BrowserConnection(String),
    #[error("")]
    Other(String),
}

impl From<Output> for backend::Output {
    fn from(output: Output) -> Self {
        Self::LoggedOut(output)
    }
}

impl From<LoggedOut> for State {
    fn from(state: LoggedOut) -> Self {
        State::LoggedOut(state)
    }
}

#[derive(Clone, Default)]
pub struct LoggedOut {}

impl LoggedOut {
    pub async fn update(
        self,
        connection: &mut Option<browser::Connection>,
        input: Input,
    ) -> (State, Option<backend::Output>) {
        match input {
            Input::Login {
                credentials,
                browser_driver_config,
            } => {
                if connection.is_none() {
                    match browser::Connection::new(&browser_driver_config).await {
                        Ok(new_connection) => *connection = Some(new_connection),
                        Err(error) => return (
                            self.into(),
                            Some(backend::Output::from(Output::from(Error::BrowserConnection(error.to_string())))),
                        ),
                    }
                }

                if let Some(connection) = connection {
                    match LoggedOut::sign_in(&mut connection.browser).await {
                        Ok(user) => (
                            backend::home::Home::new(user.clone()).into(),
                            Some(backend::Output::from(Output::LoggedIn(user))),
                        ),
                        Err(error) => (
                            self.into(),
                            Some(backend::Output::from(Output::from(error))),
                        ),
                    }
                } else {
                    todo!()
                }
            }
        }
    }

    async fn sign_in(browser: &mut tf::WebDriver) -> Result<String, Error> {
        todo!()
    }
}
