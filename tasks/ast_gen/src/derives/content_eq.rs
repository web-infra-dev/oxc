use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::{Schema, TypeDef};

use super::{define_derive, Derive};

pub struct DeriveContentEq;

define_derive!(DeriveContentEq);

impl Derive for DeriveContentEq {
    fn trait_name(&self) -> &'static str {
        "ContentEq"
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            // NOTE: writing long match expressions formats better than using `matches` macro.
            #![allow(clippy::match_like_matches_macro)]

            ///@@line_break
            use oxc_span::cmp::ContentEq;
        }
    }

    fn derive(&mut self, _def: &TypeDef, _: &Schema) -> TokenStream {
        quote!()
    }
}
