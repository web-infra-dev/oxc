use proc_macro2::TokenStream;
use quote::quote;

use super::{Def, Layout, Schema, TypeId};

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
