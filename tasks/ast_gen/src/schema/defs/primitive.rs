use proc_macro2::TokenStream;
use quote::quote;

use super::{Def, Layout, Schema, TypeDef, TypeId};

/// Type definition for a primitive type.
///
/// Includes:
/// * Built-ins e.g. `u8`, `&str`.
/// * Special Oxc types e.g. `ScopeId`, `Atom`.
#[derive(Debug)]
pub struct PrimitiveDef {
    pub id: TypeId,
    pub name: &'static str,
    pub layout: Layout,
}

impl PrimitiveDef {
    /// Create new [`PrimitiveDef`].
    pub fn new(id: TypeId, name: &'static str) -> Self {
        Self { id, name, layout: Layout::default() }
    }
}

impl Def for PrimitiveDef {
    /// Get [`TypeId`] for type.
    fn id(&self) -> TypeId {
        self.id
    }

    /// Get type name.
    fn name(&self) -> &str {
        self.name
    }

    /// Get if type has a lifetime.
    fn has_lifetime(&self, _schema: &Schema) -> bool {
        false
    }

    /// Get type signature (including lifetimes).
    /// Lifetime is anonymous (`'_`) if `anon` is true.
    fn ty_with_lifetime(&self, _schema: &Schema, anon: bool) -> TokenStream {
        if self.name == "&str" {
            if anon {
                quote!(&str)
            } else {
                quote!(&'a str)
            }
        } else {
            let ident = self.ident();
            quote!( #ident )
        }
    }

    /// Get inner type, if type has one.
    ///
    /// Primitives don't have an inner type, so returns `None`.
    fn maybe_inner_type<'s>(&self, _schema: &'s Schema) -> Option<&'s TypeDef> {
        None
    }

    /// Get type's layout.
    fn layout(&self) -> &Layout {
        &self.layout
    }
}
