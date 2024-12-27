#[derive(Debug, Default, Clone)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

#[derive(
    Clone, Copy, Debug, strum_macros::Display, strum_macros::EnumString, strum_macros::EnumIter,
)]
pub enum Mode {
    Goodreads,
}
