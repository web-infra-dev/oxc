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
    pub types: IndexVec<TypeId, TypeDef>,
    pub files: IndexVec<FileId, File>,
}

impl Schema {
    pub fn type_def(&self, type_id: TypeId) -> &TypeDef {
        &self.types[type_id]
    }

    pub fn type_def_mut(&mut self, type_id: TypeId) -> &mut TypeDef {
        &mut self.types[type_id]
    }

    /// Get iterator over all `TypeId`s.
    pub fn type_ids(&self) -> IdIter<TypeId> {
        IdIter::new(&self.types)
    }

    /// Get iterator over all `FileId`s.
    #[expect(dead_code)]
    pub fn file_ids(&self) -> IdIter<FileId> {
        IdIter::new(&self.files)
    }

    pub fn struct_def(&self, type_id: TypeId) -> &StructDef {
        self.type_def(type_id).as_struct().unwrap()
    }

    pub fn struct_def_mut(&mut self, type_id: TypeId) -> &mut StructDef {
        self.type_def_mut(type_id).as_struct_mut().unwrap()
    }

    pub fn enum_def(&self, type_id: TypeId) -> &EnumDef {
        self.type_def(type_id).as_enum().unwrap()
    }

    pub fn enum_def_mut(&mut self, type_id: TypeId) -> &mut EnumDef {
        self.type_def_mut(type_id).as_enum_mut().unwrap()
    }

    #[expect(dead_code)]
    pub fn primitive_def(&self, type_id: TypeId) -> &PrimitiveDef {
        self.type_def(type_id).as_primitive().unwrap()
    }

    pub fn primitive_def_mut(&mut self, type_id: TypeId) -> &mut PrimitiveDef {
        self.type_def_mut(type_id).as_primitive_mut().unwrap()
    }

    pub fn option_def(&self, type_id: TypeId) -> &OptionDef {
        self.type_def(type_id).as_option().unwrap()
    }

    pub fn option_def_mut(&mut self, type_id: TypeId) -> &mut OptionDef {
        self.type_def_mut(type_id).as_option_mut().unwrap()
    }

    #[expect(dead_code)]
    pub fn box_def(&self, type_id: TypeId) -> &BoxDef {
        self.type_def(type_id).as_box().unwrap()
    }

    pub fn box_def_mut(&mut self, type_id: TypeId) -> &mut BoxDef {
        self.type_def_mut(type_id).as_box_mut().unwrap()
    }

    #[expect(dead_code)]
    pub fn vec_def(&self, type_id: TypeId) -> &VecDef {
        self.type_def(type_id).as_vec().unwrap()
    }

    pub fn vec_def_mut(&mut self, type_id: TypeId) -> &mut VecDef {
        self.type_def_mut(type_id).as_vec_mut().unwrap()
    }

    pub fn cell_def(&self, type_id: TypeId) -> &CellDef {
        self.type_def(type_id).as_cell().unwrap()
    }

    pub fn cell_def_mut(&mut self, type_id: TypeId) -> &mut CellDef {
        self.type_def_mut(type_id).as_cell_mut().unwrap()
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
