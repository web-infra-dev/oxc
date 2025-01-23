use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{utils::create_ident, Schema};

use super::{DeriveId, Derives, FileId, Layout, Offset, TypeId};

mod r#box;
mod cell;
mod r#enum;
mod option;
mod primitive;
mod r#struct;
mod r#type;
mod vec;
pub use cell::CellDef;
pub use option::OptionDef;
pub use primitive::PrimitiveDef;
pub use r#box::BoxDef;
pub use r#enum::{Discriminant, EnumDef, VariantDef};
pub use r#struct::{FieldDef, StructDef, Visibility};
pub use r#type::TypeDef;
pub use vec::VecDef;

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
