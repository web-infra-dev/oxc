use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Meta, Path};

use crate::{
    schema::{Def, EnumDef, Schema, StructDef, TypeDef},
    Result,
};

use super::{define_derive, AttrPositions, Derive};

pub struct DeriveCloneIn;

define_derive!(DeriveCloneIn);

impl Derive for DeriveCloneIn {
    fn trait_name(&self) -> &'static str {
        "CloneIn"
    }

    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[("clone_in", AttrPositions::StructField)]
    }

    /// Parse `#[clone_in(default)]` on struct field.
    fn parse_field_attr(
        &self,
        _attr_name: &str,
        meta: &Meta,
        def: &mut StructDef,
        field_index: usize,
    ) -> Result<()> {
        if let Meta::List(list) = meta {
            if let Ok(path) = list.parse_args::<Path>() {
                if path.is_ident("default") {
                    def.field_mut(field_index).clone_in_default = true;
                    return Ok(());
                }
            }
        }

        Err(())
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(clippy::default_trait_access)]

            ///@@line_break
            use oxc_allocator::{Allocator, CloneIn};
        }
    }

    fn derive(&self, def: &TypeDef, schema: &Schema) -> TokenStream {
        match def {
            TypeDef::Enum(def) => derive_enum(def, schema),
            TypeDef::Struct(def) => derive_struct(def),
            _ => unreachable!(),
        }
    }
}

fn derive_struct(def: &StructDef) -> TokenStream {
    let type_ident = def.ident();

    let (alloc_ident, body) = if def.fields.is_empty() {
        (format_ident!("_"), quote!(#type_ident))
    } else {
        let fields = def.fields.iter().map(|field| {
            let field_ident = field.ident().unwrap();
            if field.clone_in_default {
                quote!( #field_ident: Default::default() )
            } else {
                quote!( #field_ident: CloneIn::clone_in(&self.#field_ident, allocator) )
            }
        });
        (format_ident!("allocator"), quote!(#type_ident { #(#fields),* }))
    };

    generate_impl(&type_ident, def.has_lifetime, &alloc_ident, &body)
}

fn derive_enum(def: &EnumDef, schema: &Schema) -> TokenStream {
    let type_ident = def.ident();

    let mut used_alloc = false;
    let match_arms = def
        .all_variants(schema)
        .map(|variant| {
            let ident = variant.ident();
            if variant.field().is_some() {
                used_alloc = true;
                quote!(Self::#ident(it) => #type_ident::#ident(CloneIn::clone_in(it, allocator)))
            } else {
                quote!(Self::#ident => #type_ident::#ident)
            }
        })
        .collect_vec();

    let alloc_ident = if used_alloc { format_ident!("allocator") } else { format_ident!("_") };
    let body = quote! {
        match self {
            #(#match_arms,)*
        }
    };

    generate_impl(&type_ident, def.has_lifetime, &alloc_ident, &body)
}

fn generate_impl(
    type_ident: &Ident,
    has_lifetime: bool,
    alloc_ident: &Ident,
    body: &TokenStream,
) -> TokenStream {
    if has_lifetime {
        quote! {
            impl<'new_alloc> CloneIn<'new_alloc> for #type_ident<'_> {
                type Cloned = #type_ident<'new_alloc>;
                fn clone_in(&self, #alloc_ident: &'new_alloc Allocator) -> Self::Cloned {
                    #body
                }
            }
        }
    } else {
        quote! {
            impl<'alloc> CloneIn<'alloc> for #type_ident {
                type Cloned = #type_ident;
                fn clone_in(&self, #alloc_ident: &'alloc Allocator) -> Self::Cloned {
                    #body
                }
            }
        }
    }
}
