#[derive(Debug, Default, Clone)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, strum_macros::Display, strum_macros::EnumString, strum_macros::EnumIter)]
pub enum Browser {
    Chrome,
    Chromium,
    Edge,
    Firefox,
    #[strum(to_string = "Internet Explorer")]
    InternetExplorer,
    Opera,
    Safari,
}
