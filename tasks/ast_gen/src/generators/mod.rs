use syn::Meta;

use crate::{
    codegen::{attr_positions, AttrLocation, AttrPositions},
    output::Output,
    Result, Runner, Schema,
};

mod assert_layouts;
mod ast_builder;
mod ast_kind;
mod get_id;
mod typescript;
mod visit;

pub use assert_layouts::AssertLayouts;
pub use ast_builder::AstBuilderGenerator;
pub use ast_kind::AstKindGenerator;
pub use get_id::GetIdGenerator;
pub use typescript::TypescriptGenerator;
pub use visit::{VisitGenerator, VisitMutGenerator};

pub trait Generator: Runner {
    // Methods which can/must be defined by implementer.

    /// Attributes that this generator uses.
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[]
    }

    /// Parse an attribute and record information from it on struct or enum.
    #[expect(unused_variables)]
    fn parse_attr(&self, attr_name: &str, location: AttrLocation<'_>, meta: &Meta) -> Result<()> {
        Ok(())
    }

    /// Prepare for generatation, modifying schema.
    ///
    /// Runs before any `generate` or `derive` method runs.
    #[expect(unused_variables)]
    fn prepare(&self, schema: &mut Schema) {}

    /// Generate output.
    fn generate(&self, schema: &Schema) -> Output;

    // Standard methods. Should not be overriden.

    fn output(&self, schema: &Schema) -> Result<Vec<Output>> {
        Ok(vec![self.generate(schema)])
    }
}

macro_rules! define_generator {
    ($ident:ident $($lifetime:lifetime)?) => {
        const _: () = {
            use $crate::{
                codegen::{Codegen, Runner},
                output::Output,
                schema::Schema,
                Result,
            };

            impl $($lifetime)? Runner for $ident $($lifetime)? {
                fn verb(&self) -> &'static str {
                    "Generate"
                }

                fn name(&self) -> &'static str {
                    stringify!($ident)
                }

                fn file_path(&self) -> &'static str {
                    file!()
                }

                fn run(&self, schema: &Schema, _codegen: &Codegen) -> Result<Vec<Output>> {
                    self.output(schema)
                }
            }
        };
    };
}
pub(crate) use define_generator;
