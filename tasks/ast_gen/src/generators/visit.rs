use quote::quote;
use syn::Meta;

use crate::{
    output::{output_path, Output},
    schema::Schema,
    Generator, Result,
};

use super::{attr_positions, define_generator, AttrLocation, AttrPositions};

pub struct VisitGenerator;

define_generator!(VisitGenerator);

impl Generator for VisitGenerator {
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[
            ("visit", attr_positions!(AstAttr | StructField | EnumVariant)),
            ("scope", attr_positions!(Struct | Enum | StructField)),
        ]
    }

    #[expect(unused_variables)]
    fn parse_attr(&self, attr_name: &str, location: AttrLocation, meta: &Meta) -> Result<()> {
        match attr_name {
            "visit" => {
                match location {
                    AttrLocation::StructAstAttr(struct_def) => {
                        struct_def.is_visited = true;
                    }
                    AttrLocation::EnumAstAttr(enum_def) => {
                        enum_def.is_visited = true;
                    }
                    AttrLocation::StructField(struct_def, field_index) => {
                        // TODO
                    }
                    AttrLocation::EnumVariant(enum_def, variant_index) => {
                        // TODO
                    }
                    _ => unreachable!(),
                }
            }
            "scope" => {
                // TODO
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    fn generate(&self, _schema: &Schema) -> Output {
        Output::Rust { path: output_path(crate::AST_CRATE, "visit.rs"), tokens: quote!() }
    }
}

pub struct VisitMutGenerator;

define_generator!(VisitMutGenerator);

impl Generator for VisitMutGenerator {
    fn generate(&self, _schema: &Schema) -> Output {
        Output::Rust { path: output_path(crate::AST_CRATE, "visit_mut.rs"), tokens: quote!() }
    }
}
