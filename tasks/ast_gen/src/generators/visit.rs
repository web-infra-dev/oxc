use quote::quote;

use crate::{
    output::{output_path, Output},
    schema::Schema,
    Generator,
};

use super::{attr_positions, define_generator, AttrPositions};

pub struct VisitGenerator;

define_generator!(VisitGenerator);

impl Generator for VisitGenerator {
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[
            ("visit", attr_positions!(StructField | EnumVariant)),
            ("scope", attr_positions!(Struct | Enum | StructField)),
        ]
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
