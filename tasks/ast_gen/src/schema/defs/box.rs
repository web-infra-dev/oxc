use proc_macro2::TokenStream;
use quote::quote;

use super::{Def, Layout, Schema, TypeDef, TypeId};

/// Type definition for a `Box`.
#[derive(Debug)]
pub struct BoxDef {
    pub id: TypeId,
    pub name: String,
    pub inner_type_id: TypeId,
    pub layout: Layout,
}

impl BoxDef {
    /// Create new [`BoxDef`].
    pub fn new(id: TypeId, name: String, inner_type_id: TypeId) -> Self {
        Self { id, name, inner_type_id, layout: Layout::default() }
    }
}

impl Def for BoxDef {
    /// Get [`TypeId`] for type.
    fn id(&self) -> TypeId {
        self.id
    }

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
        let inner_type = schema.type_def(self.inner_type_id);
        let inner_ty = inner_type.ty_with_lifetime(schema, anon);
        let lifetime = if anon { quote!( '_ ) } else { quote!( 'a ) };
        quote!( Box<#lifetime, #inner_ty> )
    }

    /// Get inner type, if type has one.
    ///
    /// `Box`es have an inner type, so always returns `Some`.
    ///
    /// This is the direct inner type e.g. `Box<Option<Expression>>` -> `Option<Expression>`.
    /// Use [`innermost_type`] method if you want `Expression` in this example.
    ///
    /// [`innermost_type`]: Self::innermost_type
    fn maybe_inner_type<'s>(&self, schema: &'s Schema) -> Option<&'s TypeDef> {
        Some(schema.type_def(self.inner_type_id))
    }

    /// Get type's layout.
    fn layout(&self) -> &Layout {
        &self.layout
    }
}
