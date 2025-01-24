use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::utils::create_ident;

use super::{Def, Derives, FileId, Layout, Offset, Schema, TypeDef, TypeId};

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
    pub span_field_index: Option<usize>,
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
            span_field_index: None,
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
    pub fn type_def<'s>(&self, schema: &'s Schema) -> &'s TypeDef {
        schema.type_def(self.type_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    Public,
    /// `pub(crate)` or `pub(super)`
    Restricted,
    Private,
}
