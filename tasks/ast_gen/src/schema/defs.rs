use std::iter::FusedIterator;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{utils::create_ident, Schema};

use super::{DeriveId, Derives, FileId, Layout, Offset, TypeId};

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

    /// Get whether type is visited.
    ///
    /// Returns `true` if type is tagged `#[ast(visit)]`.
    fn is_visited(&self) -> bool {
        false
    }

    /// Get whether type has `AstKind`.
    fn has_kind(&self) -> bool {
        false
    }

    /// Get type's layout.
    fn layout(&self) -> &Layout;
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

impl TypeDef {
    pub fn as_struct(&self) -> Option<&StructDef> {
        match self {
            Self::Struct(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_struct_mut(&mut self) -> Option<&mut StructDef> {
        match self {
            Self::Struct(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_enum(&self) -> Option<&EnumDef> {
        match self {
            Self::Enum(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_enum_mut(&mut self) -> Option<&mut EnumDef> {
        match self {
            Self::Enum(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_primitive(&self) -> Option<&PrimitiveDef> {
        match self {
            Self::Primitive(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_primitive_mut(&mut self) -> Option<&mut PrimitiveDef> {
        match self {
            Self::Primitive(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_option(&self) -> Option<&OptionDef> {
        match self {
            Self::Option(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_option_mut(&mut self) -> Option<&mut OptionDef> {
        match self {
            Self::Option(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_box(&self) -> Option<&BoxDef> {
        match self {
            Self::Box(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_box_mut(&mut self) -> Option<&mut BoxDef> {
        match self {
            Self::Box(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_vec(&self) -> Option<&VecDef> {
        match self {
            Self::Vec(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_vec_mut(&mut self) -> Option<&mut VecDef> {
        match self {
            Self::Vec(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_cell(&self) -> Option<&CellDef> {
        match self {
            Self::Cell(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_cell_mut(&mut self) -> Option<&mut CellDef> {
        match self {
            Self::Cell(def) => Some(def),
            _ => None,
        }
    }
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

    /// Get whether type is visited.
    ///
    /// Returns `true` if type is tagged `#[ast(visit)]`.
    fn is_visited(&self) -> bool {
        match self {
            TypeDef::Struct(def) => def.is_visited(),
            TypeDef::Enum(def) => def.is_visited(),
            TypeDef::Primitive(def) => def.is_visited(),
            TypeDef::Option(def) => def.is_visited(),
            TypeDef::Box(def) => def.is_visited(),
            TypeDef::Vec(def) => def.is_visited(),
            TypeDef::Cell(def) => def.is_visited(),
        }
    }

    /// Get whether type has `AstKind`.
    fn has_kind(&self) -> bool {
        match self {
            TypeDef::Struct(def) => def.has_kind(),
            TypeDef::Enum(def) => def.has_kind(),
            TypeDef::Primitive(def) => def.has_kind(),
            TypeDef::Option(def) => def.has_kind(),
            TypeDef::Box(def) => def.has_kind(),
            TypeDef::Vec(def) => def.has_kind(),
            TypeDef::Cell(def) => def.has_kind(),
        }
    }

    /// Get type's layout.
    fn layout(&self) -> &Layout {
        match self {
            TypeDef::Struct(def) => def.layout(),
            TypeDef::Enum(def) => def.layout(),
            TypeDef::Primitive(def) => def.layout(),
            TypeDef::Option(def) => def.layout(),
            TypeDef::Box(def) => def.layout(),
            TypeDef::Vec(def) => def.layout(),
            TypeDef::Cell(def) => def.layout(),
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
}

#[derive(Debug)]
pub struct StructDef {
    pub name: String,
    pub has_lifetime: bool,
    pub file_id: FileId,
    pub generated_derives: Derives,
    pub fields: Vec<FieldDef>,
    pub is_visited: bool,
    pub has_kind: bool,
    pub layout: Layout,
}

impl StructDef {
    pub fn new(
        name: String,
        has_lifetime: bool,
        file_id: FileId,
        generated_derives: Derives,
        fields: Vec<FieldDef>,
        is_visited: bool,
    ) -> Self {
        Self {
            name,
            has_lifetime,
            file_id,
            generated_derives,
            fields,
            is_visited,
            has_kind: false,
            layout: Layout::default(),
        }
    }

    pub fn field(&self, field_index: usize) -> &FieldDef {
        &self.fields[field_index]
    }

    pub fn field_mut(&mut self, field_index: usize) -> &mut FieldDef {
        &mut self.fields[field_index]
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

    /// Get whether type is visited.
    ///
    /// Returns `true` if type is tagged `#[ast(visit)]`.
    fn is_visited(&self) -> bool {
        self.is_visited
    }

    /// Get whether type has `AstKind`.
    fn has_kind(&self) -> bool {
        self.has_kind
    }

    /// Get type's layout.
    fn layout(&self) -> &Layout {
        &self.layout
    }
}

#[derive(Debug)]
pub struct EnumDef {
    pub name: String,
    pub has_lifetime: bool,
    pub file_id: FileId,
    pub generated_derives: Derives,
    pub variants: Vec<VariantDef>,
    /// For `@inherits` inherited enum variants
    pub inherits: Vec<TypeId>,
    pub is_visited: bool,
    pub has_kind: bool,
    pub layout: Layout,
}

impl EnumDef {
    pub fn new(
        name: String,
        has_lifetime: bool,
        file_id: FileId,
        generated_derives: Derives,
        variants: Vec<VariantDef>,
        inherits: Vec<TypeId>,
        is_visited: bool,
    ) -> Self {
        Self {
            name,
            has_lifetime,
            file_id,
            generated_derives,
            variants,
            inherits,
            is_visited,
            has_kind: false,
            layout: Layout::default(),
        }
    }

    pub fn variant(&self, variant_index: usize) -> &VariantDef {
        &self.variants[variant_index]
    }

    #[expect(dead_code)]
    pub fn variant_mut(&mut self, variant_index: usize) -> &mut VariantDef {
        &mut self.variants[variant_index]
    }

    pub fn all_variants<'s>(&'s self, schema: &'s Schema) -> AllVariantsIter<'s> {
        AllVariantsIter::new(self, schema)
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

    /// Get whether type is visited.
    ///
    /// Returns `true` if type is tagged `#[ast(visit)]`.
    fn is_visited(&self) -> bool {
        self.is_visited
    }

    /// Get whether type has `AstKind`.
    fn has_kind(&self) -> bool {
        self.has_kind
    }

    /// Get type's layout.
    fn layout(&self) -> &Layout {
        &self.layout
    }
}

/// Iterator over all variants of an enum (including inherited).
pub struct AllVariantsIter<'s> {
    schema: &'s Schema,
    variants_iter: std::slice::Iter<'s, VariantDef>,
    inherits_iter: std::slice::Iter<'s, TypeId>,
    inner_iter: Option<Box<AllVariantsIter<'s>>>,
}

impl<'s> AllVariantsIter<'s> {
    fn new(def: &'s EnumDef, schema: &'s Schema) -> Self {
        let variants_iter = def.variants.iter();
        let inherits_iter = def.inherits.iter();
        Self { schema, variants_iter, inherits_iter, inner_iter: None }
    }
}

impl<'s> Iterator for AllVariantsIter<'s> {
    type Item = &'s VariantDef;

    fn next(&mut self) -> Option<Self::Item> {
        // Yield own variants first
        if let Some(variant) = self.variants_iter.next() {
            return Some(variant);
        }

        // Yield from inner iterator (iterating over inheritee's variants)
        if let Some(inner_iter) = &mut self.inner_iter {
            if let Some(variant) = inner_iter.next() {
                return Some(variant);
            }
            self.inner_iter = None;
        }

        // No current inner iterator. Start iterating over next inheritee.
        if let Some(&inherits_type_id) = self.inherits_iter.next() {
            let inner_type = self.schema.def_enum(inherits_type_id);
            let inner_iter = inner_type.all_variants(self.schema);
            self.inner_iter = Some(Box::new(inner_iter));
            Some(self.inner_iter.as_mut().unwrap().next().unwrap())
        } else {
            None
        }
    }
}

impl FusedIterator for AllVariantsIter<'_> {}

#[derive(Debug)]
pub struct VariantDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
    pub discriminant: Discriminant,
}

impl VariantDef {
    /// Get variant name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get variant name as an `Ident`.
    pub fn ident(&self) -> Ident {
        create_ident(self.name())
    }

    /// Get variant's field.
    ///
    /// Returns `None` if variant is fieldless.
    ///
    /// # Panics
    /// Panics if variant has more than 1 field.
    pub fn field(&self) -> Option<&FieldDef> {
        if self.fields.is_empty() {
            None
        } else {
            assert!(self.fields.len() == 1);
            Some(&self.fields[0])
        }
    }
}

#[derive(Debug)]
pub struct FieldDef {
    /// `None` if unnamed field
    pub name: Option<String>,
    pub type_id: TypeId,
    pub visibility: Visibility,
    pub offset: Offset,
    pub clone_in_default: bool,
}

impl FieldDef {
    pub fn new(name: Option<String>, type_id: TypeId, visibility: Visibility) -> Self {
        Self { name, type_id, visibility, offset: Offset::default(), clone_in_default: false }
    }

    /// Get field name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get field name, or "unnamed" if it's not named.
    pub fn name_or_unnamed(&self) -> &str {
        self.name().unwrap_or("unnamed")
    }

    /// Get field name as an `Ident`.
    pub fn ident(&self) -> Option<Ident> {
        self.name.as_ref().map(|name| create_ident(name))
    }

    /// Get field type.
    pub fn def<'s>(&self, schema: &'s Schema) -> &'s TypeDef {
        schema.def(self.type_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    Public,
    /// `pub(crate)` or `pub(super)`
    Restricted,
    Private,
}

#[derive(Debug)]
pub struct PrimitiveDef {
    pub name: &'static str,
    pub layout: Layout,
}

impl PrimitiveDef {
    pub fn new(name: &'static str) -> Self {
        Self { name, layout: Layout::default() }
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

    /// Get type's layout.
    fn layout(&self) -> &Layout {
        &self.layout
    }
}

#[derive(Debug)]
pub struct OptionDef {
    pub name: String,
    pub inner_type_id: TypeId,
    pub layout: Layout,
}

impl OptionDef {
    pub fn new(name: String, inner_type_id: TypeId) -> Self {
        Self { name, inner_type_id, layout: Layout::default() }
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

    /// Get type's layout.
    fn layout(&self) -> &Layout {
        &self.layout
    }
}

#[derive(Debug)]
pub struct BoxDef {
    pub name: String,
    pub inner_type_id: TypeId,
    pub layout: Layout,
}

impl BoxDef {
    pub fn new(name: String, inner_type_id: TypeId) -> Self {
        Self { name, inner_type_id, layout: Layout::default() }
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

    /// Get type's layout.
    fn layout(&self) -> &Layout {
        &self.layout
    }
}

#[derive(Debug)]
pub struct VecDef {
    pub name: String,
    pub inner_type_id: TypeId,
    pub layout: Layout,
}

impl VecDef {
    pub fn new(name: String, inner_type_id: TypeId) -> Self {
        Self { name, inner_type_id, layout: Layout::default() }
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

    /// Get type's layout.
    fn layout(&self) -> &Layout {
        &self.layout
    }
}

#[derive(Debug)]
pub struct CellDef {
    pub name: String,
    pub inner_type_id: TypeId,
    pub layout: Layout,
}

impl CellDef {
    pub fn new(name: String, inner_type_id: TypeId) -> Self {
        Self { name, inner_type_id, layout: Layout::default() }
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

    /// Get type's layout.
    fn layout(&self) -> &Layout {
        &self.layout
    }
}
