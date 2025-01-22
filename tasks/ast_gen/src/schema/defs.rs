#![expect(dead_code)]

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, ItemEnum, ItemStruct};

use crate::{utils::create_ident, Schema};

use super::{DeriveId, Derives, FileId, TypeId};

pub type Discriminant = u8;

#[derive(Debug)]
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
    Primitive(PrimitiveDef),
    Option(OptionDef),
    Box(BoxDef),
    Vec(VecDef),
    Cell(CellDef),
}

impl TypeDef {
    /// Get type name.
    pub fn name(&self) -> &str {
        match self {
            TypeDef::Struct(def) => &def.name,
            TypeDef::Enum(def) => &def.name,
            TypeDef::Primitive(def) => def.name,
            TypeDef::Option(def) => &def.name,
            TypeDef::Box(def) => &def.name,
            TypeDef::Vec(def) => &def.name,
            TypeDef::Cell(def) => &def.name,
        }
    }

    /// Get type name in snake case.
    pub fn snake_name(&self) -> String {
        self.name().to_case(Case::Snake)
    }

    /// Get type name as an `Ident`.
    pub fn ident(&self) -> Ident {
        create_ident(self.name())
    }

    /// Get type definition (including lifetimes).
    pub fn typ(&self, schema: &Schema) -> TokenStream {
        match self {
            TypeDef::Struct(_) | TypeDef::Enum(_) | TypeDef::Primitive(_) => {
                let ident = self.ident();
                if self.has_lifetime() {
                    quote!( #ident<'a> )
                } else {
                    quote!( #ident )
                }
            }
            TypeDef::Option(def) => {
                let inner_type = schema.def(def.inner_type_id).typ(schema);
                quote!( Option<#inner_type> )
            }
            TypeDef::Box(def) => {
                let inner_type = schema.def(def.inner_type_id).typ(schema);
                quote!( Box<'a, #inner_type> )
            }
            TypeDef::Vec(def) => {
                let inner_type = schema.def(def.inner_type_id).typ(schema);
                quote!( Vec<'a, #inner_type> )
            }
            TypeDef::Cell(def) => {
                let inner_type = schema.def(def.inner_type_id).typ(schema);
                quote!( Cell<'a, #inner_type> )
            }
        }
    }

    /// Get if has lifetime.
    pub fn has_lifetime(&self) -> bool {
        #[expect(clippy::match_same_arms)]
        match self {
            TypeDef::Struct(def) => def.has_lifetime,
            TypeDef::Enum(def) => def.has_lifetime,
            TypeDef::Primitive(_) => false, // TODO
            TypeDef::Option(_) => false,    // TODO
            TypeDef::Box(_) => true,
            TypeDef::Vec(_) => true,
            TypeDef::Cell(_) => false, // TODO
        }
    }

    /// Get `FileId`.
    pub fn file_id(&self) -> Option<FileId> {
        match self {
            TypeDef::Struct(def) => Some(def.file_id),
            TypeDef::Enum(def) => Some(def.file_id),
            _ => None,
        }
    }

    /// Get all traits which have derives generated for this type.
    pub fn generated_derives(&self) -> Derives {
        match self {
            TypeDef::Struct(def) => def.generated_derives,
            TypeDef::Enum(def) => def.generated_derives,
            _ => Derives::none(),
        }
    }

    /// Get whether a derive is generated for this type.
    pub fn generates_derive(&self, derive_id: DeriveId) -> bool {
        self.generated_derives().has(derive_id)
    }

    /// Get whether type is visitable.
    ///
    /// Returns `true` if type is tagged `#[ast(visit)]`.
    pub fn is_visitable(&self) -> bool {
        match self {
            TypeDef::Struct(def) => def.is_visitable,
            TypeDef::Enum(def) => def.is_visitable,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct StructDef {
    pub name: String,
    pub has_lifetime: bool,
    pub file_id: FileId,
    pub generated_derives: Derives,
    pub item: ItemStruct,
    pub fields: Vec<FieldDef>,
    pub is_visitable: bool,
}

#[derive(Debug)]
pub struct EnumDef {
    pub name: String,
    pub has_lifetime: bool,
    pub file_id: FileId,
    pub generated_derives: Derives,
    pub item: ItemEnum,
    pub variants: Vec<VariantDef>,
    /// For `@inherits` inherited enum variants
    pub inherits: Vec<TypeId>,
    pub is_visitable: bool,
}

#[derive(Debug)]
pub struct VariantDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
    pub discriminant: Discriminant,
}

#[derive(Debug)]
pub struct FieldDef {
    /// `None` if unnamed field
    pub name: Option<String>,
    pub type_id: TypeId,
}

#[derive(Debug)]
pub struct PrimitiveDef {
    pub name: &'static str,
}

#[derive(Debug)]
pub struct OptionDef {
    pub name: String,
    pub inner_type_id: TypeId,
}

#[derive(Debug)]
pub struct BoxDef {
    pub name: String,
    pub inner_type_id: TypeId,
}

#[derive(Debug)]
pub struct VecDef {
    pub name: String,
    pub inner_type_id: TypeId,
}

#[derive(Debug)]
pub struct CellDef {
    pub name: String,
    pub inner_type_id: TypeId,
}
