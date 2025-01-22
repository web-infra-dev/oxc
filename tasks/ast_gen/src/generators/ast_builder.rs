use quote::quote;

use crate::{
    output::{output_path, Output},
    schema::Schema,
    Generator,
};

use super::define_generator;

pub struct AstBuilderGenerator;

define_generator!(AstBuilderGenerator);

impl Generator for AstBuilderGenerator {
    fn generate(&self, _schema: &Schema) -> Output {
        Output::Rust { path: output_path(crate::AST_CRATE, "ast_builder.rs"), tokens: quote!() }
    }
}
