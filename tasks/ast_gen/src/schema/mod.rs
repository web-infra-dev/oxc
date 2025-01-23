mod defs;
mod derives;
mod layout;
pub use defs::*;
pub use derives::{DeriveId, Derives};
pub use layout::*;

pub type FileId = usize;
pub type TypeId = usize;

#[derive(Debug)]
pub struct Schema {
    pub defs: Vec<TypeDef>,
    pub files: Vec<File>,
}

impl Schema {
    pub fn def(&self, type_id: TypeId) -> &TypeDef {
        &self.defs[type_id]
    }

    pub fn def_struct(&self, type_id: TypeId) -> &StructDef {
        self.defs[type_id].as_struct().unwrap()
    }

    pub fn def_enum(&self, type_id: TypeId) -> &EnumDef {
        self.defs[type_id].as_enum().unwrap()
    }

    pub fn def_mut(&mut self, type_id: TypeId) -> &mut TypeDef {
        &mut self.defs[type_id]
    }

    pub fn def_struct_mut(&mut self, type_id: TypeId) -> &mut StructDef {
        self.defs[type_id].as_struct_mut().unwrap()
    }

    #[expect(dead_code)]
    pub fn def_enum_mut(&mut self, type_id: TypeId) -> &mut EnumDef {
        self.defs[type_id].as_enum_mut().unwrap()
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
