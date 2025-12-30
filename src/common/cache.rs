#[derive(Clone, Debug)]
pub struct Config {
    path: std::path::PathBuf,
    valid: bool,
}

impl Config {
    pub fn new(path: std::path::PathBuf) -> Self {
        let valid = path.is_dir()
            && std::fs::metadata(&path).is_ok_and(|meta| !meta.permissions().readonly());

        Self {
            path,
            valid,
        }
    }

    pub fn valid(&self) -> bool {
        self.valid
    }

    pub fn path(&self) -> &std::path::PathBuf {
        &self.path
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new("./Cache".into())
    }
}
