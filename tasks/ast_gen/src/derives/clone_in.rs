use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Meta, Path};

use crate::{
    schema::{Def, EnumDef, Schema, StructDef},
    Result,
};

use super::{define_derive, AttrLocation, AttrPositions, Derive, StructOrEnum};

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
    fn parse_attr(&self, _attr_name: &str, location: AttrLocation<'_>, meta: &Meta) -> Result<()> {
        let AttrLocation::StructField(struct_def, field_index) = location else { unreachable!() };

        if let Meta::List(list) = meta {
            if let Ok(path) = list.parse_args::<Path>() {
                if path.is_ident("default") {
                    struct_def.field_mut(field_index).clone_in_default = true;
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

    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        match type_def {
            StructOrEnum::Struct(struct_def) => derive_struct(struct_def),
            StructOrEnum::Enum(enum_def) => derive_enum(enum_def, schema),
        }
    }
}

fn derive_struct(struct_def: &StructDef) -> TokenStream {
    let type_ident = struct_def.ident();

    let (alloc_ident, body) = if struct_def.fields.is_empty() {
        (format_ident!("_"), quote!(#type_ident))
    } else {
        let fields = struct_def.fields.iter().map(|field| {
            let field_ident = field.ident().unwrap();
            if field.clone_in_default {
                quote!( #field_ident: Default::default() )
            } else {
                quote!( #field_ident: CloneIn::clone_in(&self.#field_ident, allocator) )
            }
        });
        (format_ident!("allocator"), quote!(#type_ident { #(#fields),* }))
    };

    generate_impl(&type_ident, struct_def.has_lifetime, &alloc_ident, &body)
}

fn derive_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let type_ident = enum_def.ident();

    let mut used_alloc = false;
    let match_arms = enum_def
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

    generate_impl(&type_ident, enum_def.has_lifetime, &alloc_ident, &body)
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
