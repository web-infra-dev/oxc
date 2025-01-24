use oxc_index::IndexVec;

mod defs;
mod derives;
mod ids;
mod layout;
pub use defs::*;
pub use derives::{DeriveId, Derives};
pub use ids::*;
pub use layout::*;

#[derive(Debug)]
pub struct Schema {
    pub defs: IndexVec<TypeId, TypeDef>,
    pub files: IndexVec<FileId, File>,
}

impl Schema {
    pub fn def(&self, type_id: TypeId) -> &TypeDef {
        &self.defs[type_id]
    }

    #[expect(dead_code)]
    pub fn def_mut(&mut self, type_id: TypeId) -> &mut TypeDef {
        &mut self.defs[type_id]
    }

    /// Get iterator over all `TypeId`s.
    pub fn type_ids(&self) -> IdIter<TypeId> {
        IdIter::new(&self.defs)
    }

    /// Get iterator over all `FileId`s.
    #[expect(dead_code)]
    pub fn file_ids(&self) -> IdIter<FileId> {
        IdIter::new(&self.files)
    }

    pub fn def_struct(&self, type_id: TypeId) -> &StructDef {
        self.defs[type_id].as_struct().unwrap()
    }

    pub fn def_struct_mut(&mut self, type_id: TypeId) -> &mut StructDef {
        self.defs[type_id].as_struct_mut().unwrap()
    }

    pub fn def_enum(&self, type_id: TypeId) -> &EnumDef {
        self.defs[type_id].as_enum().unwrap()
    }

    pub fn def_enum_mut(&mut self, type_id: TypeId) -> &mut EnumDef {
        self.defs[type_id].as_enum_mut().unwrap()
    }

    #[expect(dead_code)]
    pub fn def_primitive(&self, type_id: TypeId) -> &PrimitiveDef {
        self.defs[type_id].as_primitive().unwrap()
    }

    pub fn def_primitive_mut(&mut self, type_id: TypeId) -> &mut PrimitiveDef {
        self.defs[type_id].as_primitive_mut().unwrap()
    }

    pub fn def_option(&self, type_id: TypeId) -> &OptionDef {
        self.defs[type_id].as_option().unwrap()
    }

    pub fn def_option_mut(&mut self, type_id: TypeId) -> &mut OptionDef {
        self.defs[type_id].as_option_mut().unwrap()
    }

    #[expect(dead_code)]
    pub fn def_box(&self, type_id: TypeId) -> &BoxDef {
        self.defs[type_id].as_box().unwrap()
    }

    pub fn def_box_mut(&mut self, type_id: TypeId) -> &mut BoxDef {
        self.defs[type_id].as_box_mut().unwrap()
    }

    #[expect(dead_code)]
    pub fn def_vec(&self, type_id: TypeId) -> &VecDef {
        self.defs[type_id].as_vec().unwrap()
    }

    pub fn def_vec_mut(&mut self, type_id: TypeId) -> &mut VecDef {
        self.defs[type_id].as_vec_mut().unwrap()
    }

    pub fn def_cell(&self, type_id: TypeId) -> &CellDef {
        self.defs[type_id].as_cell().unwrap()
    }

    pub fn def_cell_mut(&mut self, type_id: TypeId) -> &mut CellDef {
        self.defs[type_id].as_cell_mut().unwrap()
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
