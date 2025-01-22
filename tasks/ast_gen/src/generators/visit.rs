use quote::quote;

use crate::{
    output::{output_path, Output},
    schema::Schema,
    Generator,
};

use super::define_generator;

pub struct VisitGenerator;

define_generator!(VisitGenerator);

impl Generator for VisitGenerator {
    fn type_attrs(&self) -> &[&'static str] {
        &["scope"]
    }

    fn field_attrs(&self) -> &[&'static str] {
        &["visit", "scope"]
    }

    fn variant_attrs(&self) -> &[&'static str] {
        &["visit"]
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
