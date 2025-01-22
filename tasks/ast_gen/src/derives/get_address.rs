use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::{Schema, TypeDef};

use super::{define_derive, Derive};

pub struct DeriveGetAddress;

define_derive!(DeriveGetAddress);

impl Derive for DeriveGetAddress {
    fn trait_name(&self) -> &'static str {
        "GetAddress"
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(clippy::match_same_arms)]

            ///@@line_break
            use oxc_allocator::{Address, GetAddress};
        }
    }

    fn derive(&self, _def: &TypeDef, _: &Schema) -> TokenStream {
        quote!()
    }
}
