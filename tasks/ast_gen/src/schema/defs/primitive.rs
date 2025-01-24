use proc_macro2::TokenStream;
use quote::quote;

use super::{Def, Layout, Schema};

/// Type definition for a primitive type.
///
/// Includes:
/// * Built-ins e.g. `u8`, `&str`.
/// * Special Oxc types e.g. `ScopeId`, `Atom`.
#[derive(Debug)]
pub struct PrimitiveDef {
    pub name: &'static str,
    pub layout: Layout,
}

impl PrimitiveDef {
    /// Create new [`PrimitiveDef`].
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

    /// Get type signature with lifetime.
    fn ty_with_lifetime(&self, _schema: &Schema, _anon: bool) -> TokenStream {
        let ident = self.ident();
        quote!( #ident )
    }

    /// Get type's layout.
    fn layout(&self) -> &Layout {
        &self.layout
    }
}
