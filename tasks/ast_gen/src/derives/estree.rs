use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, Meta, Token};

use crate::{
    schema::{Def, EnumDef, Schema, StructDef},
    Result,
};

use super::{attr_positions, define_derive, AttrLocation, AttrPositions, Derive, StructOrEnum};

pub struct DeriveESTree;

define_derive!(DeriveESTree);

impl Derive for DeriveESTree {
    fn trait_name(&self) -> &'static str {
        "ESTree"
    }

    fn snake_name(&self) -> String {
        "estree".to_string()
    }

    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[("estree", attr_positions!(Struct | Enum | StructField | EnumVariant))]
    }

    fn parse_attr(&self, _attr_name: &str, location: AttrLocation<'_>, meta: &Meta) -> Result<()> {
        match location {
            AttrLocation::Struct(_struct_def) => {
                // TODO
                Ok(())
            }
            AttrLocation::StructField(struct_def, field_index) => {
                /*
                let attr_str = meta.to_token_stream().to_string();
                println!(
                    "> {}::{} = {}",
                    struct_def.name(),
                    struct_def.field(field_index).name().unwrap(),
                    &attr_str
                );
                */

                // TODO: Fails  to parse `estree(type = "string | null")`
                #[expect(clippy::print_stdout, clippy::overly_complex_bool_expr)]
                if struct_def.name() == "ForStatement" && false {
                    if let Meta::List(meta_list) = meta {
                        let attr_str = meta.to_token_stream().to_string();
                        println!(
                            "> {}::{} = {}",
                            struct_def.name(),
                            struct_def.field(field_index).name().unwrap(),
                            &attr_str
                        );
                        // dbg!(meta_list);

                        let punctuated = meta_list
                            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                            .unwrap();
                        for punc in &punctuated {
                            // println!("{punc:?}");
                            match punc {
                                Meta::Path(path) => {
                                    let ident = path.get_ident().unwrap();
                                    println!("path: {ident}");
                                    if ident == "skip" {
                                        struct_def.field_mut(field_index).estree.skip = true;
                                    }
                                }
                                Meta::List(list) => {
                                    println!("list: {list:?}");
                                }
                                Meta::NameValue(name_value) => {
                                    println!("name value: {name_value:?}");
                                }
                            }
                        }
                    }
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(unused_imports, unused_mut, clippy::match_same_arms)]

            ///@@line_break
            use serde::{Serialize, Serializer, ser::SerializeMap};
        }
    }

    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        /*
        if let StructOrEnum::Struct(struct_def) = type_def {
            if struct_def
                .markers
                .estree
                .as_ref()
                .and_then(|e| e.tag_mode.as_ref())
                .is_some_and(|e| e == &ESTreeStructTagMode::CustomSerialize)
            {
                return TokenStream::new();
            }
        }
        */

        let (ty, body) = match type_def {
            StructOrEnum::Struct(struct_def) => {
                (struct_def.ty_anon(schema), serialize_struct(struct_def))
            }
            StructOrEnum::Enum(enum_def) => {
                (enum_def.ty_anon(schema), serialize_enum(enum_def, schema))
            }
        };

        quote! {
            impl Serialize for #ty {
                fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                    #body
                }
            }
        }
    }
}

fn serialize_struct(struct_def: &StructDef) -> TokenStream {
    let mut fields = vec![];

    let type_name = struct_def.name();
    fields.push(quote! { map.serialize_entry("type", #type_name)?; });

    for field in &struct_def.fields {
        if field.estree.skip {
            continue;
        }

        let field_name_camel = field.camel_name().unwrap();
        let field_ident = field.ident().unwrap();
        fields.push(quote!( map.serialize_entry(#field_name_camel, &self.#field_ident)?; ));
    }

    quote! {
        let mut map = serializer.serialize_map(None)?;
        #(#fields)*
        map.end()
    }
}

fn serialize_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let enum_ident = enum_def.ident();
    let match_arms = enum_def.all_variants(schema).map(|var| {
        let var_ident = var.ident();
        quote! {
            #enum_ident::#var_ident(x) => {
                Serialize::serialize(x, serializer)
            }
        }
    });

    quote! {
        match self {
            #(#match_arms),*
        }
    }
}
