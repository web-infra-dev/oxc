use oxc_index::IndexVec;

mod defs;
mod derives;
mod file;
mod ids;
mod layout;
pub use defs::*;
pub use derives::{DeriveId, Derives};
pub use file::File;
pub use ids::*;
pub use layout::*;

/// Schema of all AST types.
#[derive(Debug)]
pub struct Schema {
    /// Type definitions
    pub types: IndexVec<TypeId, TypeDef>,
    /// Source files
    pub files: IndexVec<FileId, File>,
}

impl Schema {
    /// Get iterator over all [`TypeId`]s.
    pub fn type_ids(&self) -> IdIter<TypeId> {
        IdIter::new(&self.types)
    }

    /// Get iterator over all [`FileId`]s.
    #[expect(dead_code)]
    pub fn file_ids(&self) -> IdIter<FileId> {
        IdIter::new(&self.files)
    }

    /// Get reference to [`File`] for a [`FileId`].
    pub fn file(&self, file_id: FileId) -> &File {
        &self.files[file_id]
    }

    /// Get reference to [`TypeDef`] for a [`TypeId`].
    pub fn type_def(&self, type_id: TypeId) -> &TypeDef {
        &self.types[type_id]
    }

    /// Get mutable reference to [`TypeDef`] for a [`TypeId`].
    pub fn type_def_mut(&mut self, type_id: TypeId) -> &mut TypeDef {
        &mut self.types[type_id]
    }

    /// Get reference to [`StructDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a struct.
    pub fn struct_def(&self, type_id: TypeId) -> &StructDef {
        self.type_def(type_id).as_struct().unwrap()
    }

    /// Get mutable reference to [`StructDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a struct.
    pub fn struct_def_mut(&mut self, type_id: TypeId) -> &mut StructDef {
        self.type_def_mut(type_id).as_struct_mut().unwrap()
    }

    /// Get reference to [`EnumDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not an enum.
    pub fn enum_def(&self, type_id: TypeId) -> &EnumDef {
        self.type_def(type_id).as_enum().unwrap()
    }

    /// Get mutable reference to [`EnumDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not an enum.
    pub fn enum_def_mut(&mut self, type_id: TypeId) -> &mut EnumDef {
        self.type_def_mut(type_id).as_enum_mut().unwrap()
    }

    /// Get reference to [`PrimitiveDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a primitive.
    #[expect(dead_code)]
    pub fn primitive_def(&self, type_id: TypeId) -> &PrimitiveDef {
        self.type_def(type_id).as_primitive().unwrap()
    }

    /// Get mutable reference to [`PrimitiveDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a primitive.
    pub fn primitive_def_mut(&mut self, type_id: TypeId) -> &mut PrimitiveDef {
        self.type_def_mut(type_id).as_primitive_mut().unwrap()
    }

    /// Get reference to [`OptionDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not an `Option`.
    pub fn option_def(&self, type_id: TypeId) -> &OptionDef {
        self.type_def(type_id).as_option().unwrap()
    }

    /// Get mutable reference to [`OptionDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not an `Option`.
    pub fn option_def_mut(&mut self, type_id: TypeId) -> &mut OptionDef {
        self.type_def_mut(type_id).as_option_mut().unwrap()
    }

    /// Get reference to [`BoxDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a `Box`.
    #[expect(dead_code)]
    pub fn box_def(&self, type_id: TypeId) -> &BoxDef {
        self.type_def(type_id).as_box().unwrap()
    }

    /// Get mutable reference to [`BoxDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a `Box`.
    pub fn box_def_mut(&mut self, type_id: TypeId) -> &mut BoxDef {
        self.type_def_mut(type_id).as_box_mut().unwrap()
    }

    /// Get reference to [`VecDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a `Vec`.
    #[expect(dead_code)]
    pub fn vec_def(&self, type_id: TypeId) -> &VecDef {
        self.type_def(type_id).as_vec().unwrap()
    }

    /// Get mutable reference to [`VecDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a `Vec`.
    pub fn vec_def_mut(&mut self, type_id: TypeId) -> &mut VecDef {
        self.type_def_mut(type_id).as_vec_mut().unwrap()
    }

    /// Get reference to [`CellDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a `Cell`.
    pub fn cell_def(&self, type_id: TypeId) -> &CellDef {
        self.type_def(type_id).as_cell().unwrap()
    }

    /// Get mutable reference to [`CellDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a `Cell`.
    pub fn cell_def_mut(&mut self, type_id: TypeId) -> &mut CellDef {
        self.type_def_mut(type_id).as_cell_mut().unwrap()
    }
}
