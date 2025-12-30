#[derive(Clone, Debug)]
pub struct Config {
    pub path: std::path::PathBuf
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: "./Cache".into()
        }
    }
}
