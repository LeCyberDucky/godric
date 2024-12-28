use strum::IntoEnumIterator;

use crate::{
    backend,
    common::{browser, helpers::Mode},
    scene::Error,
    scene::State,
};

#[derive(Clone, Debug)]
pub enum Message {
    LaunchAttempt,
    LaunchSuccess(crate::common::helpers::Mode),
    ServerAddressInput(String),
    ServerPortInput(String),
    SettingsClick,
    BrowserSelected(browser::Browser),
    BrowserHeadlessToggle(bool),
    ModeSelected(crate::common::helpers::Mode),
    BackendConnected,
}

impl TryFrom<crate::scene::Message> for Message {
    type Error = crate::backend::Error;

    fn try_from(message: crate::scene::Message) -> Result<Self, Self::Error> {
        match message {
            super::Message::Connected => Ok(Message::BackendConnected),
            super::Message::Launch(message) => Ok(message),
            _ => Err(Error::InvalidState {
                state: "Launch".into(),
                message: format!("{:?}", message),
            }),
        }
    }
}

impl From<crate::backend::uninitialized::Output> for Message {
    fn from(output: crate::backend::uninitialized::Output) -> Self {
        match output {
            backend::uninitialized::Output::Initialized(mode) => Self::LaunchSuccess(mode),
        }
    }
}

impl From<Message> for crate::scene::Message {
    fn from(message: Message) -> Self {
        Self::Launch(message)
    }
}

#[derive(Clone)]
pub struct Launch {
    browser_driver_ip_input: String,
    browser_driver_port_input: String,
    browser_headless: bool,
    browser: browser::Browser,
    mode: Mode,
}

impl Default for Launch {
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
            browser_driver_ip_input: browser_driver_config.driver_address.ip().to_string(),
            browser_driver_port_input: browser_driver_config.driver_address.port().to_string(),
            browser_headless: std::env::var("godric_headless")
                .ok()
                .and_then(|string| string.parse().ok())
                .unwrap_or(true),
            browser: browser::Browser::Firefox,
            mode: Mode::Goodreads,
        }
    }
}

impl From<Launch> for State {
    fn from(state: Launch) -> Self {
        State::Launch(state)
    }
}

impl Launch {
    pub fn update(mut self, message: Result<Message, Error>) -> (State, Option<backend::Input>) {
        let mut output = None;
        let mut state = None;
        match message.expect("Failed to launch.") {
            Message::LaunchAttempt => {
                if let Ok(ip) = self.browser_driver_ip_input.parse()
                    && let Ok(port) = self.browser_driver_port_input.parse()
                {
                    output = Some(
                        backend::uninitialized::Input::Launch {
                            browser_driver_config: browser::DriverConfig {
                                browser: self.browser,
                                driver_address: std::net::SocketAddrV4::new(ip, port),
                                headless: self.browser_headless,
                            },
                            mode: self.mode,
                        }
                        .into(),
                    )
                }
            }
            Message::LaunchSuccess(mode) => {
                state =
                    State::Goodreads(crate::scene::goodreads::welcome::Welcome::default().into())
                        .into()
            }

            Message::ServerAddressInput(address) => self.browser_driver_ip_input = address,
            Message::ServerPortInput(port) => self.browser_driver_port_input = port,
            Message::SettingsClick => todo!(),
            Message::BrowserSelected(browser) => self.browser = browser,
            Message::BrowserHeadlessToggle(headless) => self.browser_headless = headless,
            Message::ModeSelected(mode) => self.mode = mode,
            Message::BackendConnected => println!("Backend connected!"),
        };

        (state.unwrap_or(self.into()), output)
    }

    pub fn view(&self) -> iced::Element<Message> {
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
                .align_y(iced::Alignment::End)
                .spacing(10)
                .padding(10)
        };

        let image = iced::widget::container(iced::widget::image("Assets/Logo/Welcome.png"))
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill);

        let mode_selection = iced::widget::pick_list(
            crate::common::helpers::Mode::iter()
                .map(|mode| mode.to_string())
                .collect::<Vec<_>>(),
            Some(self.mode.to_string()),
            |selection| {
                Message::ModeSelected(
                    crate::common::helpers::Mode::try_from(selection.as_str())
                        .expect("Invalid mode selected!"),
                )
            },
        );

        let launch_button = iced::widget::Button::new(
            iced::widget::Container::new("Launch!")
                .center_x(iced::Length::Fill),
        )
        .on_press(Message::LaunchAttempt)
        .width(iced::Length::Fill);

        let launch_prompt = iced::widget::column!(mode_selection, launch_button)
            .spacing(10)
            .padding(10)
            .align_x(iced::Alignment::Center);

        let content = iced::widget::column!(browser_settings, image, launch_prompt)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);

        iced::widget::Container::new(content)
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill)
            .into()
    }
}
