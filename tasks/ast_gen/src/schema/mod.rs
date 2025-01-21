mod defs;
mod derives;
pub use defs::*;
pub use derives::{DeriveId, Derives};

pub type FileId = usize;
pub type TypeId = usize;

#[derive(Debug)]
pub struct Schema {
    pub defs: Vec<TypeDef>,
    pub files: Vec<File>,
}

impl Schema {
    #[expect(dead_code)]
    pub fn def(&self, type_id: TypeId) -> &TypeDef {
        &self.defs[type_id]
    }

    pub fn file(&self, file_id: FileId) -> &File {
        &self.files[file_id]
    }
}

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
