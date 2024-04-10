use iced::executor;
use iced::{Application, Command, Element, Settings, Theme};

pub trait Scene {
    type Message;
    type Theme;
    fn update(&mut self, message: Self::Message) -> Command<crate::common::gui::Message>;
    fn view(&self) -> Element<'_, crate::common::gui::Message, Self::Theme, iced::Renderer>;
}

pub mod login {
    use super::*;

    #[derive(Clone, Debug)]
    pub enum Message {
        EmailInput(String),
        PasswordInput(String),
        Login,
        ServerAddressInput(String),
        ServerPortInput(String),
    }
    
        impl Into<crate::common::gui::Message> for Message {
            fn into(self) -> crate::common::gui::Message {
                crate::common::gui::Message::LoginScene(self)
            }
        }

    #[derive(Default)]
    pub struct State {
        email: String,
        password: String,
        server_address: String,
        server_port: String,
    }

    impl super::Scene for State {
        type Message = Message;
        type Theme = Theme;

        fn update(&mut self, message: Self::Message) -> iced::Command<crate::common::gui::Message> {
            match message {
                Message::EmailInput(email) => self.email = email,
                Message::PasswordInput(password) => self.password = password,
                Message::Login => println!("Signing in!"),
                Message::ServerAddressInput(address) => self.server_address = address,
                Message::ServerPortInput(port) => self.server_port = port,
            }

            Command::none()
        }

        fn view(
            &self,
        ) -> iced::Element<'_, crate::common::gui::Message, Self::Theme, iced::Renderer> {
            let email_input = {
                let title = iced::widget::text("E-Mail");
                let input = iced::widget::TextInput::new("Godric@example.com", &self.email)
                    .on_input(|input| Message::EmailInput(input).into())
                    .padding(10);
                iced::widget::column!(title, input)
            };

            let password_input = {
                let title = iced::widget::text("Password");
                let input = iced::widget::TextInput::new("Swordfish", &self.password)
                    .on_input(|input| Message::PasswordInput(input).into())
                    .secure(true)
                    .padding(10);
                iced::widget::column!(title, input)
            };

            let login_details = iced::widget::row!(email_input, password_input)
                .spacing(10)
                .padding([0, 10]);

            let login_button = iced::widget::Container::new(
                iced::widget::Button::new(
                    iced::widget::Container::new("Sign in!")
                        .width(iced::Length::Fill)
                        .center_x(),
                )
                .on_press(Message::Login.into())
                .width(iced::Length::Fill),
            )
            .padding([0, 10])
            .center_x();

            let login_prompt = iced::widget::column!(login_details, login_button)
                .spacing(10)
                .padding(10);

            let settings = {
                let server_address_input = {
                    let title = iced::widget::text("Server address");
                    // let address = "http://".to_owned() + &self.server_address;
                    let address = &self.server_address;
                    let input = iced::widget::TextInput::new("127.0.0.1", &address)
                        .on_input(|input| Message::ServerAddressInput(input).into())
                        .padding(10);
                    iced::widget::row!(title, input).spacing(10).padding(10)
                };

                let server_port_input = {
                    let title = iced::widget::text("Server port");
                    let input = iced::widget::TextInput::new("4444", &self.server_port)
                        .on_input(|input| Message::ServerPortInput(input).into())
                        .padding(10);
                    iced::widget::row!(title, input).spacing(10).padding(10)
                };

                iced::widget::column!(server_address_input, server_port_input)
            };

            let content = iced::widget::column!(login_prompt, settings);

            iced::widget::Container::new(content)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center_x()
                .center_y()
                .into()
        }
    }
}

pub mod home {
    use super::*;

    #[derive(Clone, Debug)]
    pub enum Message {

    }

    impl Into<crate::common::gui::Message> for Message {
        fn into(self) -> crate::common::gui::Message {
            crate::common::gui::Message::Home(self)
        }
    }

    #[derive(Default)]
    struct State {
        current_book: usize
    }

    impl super::Scene for State {
        type Message = Message;
    
        type Theme = Theme;
    
        fn update(&mut self, message: Self::Message) -> Command<crate::common::gui::Message> {
            todo!()
        }
    
        fn view(&self) -> Element<'_, crate::common::gui::Message, Self::Theme, iced::Renderer> {
            todo!()
        }
    }
}