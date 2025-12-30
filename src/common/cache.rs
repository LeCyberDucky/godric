use tempfile::TempDir;

use crate::common::helpers::dir_is_valid;

#[derive(Clone, Debug)]
pub struct Config {
    path: std::path::PathBuf,
    temporary_path: std::sync::Arc<TempDir>, // TempDir isn't Clone, so we arc it
    use_temporary: bool,
    valid: bool,
    subdirectory: String
}

impl Config {
    pub fn new() -> Self {
        let temporary_path: std::sync::Arc<TempDir> = TempDir::new()
                .expect("Failed to create temporary directory to store book covers")
                .into();
        let valid = dir_is_valid(temporary_path.path());

        Self { path: "".into(), valid, temporary_path, use_temporary: true, subdirectory: "".to_string() }
    }

    pub fn valid(&self) -> bool {
        self.valid
    }

    fn validate(&mut self) {
        self.valid = if self.use_temporary {
            dir_is_valid(self.temporary_path.path())
        } else {
            dir_is_valid(&self.path)
        };
    }

    pub fn path(&self) -> &std::path::PathBuf {
        &self.path
    }
    
    pub fn temporary_path(&self) -> &TempDir {
        &self.temporary_path
    }

    pub fn active_path(&self) -> &std::path::Path {
        if self.use_temporary {
            self.temporary_path().path()
        } else {
            self.path()
        }
    }

    pub fn with_path(mut self, path: std::path::PathBuf) -> Self {
        self.path = path;
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

    fn create_subdirectory(&self) -> std::io::Result<()> {
        let path = self.active_path().join(&self.subdirectory);
        if !path.exists() {
            std::fs::create_dir(path)?;
        }
        Ok(())
    }

    pub fn with_sub_directory(mut self, subdirectory: String) -> Self {
        self.subdirectory = subdirectory;
        self.create_subdirectory();
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new().with_path("./Cache".into()).with_use_temporary(false)
    }
}
