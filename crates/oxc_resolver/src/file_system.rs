use std::{fs, io, path::Path};

pub trait FileSystem: Default + Send + Sync {
    /// # Errors
    ///
    /// * Any [io::Error]
    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> io::Result<String>;

    /// # Errors
    ///
    /// This function will return an error in the following situations, but is not
    /// limited to just these cases:
    ///
    /// * The user lacks permissions to perform `metadata` call on `path`.
    /// * `path` does not exist.
    fn symlink_metadata<P: AsRef<Path>>(&self, path: P) -> io::Result<FileMetadata>;
}

#[derive(Debug, Clone, Copy)]
pub struct FileMetadata {
    pub(crate) is_file: bool,
}

impl FileMetadata {
    pub fn new(is_file: bool) -> Self {
        Self { is_file }
    }
}

/// Operating System
#[derive(Default)]
pub struct FileSystemOs;

impl FileSystem for FileSystemOs {
    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> io::Result<String> {
        fs::read_to_string(path)
    }

    fn symlink_metadata<P: AsRef<Path>>(&self, path: P) -> io::Result<FileMetadata> {
        fs::symlink_metadata(path).map(|metadata| FileMetadata { is_file: metadata.is_file() })
    }
}
