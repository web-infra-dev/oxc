use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::{Schema, TypeDef};

use super::{define_derive, Derive};

pub struct DeriveGetSpan;

define_derive!(DeriveGetSpan);

impl Derive for DeriveGetSpan {
    fn trait_name(&self) -> &'static str {
        "GetSpan"
    }

    fn field_attrs(&self) -> &[&'static str] {
        &["span"]
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(clippy::match_same_arms)]

            ///@@line_break
            use oxc_span::{Span, GetSpan};
        }
    }

    fn derive(&self, _def: &TypeDef, _: &Schema) -> TokenStream {
        quote!()
    }
}

pub struct DeriveGetSpanMut;

define_derive!(DeriveGetSpanMut);

impl Derive for DeriveGetSpanMut {
    fn trait_name(&self) -> &'static str {
        "GetSpanMut"
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(clippy::match_same_arms)]

            ///@@line_break
            use oxc_span::{Span, GetSpanMut};
        }
    }

    fn derive(&self, _def: &TypeDef, _: &Schema) -> TokenStream {
        quote!()
    }
}
