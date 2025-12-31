use tempfile::TempDir;

use crate::common::helpers::dir_is_valid;

#[derive(Clone, Debug)]
pub struct Config {
    root: std::path::PathBuf,
    temporary_root: std::sync::Arc<TempDir>, // TempDir isn't Clone, so we arc it
    use_temporary: bool,
    valid: bool,
    subdirectory_stem: String,
}

impl Config {
    pub fn new() -> std::io::Result<Self> {
        let temporary_root: std::sync::Arc<TempDir> = TempDir::new()?.into();
        let valid = dir_is_valid(temporary_root.path());

        Ok(Self {
            root: "".into(),
            valid,
            temporary_root,
            use_temporary: true,
            subdirectory_stem: "".to_string(),
        })
    }

    pub fn valid(&self) -> bool {
        self.valid
    }

    fn validate(&mut self) {
        self.valid = if self.use_temporary {
            dir_is_valid(self.temporary_root.path())
        } else {
            dir_is_valid(&self.root)
        };
    }

    pub fn root(&self) -> &std::path::PathBuf {
        &self.root
    }

    pub fn temporary_root(&self) -> &TempDir {
        &self.temporary_root
    }

    pub fn active_root(&self) -> &std::path::Path {
        if self.use_temporary {
            self.temporary_root().path()
        } else {
            self.root()
        }
    }

    pub fn with_root(mut self, root: std::path::PathBuf) -> Self {
        self.root = root;
        self.validate();
        self
    }

    pub fn use_temporary(&self) -> bool {
        self.use_temporary
    }

    pub fn set_use_temporary(&mut self, use_temporary: bool) {
        self.use_temporary = use_temporary;
        self.validate();
    }

    pub fn with_use_temporary(mut self, use_temporary: bool) -> Self {
        self.set_use_temporary(use_temporary);
        self
    }

    pub fn get_subdirectory(&self) -> std::io::Result<std::path::PathBuf> {
        let path = self.active_root().join(&self.subdirectory_stem);
        dbg!(&path);
        if !path.exists() {
            std::fs::create_dir(&path)?;
        }
        Ok(path)
    }

    pub fn with_subdirectory(mut self, stem: String) -> Self {
        self.subdirectory_stem = stem;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
            .expect("Failed to initialize cache.")
            .with_root("./Cache".into())
            .with_use_temporary(false)
    }
}
