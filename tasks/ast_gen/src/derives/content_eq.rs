use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::schema::{Def, EnumDef, Schema, StructDef, TypeDef};

use super::{define_derive, Derive};

const IGNORE_FIELD_TYPES: [&str; 4] = ["Span", "ScopeId", "SymbolId", "ReferenceId"];

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

    fn derive(&self, type_def: &TypeDef, schema: &Schema) -> TokenStream {
        match type_def {
            TypeDef::Struct(struct_def) => derive_struct(struct_def, schema),
            TypeDef::Enum(enum_def) => derive_enum(enum_def, schema),
            _ => unreachable!(),
        }
    }
}

fn derive_struct(struct_def: &StructDef, schema: &Schema) -> TokenStream {
    let fields = struct_def
        .fields
        .iter()
        .filter(|field| {
            let innermost_type = field.type_def(schema).innermost_type(schema);
            !IGNORE_FIELD_TYPES.contains(&innermost_type.name())
        })
        .map(|field| {
            let ident = field.ident();
            quote!(ContentEq::content_eq(&self.#ident, &other.#ident))
        })
        .collect_vec();

    let (other, body) =
        if fields.is_empty() { ("_", quote!(true)) } else { ("other", quote!(#(#fields)&&*)) };

    generate_impl(&struct_def.ty_anon(schema), other, &body)
}

fn derive_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let body = if enum_def.is_fieldless() {
        // We assume fieldless enums implement `PartialEq`
        quote!(self == other)
    } else {
        let matches = enum_def.all_variants(schema).map(|variant| {
            let ident = variant.ident();
            if variant.field().is_none() {
                quote!( (Self::#ident, Self::#ident) => true )
            } else {
                quote!( (Self::#ident(a), Self::#ident(b)) => a.content_eq(b) )
            }
        });
        quote! {
            #[allow(clippy::match_same_arms)]
            match (self, other) {
                #(#matches,)*
                _ => false,
            }
        }
    };

    generate_impl(&enum_def.ty_anon(schema), "other", &body)
}

fn generate_impl(ty: &TokenStream, other_name: &str, body: &TokenStream) -> TokenStream {
    let other_ident = format_ident!("{other_name}");
    quote! {
        impl ContentEq for #ty {
            fn content_eq(&self, #other_ident: &Self) -> bool {
                #body
            }
        }
    }
}
