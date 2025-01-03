use crate::{common::helpers::Credentials, scene::goodreads::State};

use color_eyre::Result;

#[derive(Clone, Debug)]
pub struct Welcome {
    credentials: Credentials,
}

impl Default for Welcome {
    fn default() -> Self {
        Self {
            credentials: Credentials {
                email: std::env::var("godric_email").unwrap_or("".to_string()),
                password: std::env::var("godric_password").unwrap_or("".to_string()),
            },
        }
    }
}

impl From<Welcome> for State {
    fn from(state: Welcome) -> Self {
        Self::Welcome(state)
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    EmailInput(String),
    PasswordInput(String),
    LoginAttempt,
    LoginSuccess { user_id: String },
}

impl From<crate::backend::goodreads::welcome::Output> for Message {
    fn from(output: crate::backend::goodreads::welcome::Output) -> Self {
        match output {
            crate::backend::goodreads::welcome::Output::LoginSuccess { user_id } => {
                Self::LoginSuccess { user_id }
            }
        }
    }
}

impl TryFrom<crate::scene::goodreads::Message> for Message {
    type Error = crate::backend::Error;

    fn try_from(message: crate::scene::goodreads::Message) -> Result<Self, Self::Error> {
        match message {
            super::Message::Welcome(message) => Ok(message),
            _ => Err(Self::Error::InvalidState {
                state: "Welcome".into(),
                message: format!("{:?}", message),
            }),
        }
    }
}

impl Welcome {
    pub fn update(
        mut self,
        message: Result<Message, crate::backend::Error>,
    ) -> (State, Option<crate::backend::goodreads::Input>) {
        let mut output = None;
        let mut state = None;

        match message {
            Ok(message) => match message {
                Message::EmailInput(email) => self.credentials.email = email,
                Message::PasswordInput(password) => self.credentials.password = password,
                Message::LoginAttempt => {
                    output = Some(crate::backend::goodreads::welcome::Input::LoginAttempt {
                        credentials: self.credentials.clone(),
                    })
                }
                Message::LoginSuccess { user_id } => {
                    state = Some(State::Home(super::home::Home::new(user_id)))
                }
            },
            Err(error) => todo!(),
        }

        (
            state.unwrap_or(self.into()),
            output.map(|output| output.into()),
        )
    }

    pub fn view(&self) -> iced::Element<Message> {
        let image = iced::widget::container(iced::widget::image("Assets/Logo/Welcome.png"))
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill);

        let email_input = {
            let title = iced::widget::text("E-Mail");
            let input = iced::widget::TextInput::new("Godric@example.com", &self.credentials.email)
                .on_input(Message::EmailInput)
                .padding(10);
            iced::widget::column!(title, input)
        };

        let password_input = {
            let title = iced::widget::text("Password");
            let input = iced::widget::TextInput::new("Swordfish", &self.credentials.password)
                .on_input(Message::PasswordInput)
                .secure(true)
                .padding(10);
            iced::widget::column!(title, input)
        };
        let login_details = iced::widget::row!(email_input, password_input).spacing(10);

        let login_button = iced::widget::Container::new(
            iced::widget::Button::new(
                iced::widget::Container::new("Sign in!").center_x(iced::Length::Fill),
            )
            .on_press(Message::LoginAttempt)
            .width(iced::Length::Fill),
        )
        .center_x(iced::Length::Fill);

        let login_prompt = iced::widget::column!(login_details, login_button)
            .spacing(10)
            .padding(10);

        let content = iced::widget::column!(image, login_prompt)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);

        iced::widget::Container::new(content)
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill)
            .into()
    }
}
