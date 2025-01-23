use proc_macro2::TokenStream;
use quote::quote;

use super::{Def, Layout, Schema};

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
