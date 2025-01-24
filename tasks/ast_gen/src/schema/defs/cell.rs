use proc_macro2::TokenStream;
use quote::quote;

use super::{Def, Layout, Schema, TypeDef, TypeId};

/// Type definition for a `Cell`.
#[derive(Debug)]
pub struct CellDef {
    pub id: TypeId,
    pub name: String,
    pub inner_type_id: TypeId,
    pub layout: Layout,
}

impl CellDef {
    /// Create new [`CellDef`].
    pub fn new(id: TypeId, name: String, inner_type_id: TypeId) -> Self {
        Self { id, name, inner_type_id, layout: Layout::default() }
    }
}

impl Def for CellDef {
    /// Get [`TypeId`] for type.
    fn id(&self) -> TypeId {
        self.id
    }

    /// Get type name.
    fn name(&self) -> &str {
        &self.name
    }

    /// Get if type has a lifetime.
    fn has_lifetime(&self, schema: &Schema) -> bool {
        let inner_type = schema.type_def(self.inner_type_id);
        inner_type.has_lifetime(schema)
    }

    /// Get type signature (including lifetimes).
    /// Lifetimes are anonymous (`'_`) if `anon` is true.
    fn ty_with_lifetime(&self, schema: &Schema, anon: bool) -> TokenStream {
        let inner_type = schema.type_def(self.inner_type_id);
        let inner_ty = inner_type.ty_with_lifetime(schema, anon);
        let lifetime = if anon { quote!( '_ ) } else { quote!( 'a ) };
        quote!( Vec<#lifetime, #inner_ty> )
    }

    /// Get inner type.
    ///
    /// This is the direct inner type e.g. `Cell<Option<ScopeId>>` -> `Option<ScopeId>`.
    /// Use [`innermost_type`] method if you want `ScopeId` in this example.
    ///
    /// [`innermost_type`]: Self::innermost_type
    fn inner_type<'s>(&self, schema: &'s Schema) -> Option<&'s TypeDef> {
        Some(schema.type_def(self.inner_type_id))
    }

    /// Get type's layout.
    fn layout(&self) -> &Layout {
        &self.layout
    }
}
