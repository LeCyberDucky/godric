use strum::IntoEnumIterator;

use crate::{
    backend,
    common::helpers::{Browser, Credentials},
    scene::State,
};

#[derive(Clone, Debug)]
pub enum Message {
    EmailInput(String),
    PasswordInput(String),
    AttemptLogin,
    LoginSuccess,
    ServerAddressInput(String),
    ServerPortInput(String),
    SettingsClick,
    BrowserSelected(Browser),
}

#[derive(Clone)]
pub struct Login {
    credentials: Credentials,
    server_address: String,
    server_port: String,
    browser: Browser,
}

impl Default for Login {
    fn default() -> Self {
        Self {
            credentials: Default::default(),
            server_address: "http://127.0.0.1".to_string(),
            server_port: "4444".to_string(),
            browser: Browser::Firefox,
        }
    }
}

impl From<Login> for State {
    fn from(state: Login) -> Self {
        State::Login(state)
    }
}

impl Login {
    pub fn update(mut self, message: Message) -> (State, Option<backend::Input>) {
        match message {
            Message::EmailInput(email) => self.credentials.email = email,
            Message::PasswordInput(password) => self.credentials.password = password,
            Message::AttemptLogin => todo!(),
            Message::LoginSuccess => todo!(),
            Message::ServerAddressInput(address) => self.server_address = address,
            Message::ServerPortInput(port) => self.server_port = port,
            Message::SettingsClick => todo!(),
            Message::BrowserSelected(browser) => self.browser = browser,
        }

        (self.into(), None)
    }

    pub fn view(&self) -> iced::Element<Message> {
        let image = iced::widget::container(iced::widget::image("Assets/Logo/Welcome.png"))
            .height(iced::Length::Fill)
            .width(iced::Length::Fill)
            .center_x()
            .center_y();

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
                iced::widget::Container::new("Sign in!")
                    .width(iced::Length::Fill)
                    .center_x(),
            )
            .on_press(Message::AttemptLogin)
            .width(iced::Length::Fill),
        )
        .center_x();

        let login_prompt = iced::widget::column!(login_details, login_button)
            .spacing(10)
            .padding(10);

        let browser_settings = {
            let server_address_input = {
                let title = iced::widget::text("Server address");

                let address = &self.server_address;
                let input = iced::widget::TextInput::new("http://127.0.0.1", address)
                    .on_input(Message::ServerAddressInput)
                    .padding(10);
                iced::widget::column!(title, input)
            };

            let server_port_input = {
                let title = iced::widget::text("Server port");
                let input = iced::widget::TextInput::new("4444", &self.server_port)
                    .on_input(Message::ServerPortInput)
                    .padding(10);
                iced::widget::column!(title, input)
            };

            let server_selection = {
                iced::widget::container(iced::widget::pick_list(
                    Browser::iter()
                        .map(|browser| browser.to_string())
                        .collect::<Vec<_>>(),
                    Some(self.browser.to_string()),
                    |selection| {
                        Message::BrowserSelected(
                            Browser::try_from(selection.as_str())
                                .expect("Invalid browser selected!"),
                        )
                    },
                ))
            };

            iced::widget::row!(server_address_input, server_port_input, server_selection)
                .align_items(iced::Alignment::End)
                .spacing(10)
                .padding(10)
        };

        let content = iced::widget::column!(browser_settings, image, login_prompt)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);

        iced::widget::Container::new(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
