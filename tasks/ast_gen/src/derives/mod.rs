use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use rustc_hash::{FxHashMap, FxHashSet};
use syn::{parse_str, ItemUse, Meta};

use crate::{
    codegen::{AttrPositions, Codegen},
    output::{output_path, Output},
    schema::{EnumDef, Schema, StructDef, TypeDef},
    Result, Runner,
};

mod clone_in;
mod content_eq;
mod estree;
mod get_address;
mod get_span;

pub use clone_in::DeriveCloneIn;
pub use content_eq::DeriveContentEq;
pub use estree::DeriveESTree;
pub use get_address::DeriveGetAddress;
pub use get_span::{DeriveGetSpan, DeriveGetSpanMut};

pub trait Derive: Runner {
    // Methods which can/must be defined by implementer.

    /// Get trait name.
    fn trait_name(&self) -> &'static str;

    /// Get snake case trait name.
    ///
    /// Defaults to `trait_name()` converted to snake case.
    /// Can be overridden.
    fn snake_name(&self) -> String {
        self.trait_name().to_case(Case::Snake)
    }

    /// Attributes that this derive uses.
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[]
    }

    /// Process an attribute on a struct or enum.
    #[expect(unused_variables)]
    fn parse_type_attr(&self, attr_name: &str, meta: &Meta, def: &mut TypeDef) -> Result<()> {
        Ok(())
    }

    /// Process an attribute on a struct field.
    #[expect(unused_variables)]
    fn parse_field_attr(
        &self,
        attr_name: &str,
        meta: &Meta,
        def: &mut StructDef,
        field_index: usize,
    ) -> Result<()> {
        Ok(())
    }

    /// Process an attribute on an enum variant.
    #[expect(unused_variables)]
    fn parse_variant_attr(
        &self,
        attr_name: &str,
        meta: &Meta,
        def: &mut EnumDef,
        variant_index: usize,
    ) -> Result<()> {
        Ok(())
    }

    /// Generate prelude to be output at top of generated files.
    ///
    /// Defaults to no prelude.
    /// Can be overridden.
    fn prelude(&self) -> TokenStream {
        TokenStream::default()
    }

    /// Modify schema.
    /// Runs before any `generate` or `derive` method runs.
    #[expect(unused_variables)]
    fn modify(&self, schema: &mut Schema) {}

    /// Generate trait implementation for a type.
    fn derive(&self, def: &TypeDef, schema: &Schema) -> TokenStream;

    // Standard methods. Should not be overriden.

    fn template(&self, module_paths: Vec<&str>, impls: TokenStream) -> TokenStream {
        let prelude = self.prelude();

        // from `x::y::z` to `crate::y::z::*`
        let use_modules = module_paths.into_iter().map(|module_path| {
            let module_path = module_path.strip_suffix("::mod").unwrap_or(module_path);
            let local_path = ["crate"]
                .into_iter()
                .chain(module_path.split("::").skip(1))
                .chain(["*"])
                .join("::");
            let use_module: ItemUse = parse_str(format!("use {local_path};").as_str()).unwrap();
            quote! {
                ///@@line_break
                #use_module
            }
        });

        quote! {
            #prelude

            #(#use_modules)*

            ///@@line_break
            #impls
        }
    }

    fn output(&self, schema: &Schema, codegen: &Codegen) -> Result<Vec<Output>> {
        let trait_name = self.trait_name();
        let filename = format!("derive_{}.rs", self.snake_name());

        let derive_id = codegen.get_derive_id_by_name(trait_name);

        let output = schema
            .types
            .iter()
            .filter(|def| def.generates_derive(derive_id))
            .map(|def| (def, self.derive(def, schema)))
            .fold(
                FxHashMap::<&str, (FxHashSet<&str>, Vec<TokenStream>)>::default(),
                |mut acc, (def, tokens)| {
                    let file = schema.file(def.file_id().unwrap());
                    let import_path = file.import_path();
                    let krate = file.krate();
                    let streams = acc.entry(krate).or_default();
                    streams.0.insert(import_path);
                    streams.1.push(tokens);
                    acc
                },
            )
            .into_iter()
            .sorted_by(|lhs, rhs| lhs.0.cmp(rhs.0))
            .fold(Vec::new(), |mut acc, (krate, (modules, streams))| {
                let mut modules = Vec::from_iter(modules);
                modules.sort_unstable();

                let output = Output::Rust {
                    path: output_path(&format!("crates/{krate}"), &filename),
                    tokens: self.template(
                        modules,
                        streams.into_iter().fold(TokenStream::new(), |mut acc, it| {
                            acc.extend(quote! {
                                ///@@line_break
                            });
                            acc.extend(it);
                            acc
                        }),
                    ),
                };

                acc.push(output);
                acc
            });
        Ok(output)
    }
}

macro_rules! define_derive {
    ($ident:ident $($lifetime:lifetime)?) => {
        const _: () = {
            use $crate::{Output, Runner, Schema, Result, Codegen};

            impl $($lifetime)? Runner for $ident $($lifetime)? {
                fn verb(&self) -> &'static str {
                    "Derive"
                }

                fn name(&self) -> &'static str {
                    stringify!($ident)
                }

                fn file_path(&self) -> &'static str {
                    file!()
                }

                fn run(&self, schema: &Schema, codegen: &Codegen) -> Result<Vec<Output>> {
                    self.output(schema, codegen)
                }
            }
        };
    };
}
pub(crate) use define_derive;
