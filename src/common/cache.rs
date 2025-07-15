#[derive(Clone, Debug)]
pub struct Config {
    pub path: std::path::PathBuf,
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: "./Cache".into(),
            enabled: true,
        }
    }
}
