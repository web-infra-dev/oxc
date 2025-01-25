use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Meta};

use crate::{
    schema::{Def, EnumDef, Schema, StructDef, TypeDef},
    Result,
};

use super::{define_derive, AttrPositions, Derive};

pub struct DeriveGetSpan;

define_derive!(DeriveGetSpan);

impl Derive for DeriveGetSpan {
    fn trait_name(&self) -> &'static str {
        "GetSpan"
    }

    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[("span", AttrPositions::StructField)]
    }

    /// Parse `#[span]` on struct field.
    fn parse_field_attr(
        &self,
        _attr_name: &str,
        meta: &Meta,
        def: &mut StructDef,
        field_index: usize,
    ) -> Result<()> {
        if matches!(meta, Meta::Path(_)) {
            def.span_field_index = Some(field_index);
            Ok(())
        } else {
            Err(())
        }
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(clippy::match_same_arms)]

            ///@@line_break
            use oxc_span::{Span, GetSpan};
        }
    }

    fn derive(&self, def: &TypeDef, schema: &Schema) -> TokenStream {
        let self_type = quote!(&self);
        let result_type = quote!(Span);
        let result_expr = quote!(self.span);
        let unbox = |it| quote!( #it.as_ref() );
        let reference = |it| quote!( &#it );

        derive(
            "GetSpan",
            "span",
            &self_type,
            &result_type,
            &result_expr,
            def,
            unbox,
            reference,
            schema,
        )
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

    fn derive(&self, def: &TypeDef, schema: &Schema) -> TokenStream {
        let self_type = quote!(&mut self);
        let result_type = quote!(&mut Span);
        let result_expr = quote!(&mut self.span);
        let unbox = |it| quote!( &mut **#it );
        let reference = |it| quote!( &mut #it );

        derive(
            "GetSpanMut",
            "span_mut",
            &self_type,
            &result_type,
            &result_expr,
            def,
            unbox,
            reference,
            schema,
        )
    }
}

#[expect(clippy::too_many_arguments)]
fn derive<U, R>(
    trait_name: &str,
    method_name: &str,
    self_type: &TokenStream,
    result_type: &TokenStream,
    result_expr: &TokenStream,
    def: &TypeDef,
    unbox: U,
    reference: R,
    schema: &Schema,
) -> TokenStream
where
    U: Fn(TokenStream) -> TokenStream,
    R: Fn(TokenStream) -> TokenStream,
{
    let trait_ident = format_ident!("{trait_name}");
    let method_ident = format_ident!("{method_name}");
    match def {
        TypeDef::Struct(def) => derive_struct(
            def,
            &trait_ident,
            &method_ident,
            self_type,
            result_type,
            result_expr,
            reference,
            schema,
        ),
        TypeDef::Enum(def) => {
            derive_enum(def, &trait_ident, &method_ident, self_type, result_type, unbox, schema)
        }
        _ => unreachable!(),
    }
}

#[expect(clippy::too_many_arguments)]
fn derive_struct<R>(
    def: &StructDef,
    trait_name: &Ident,
    method_name: &Ident,
    self_type: &TokenStream,
    result_type: &TokenStream,
    result_expr: &TokenStream,
    reference: R,
    schema: &Schema,
) -> TokenStream
where
    R: Fn(TokenStream) -> TokenStream,
{
    let ty = def.ty_anon(schema);

    let result_expr = if let Some(field_index) = def.span_field_index {
        let field_ident = def.field(field_index).ident().unwrap();
        let reference = reference(quote!( self.#field_ident ));
        quote!( #trait_name::#method_name(#reference) )
    } else {
        result_expr.clone()
    };

    quote! {
        impl #trait_name for #ty {
            #[inline]
            fn #method_name(#self_type) -> #result_type {
                #result_expr
            }
        }
    }
}

fn derive_enum<U>(
    def: &EnumDef,
    trait_ident: &Ident,
    method_ident: &Ident,
    self_type: &TokenStream,
    result_type: &TokenStream,
    unbox: U,
    schema: &Schema,
) -> TokenStream
where
    U: Fn(TokenStream) -> TokenStream,
{
    let ty = def.ty_anon(schema);

    let matches = def.all_variants(schema).map(|variant| {
        let variant_ident = variant.ident();

        let mut it = quote!(it);
        let variant_type = variant.field().unwrap().type_def(schema);
        if variant_type.is_box() {
            it = unbox(it);
        }

        quote!( Self::#variant_ident(it) => #trait_ident::#method_ident(#it) )
    });

    quote! {
        impl #trait_ident for #ty {
            fn #method_ident(#self_type) -> #result_type {
                match self {
                    #(#matches),*
                }
            }
        }
    }
}
