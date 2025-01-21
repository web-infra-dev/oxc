use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::{Schema, TypeDef};

use super::{define_derive, Derive};

pub struct DeriveESTree;

define_derive!(DeriveESTree);

impl Derive for DeriveESTree {
    fn trait_name(&self) -> &'static str {
        "ESTree"
    }

    fn snake_name(&self) -> String {
        "estree".to_string()
    }

    fn type_attrs(&self) -> &[&'static str] {
        &["estree"]
    }

    fn field_attrs(&self) -> &[&'static str] {
        &["estree"]
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(unused_imports, unused_mut, clippy::match_same_arms)]

            ///@@line_break
            use serde::{Serialize, Serializer, ser::SerializeMap};
        }
    }

    fn derive(&mut self, _def: &TypeDef, _: &Schema) -> TokenStream {
        quote!()
    }
}
