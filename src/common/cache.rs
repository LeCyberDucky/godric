use crate::common::helpers::dir_is_valid;
use color_eyre::eyre::Result;
use std::{collections::HashMap, io::Write};
use tempfile::TempDir;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("Io error: {0}")]
    Io(String),
    #[error("Deserialization failed")]
    Deserialization(#[from] ron::de::SpannedError),
    #[error("Serialization failed")]
    Serialization(#[from] ron::Error),
}

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
            .with_root(
                std::env::var("godric_cache_path")
                    .unwrap_or("./Cache".to_string())
                    .into(),
            )
            .with_use_temporary(false)
    }
}

pub trait CacheKey =
    std::fmt::Debug + Eq + std::hash::Hash + serde::Serialize + for<'a> serde::Deserialize<'a>;
pub trait CacheValue = std::fmt::Debug + serde::Serialize + for<'a> serde::Deserialize<'a>;

#[derive(Debug)]
pub struct Cache<K, V> {
    index: HashMap<K, V>,
    directory: std::path::PathBuf,
    index_path: std::path::PathBuf,
}

impl<K, V> Default for Cache<K, V>
where
    K: CacheKey,
    V: CacheValue,
{
    fn default() -> Self {
        Config::default()
            .try_into()
            .expect("Failed to initialize default cache")
    }
}

impl<K, V> TryFrom<Config> for Cache<K, V>
where
    K: CacheKey,
    V: CacheValue,
{
    type Error = Error;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        // Attempt to load index
        // Create index if not available
        // Fail if nothing works
        let directory = config
            .get_subdirectory()
            .map_err(|e| Error::Io(e.to_string()))?;
        let index_path = directory.join("index.ron");
        let index = if let Ok(index_file) = std::fs::File::open(&index_path) {
            ron::de::from_reader(std::io::BufReader::new(&index_file))?
        } else {
            HashMap::new()
        };

        Ok(Self {
            index,
            directory,
            index_path,
        })
    }
}

impl<K, V> Cache<K, V>
where
    K: CacheKey,
    V: CacheValue,
{
    pub fn get(&self, key: &K) -> Option<&V> {
        self.index.get(key)
    }

    /// Inserts an object into the cache and writes the cache to file.
    /// If the cache already contains an entry for the object, the entry is updated.
    pub fn push(&mut self, key: K, value: V) -> Result<(), Error> {
        self.index.insert(key, value);
        let temp_file_path = self.directory.join("index.ron.tmp");
        let temp_file =
            std::fs::File::create(&temp_file_path).map_err(|e| Error::Io(e.to_string()))?;
        let mut writer = std::io::BufWriter::new(temp_file);
        ron::Options::default().to_io_writer_pretty(
            &mut writer,
            &self.index,
            ron::ser::PrettyConfig::default(),
        )?;
        writer.flush().map_err(|e| Error::Io(e.to_string()))?;
        std::fs::rename(&temp_file_path, &self.index_path).map_err(|e| Error::Io(e.to_string()))?;
        Ok(())
    }

    pub fn directory(&self) -> &std::path::Path {
        self.directory.as_path()
    }
}
