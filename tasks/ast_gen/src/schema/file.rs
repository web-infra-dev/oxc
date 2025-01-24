/// Represents a Rust source file.
#[derive(Debug)]
pub struct File {
    pub file_path: String,
    pub import_path: String,
}

impl File {
    #[expect(dead_code)]
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    pub fn import_path(&self) -> &str {
        &self.import_path
    }

    pub fn krate(&self) -> &str {
        self.import_path.split("::").next().unwrap()
    }
}
