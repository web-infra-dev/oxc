#![expect(dead_code)]

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, ItemEnum, ItemStruct};

use crate::{utils::create_ident, Schema};

use super::{DeriveId, Derives, FileId, TypeId};

pub type Discriminant = u8;

/// Trait for type defs.
pub trait Def {
    /// Get type name.
    fn name(&self) -> &str;

    /// Get if type has a lifetime.
    fn has_lifetime(&self, schema: &Schema) -> bool;

    /// Get type name in snake case.
    fn snake_name(&self) -> String {
        self.name().to_case(Case::Snake)
    }

    /// Get type name as an `Ident`.
    fn ident(&self) -> Ident {
        create_ident(self.name())
    }

    /// Get type signature (including lifetimes).
    fn ty(&self, schema: &Schema) -> TokenStream {
        self.ty_with_lifetime(schema, false)
    }

    /// Get type signature (including anonymous lifetimes).
    fn ty_anon(&self, schema: &Schema) -> TokenStream {
        self.ty_with_lifetime(schema, true)
    }

    /// Get type signature (including lifetimes).
    /// Lifetimes are anonymous (`'_`) if `anon` is true.
    fn ty_with_lifetime(&self, schema: &Schema, anon: bool) -> TokenStream;

    /// Get lifetime (if type has one).
    /// Lifetime is anonymous (`'_`) if `anon` is true.
    fn lifetime_maybe_anon(&self, schema: &Schema, anon: bool) -> TokenStream {
        if anon {
            self.lifetime_anon(schema)
        } else {
            self.lifetime(schema)
        }
    }

    /// Get lifetime (if type has one).
    fn lifetime(&self, schema: &Schema) -> TokenStream {
        if self.has_lifetime(schema) {
            quote!( <'a> )
        } else {
            TokenStream::new()
        }
    }

    /// Get anonymous lifetime (if type has one).
    fn lifetime_anon(&self, schema: &Schema) -> TokenStream {
        if self.has_lifetime(schema) {
            quote!( <'_> )
        } else {
            TokenStream::new()
        }
    }
}

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

impl Def for TypeDef {
    /// Get type name.
    fn name(&self) -> &str {
        match self {
            TypeDef::Struct(def) => def.name(),
            TypeDef::Enum(def) => def.name(),
            TypeDef::Primitive(def) => def.name(),
            TypeDef::Option(def) => def.name(),
            TypeDef::Box(def) => def.name(),
            TypeDef::Vec(def) => def.name(),
            TypeDef::Cell(def) => def.name(),
        }
    }

    /// Get if type has a lifetime.
    fn has_lifetime(&self, schema: &Schema) -> bool {
        match self {
            TypeDef::Struct(def) => def.has_lifetime(schema),
            TypeDef::Enum(def) => def.has_lifetime(schema),
            TypeDef::Primitive(def) => def.has_lifetime(schema),
            TypeDef::Option(def) => def.has_lifetime(schema),
            TypeDef::Box(def) => def.has_lifetime(schema),
            TypeDef::Vec(def) => def.has_lifetime(schema),
            TypeDef::Cell(def) => def.has_lifetime(schema),
        }
    }

    /// Get type signature (including anonymous lifetimes).
    fn ty_with_lifetime(&self, schema: &Schema, anon: bool) -> TokenStream {
        match self {
            TypeDef::Struct(def) => def.ty_with_lifetime(schema, anon),
            TypeDef::Enum(def) => def.ty_with_lifetime(schema, anon),
            TypeDef::Primitive(def) => def.ty_with_lifetime(schema, anon),
            TypeDef::Option(def) => def.ty_with_lifetime(schema, anon),
            TypeDef::Box(def) => def.ty_with_lifetime(schema, anon),
            TypeDef::Vec(def) => def.ty_with_lifetime(schema, anon),
            TypeDef::Cell(def) => def.ty_with_lifetime(schema, anon),
        }
    }
}

impl TypeDef {
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

impl StructDef {
    pub fn new(
        name: String,
        has_lifetime: bool,
        file_id: FileId,
        generated_derives: Derives,
        item: ItemStruct,
        fields: Vec<FieldDef>,
        is_visitable: bool,
    ) -> Self {
        Self { name, has_lifetime, file_id, generated_derives, item, fields, is_visitable }
    }
}

impl Def for StructDef {
    /// Get type name.
    fn name(&self) -> &str {
        &self.name
    }

    /// Get if type has a lifetime.
    fn has_lifetime(&self, _schema: &Schema) -> bool {
        self.has_lifetime
    }

    /// Get type signature (including lifetime).
    /// Lifetime is anonymous (`'_`) if `anon` is true.
    fn ty_with_lifetime(&self, schema: &Schema, anon: bool) -> TokenStream {
        let ident = self.ident();
        let lifetime = self.lifetime_maybe_anon(schema, anon);
        quote!( #ident #lifetime )
    }
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

impl EnumDef {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        has_lifetime: bool,
        file_id: FileId,
        generated_derives: Derives,
        item: ItemEnum,
        variants: Vec<VariantDef>,
        inherits: Vec<TypeId>,
        is_visitable: bool,
    ) -> Self {
        Self {
            name,
            has_lifetime,
            file_id,
            generated_derives,
            item,
            variants,
            inherits,
            is_visitable,
        }
    }
}

impl Def for EnumDef {
    /// Get type name.
    fn name(&self) -> &str {
        &self.name
    }

    /// Get if type has a lifetime.
    fn has_lifetime(&self, _schema: &Schema) -> bool {
        self.has_lifetime
    }

    /// Get type signature (including lifetime).
    /// Lifetime is anonymous (`'_`) if `anon` is true.
    fn ty_with_lifetime(&self, schema: &Schema, anon: bool) -> TokenStream {
        let ident = self.ident();
        let lifetime = self.lifetime_maybe_anon(schema, anon);
        quote!( #ident #lifetime )
    }
}

#[derive(Debug)]
pub struct VariantDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
    pub discriminant: Discriminant,
}

impl VariantDef {
    /// Get variant name as an `Ident`.
    pub fn ident(&self) -> Ident {
        create_ident(&self.name)
    }
}

#[derive(Debug)]
pub struct FieldDef {
    /// `None` if unnamed field
    pub name: Option<String>,
    pub type_id: TypeId,
}

impl FieldDef {
    pub fn new(name: Option<String>, type_id: TypeId) -> Self {
        Self { name, type_id }
    }
}

impl FieldDef {
    /// Get field name as an `Ident`.
    pub fn ident(&self) -> Option<Ident> {
        self.name.as_ref().map(|name| create_ident(name))
    }

    /// Get field type.
    pub fn def<'s>(&self, schema: &'s Schema) -> &'s TypeDef {
        schema.def(self.type_id)
    }
}

#[derive(Debug)]
pub struct PrimitiveDef {
    pub name: &'static str,
}

impl PrimitiveDef {
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }
}

impl Def for PrimitiveDef {
    /// Get type name.
    fn name(&self) -> &str {
        self.name
    }

    /// Get if type has a lifetime.
    fn has_lifetime(&self, _schema: &Schema) -> bool {
        false
    }

    /// Get type signature.
    fn ty_with_lifetime(&self, _schema: &Schema, _anon: bool) -> TokenStream {
        let ident = self.ident();
        quote!( #ident )
    }
}

#[derive(Debug)]
pub struct OptionDef {
    pub name: String,
    pub inner_type_id: TypeId,
}

impl OptionDef {
    pub fn new(name: String, inner_type_id: TypeId) -> Self {
        Self { name, inner_type_id }
    }
}

impl Def for OptionDef {
    /// Get type name.
    fn name(&self) -> &str {
        &self.name
    }

    /// Get if type has a lifetime.
    fn has_lifetime(&self, schema: &Schema) -> bool {
        let inner_type = schema.def(self.inner_type_id);
        inner_type.has_lifetime(schema)
    }

    /// Get type signature (including lifetimes).
    /// Lifetimes are anonymous (`'_`) if `anon` is true.
    fn ty_with_lifetime(&self, schema: &Schema, anon: bool) -> TokenStream {
        let inner_type = schema.def(self.inner_type_id);
        let inner_ty = inner_type.ty_with_lifetime(schema, anon);
        quote!( Option<#inner_ty> )
    }
}

#[derive(Debug)]
pub struct BoxDef {
    pub name: String,
    pub inner_type_id: TypeId,
}

impl BoxDef {
    pub fn new(name: String, inner_type_id: TypeId) -> Self {
        Self { name, inner_type_id }
    }
}

impl Def for BoxDef {
    /// Get type name.
    fn name(&self) -> &str {
        &self.name
    }

    /// Get if type has a lifetime.
    fn has_lifetime(&self, _schema: &Schema) -> bool {
        true
    }

    /// Get type signature (including lifetimes).
    /// Lifetimes are anonymous (`'_`) if `anon` is true.
    fn ty_with_lifetime(&self, schema: &Schema, anon: bool) -> TokenStream {
        let inner_type = schema.def(self.inner_type_id);
        let inner_ty = inner_type.ty_with_lifetime(schema, anon);
        let lifetime = if anon { quote!( '_ ) } else { quote!( 'a ) };
        quote!( Box<#lifetime, #inner_ty> )
    }
}

#[derive(Debug)]
pub struct VecDef {
    pub name: String,
    pub inner_type_id: TypeId,
}

impl VecDef {
    pub fn new(name: String, inner_type_id: TypeId) -> Self {
        Self { name, inner_type_id }
    }
}

impl Def for VecDef {
    /// Get type name.
    fn name(&self) -> &str {
        &self.name
    }

    /// Get if type has a lifetime.
    fn has_lifetime(&self, _schema: &Schema) -> bool {
        true
    }

    /// Get type signature (including lifetimes).
    /// Lifetimes are anonymous (`'_`) if `anon` is true.
    fn ty_with_lifetime(&self, schema: &Schema, anon: bool) -> TokenStream {
        let inner_type = schema.def(self.inner_type_id);
        let inner_ty = inner_type.ty_with_lifetime(schema, anon);
        let lifetime = if anon { quote!( '_ ) } else { quote!( 'a ) };
        quote!( Vec<#lifetime, #inner_ty> )
    }
}

#[derive(Debug)]
pub struct CellDef {
    pub name: String,
    pub inner_type_id: TypeId,
}

impl CellDef {
    pub fn new(name: String, inner_type_id: TypeId) -> Self {
        Self { name, inner_type_id }
    }
}

impl Def for CellDef {
    /// Get type name.
    fn name(&self) -> &str {
        &self.name
    }

    /// Get if type has a lifetime.
    fn has_lifetime(&self, schema: &Schema) -> bool {
        let inner_type = schema.def(self.inner_type_id);
        inner_type.has_lifetime(schema)
    }

    /// Get type signature (including lifetimes).
    /// Lifetimes are anonymous (`'_`) if `anon` is true.
    fn ty_with_lifetime(&self, schema: &Schema, anon: bool) -> TokenStream {
        let inner_type = schema.def(self.inner_type_id);
        let inner_ty = inner_type.ty_with_lifetime(schema, anon);
        let lifetime = if anon { quote!( '_ ) } else { quote!( 'a ) };
        quote!( Vec<#lifetime, #inner_ty> )
    }
}
