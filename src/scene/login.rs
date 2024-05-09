use strum::IntoEnumIterator;

use crate::{
    backend,
    common::{browser, helpers::Credentials},
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
    BrowserSelected(browser::Browser),
    BrowserHeadlessToggle(bool),
}

#[derive(Clone)]
pub struct Login {
    credentials: Credentials,
    browser_driver_ip_input: String,
    browser_driver_port_input: String,
    browser_headless: bool,
    browser: browser::Browser,
}

impl Default for Login {
    fn default() -> Self {
        let browser_driver_config = browser::DriverConfig {
            driver_address: std::net::SocketAddrV4::new(
                std::net::Ipv4Addr::new(127, 0, 0, 1),
                4444,
            ),
            browser: browser::Browser::Firefox,
            headless: true,
        };

        Self {
            credentials: Credentials {
                email: std::env::var("godric_email").unwrap_or("".to_string()),
                password: std::env::var("godric_password").unwrap_or("".to_string()),
            },
            browser_driver_ip_input: browser_driver_config.driver_address.ip().to_string(),
            browser_driver_port_input: browser_driver_config.driver_address.port().to_string(),
            browser_headless: std::env::var("godric_headless")
                .ok()
                .and_then(|string| string.parse().ok())
                .unwrap_or(true),
            browser: browser::Browser::Firefox,
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
        let mut output = None;
        match message {
            Message::EmailInput(email) => self.credentials.email = email,
            Message::PasswordInput(password) => self.credentials.password = password,
            Message::AttemptLogin => {
                if let Ok(ip) = self.browser_driver_ip_input.parse()
                    && let Ok(port) = self.browser_driver_port_input.parse()
                {
                    output = Some(
                        backend::logged_out::Input::Login {
                            credentials: self.credentials.clone(),
                            browser_driver_config: browser::DriverConfig {
                                browser: self.browser,
                                driver_address: std::net::SocketAddrV4::new(ip, port),
                                headless: self.browser_headless,
                            },
                        }
                        .into(),
                    )
                }
            }
            Message::LoginSuccess => todo!(),
            Message::ServerAddressInput(address) => self.browser_driver_ip_input = address,
            Message::ServerPortInput(port) => self.browser_driver_port_input = port,
            Message::SettingsClick => todo!(),
            Message::BrowserSelected(browser) => self.browser = browser,
            Message::BrowserHeadlessToggle(headless) => self.browser_headless = headless,
        }

        (self.into(), output)
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
            let server_ip_input = {
                let title = iced::widget::text("Server ip");
                let input =
                    iced::widget::TextInput::new("127.0.0.1", &self.browser_driver_ip_input)
                        .on_input(Message::ServerAddressInput)
                        .padding(10);
                iced::widget::column!(title, input)
            };

            let server_port_input = {
                let title = iced::widget::text("Server port");
                let input = iced::widget::TextInput::new("4444", &self.browser_driver_port_input)
                    .on_input(Message::ServerPortInput)
                    .padding(10);
                iced::widget::column!(title, input)
            };

            let browser_headless_control = iced::widget::container(
                iced::widget::checkbox("Headless browser", self.browser_headless)
                    .on_toggle(Message::BrowserHeadlessToggle),
            );

            let browser_selection = {
                iced::widget::container(iced::widget::pick_list(
                    browser::Browser::iter()
                        .map(|browser| browser.to_string())
                        .collect::<Vec<_>>(),
                    Some(self.browser.to_string()),
                    |selection| {
                        Message::BrowserSelected(
                            browser::Browser::try_from(selection.as_str())
                                .expect("Invalid browser selected!"),
                        )
                    },
                ))
            };

            let browser_controls =
                iced::widget::column!(browser_headless_control, browser_selection).spacing(10);

            iced::widget::row!(server_ip_input, server_port_input, browser_controls)
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
