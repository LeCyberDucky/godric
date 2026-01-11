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
    Steam,
}

pub fn dir_is_valid<T: AsRef<std::path::Path>>(path: T) -> bool {
    let path = path.as_ref();
    path.is_dir() && std::fs::metadata(path).is_ok_and(|meta| !meta.permissions().readonly())
}
