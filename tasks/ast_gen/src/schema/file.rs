/// Represents a Rust source file.
#[derive(Debug)]
pub struct File {
    /// File path e.g. `crates/oxc_ast/src/ast/js.rs`
    pub file_path: String,
    /// Import path e.g. `oxc_ast::ast::js`
    pub import_path: String,
}

impl File {
    /// Get file path for this [`File`].
    #[expect(dead_code)]
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Get import path for this [`File`].
    pub fn import_path(&self) -> &str {
        &self.import_path
    }

    /// Get name of crate this [`File`] is in.
    pub fn krate(&self) -> &str {
        self.import_path.split("::").next().unwrap()
    }
}
