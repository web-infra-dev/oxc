use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::{Schema, TypeDef};

use super::{define_derive, Derive};

pub struct DeriveCloneIn;

define_derive!(DeriveCloneIn);

impl Derive for DeriveCloneIn {
    fn trait_name(&self) -> &'static str {
        "CloneIn"
    }

    fn field_attrs(&self) -> &[&'static str] {
        &["clone_in"]
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(clippy::default_trait_access)]

            ///@@line_break
            use oxc_allocator::{Allocator, CloneIn};
        }
    }

    fn derive(&self, _def: &TypeDef, _: &Schema) -> TokenStream {
        quote! {
            const TODO: u64 = 123;
        }
    }
}
